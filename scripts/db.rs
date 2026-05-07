use anyhow::Result;
use clap::{Parser, Subcommand};
use scylla::client::session::Session;
use scylla::client::session_builder::SessionBuilder;

const KEYSPACE: &str = "zinq";
const SCHEMA_PATH: &str = "migrations/1.cql";
const DEFAULT_URI: &str = "127.0.0.1:9042";

#[derive(Parser)]
#[command(name = "db")]
#[command(about = "ScyllaDB schema management utility")]
struct Cli {
    #[arg(long, default_value = DEFAULT_URI)]
    uri: String,

    #[arg(long, default_value = KEYSPACE)]
    keyspace: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Init,
    Reset,
    RecreateTable { table_name: String },
}

async fn create_session(uri: &str) -> Result<Session> {
    SessionBuilder::new()
        .known_node(uri)
        .build()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect to ScyllaDB: {}", e))
}

fn read_schema(path: &str) -> Result<String> {
    std::fs::read_to_string(path).map_err(|e| anyhow::anyhow!("Failed to read schema file: {}", e))
}

fn split_statements(cql: &str) -> Vec<String> {
    cql.split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

async fn execute_statements(session: &Session, stmts: &[String]) -> Result<()> {
    for stmt in stmts {
        if stmt.is_empty() {
            continue;
        }
        session
            .query_unpaged(stmt.as_str(), &[])
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute: {}\nStatement: {}", e, stmt))?;
    }
    Ok(())
}

fn find_create_table(schema: &str, table_name: &str) -> Vec<String> {
    let mut results = Vec::new();

    for stmt in schema.split(';') {
        let stmt_upper = stmt.to_uppercase();
        if stmt_upper.contains("CREATE TABLE") && stmt_upper.contains(&table_name.to_uppercase()) {
            let trimmed = format!("{};", stmt.trim());
            if !results.contains(&trimmed) {
                results.push(trimmed);
            }
        }
    }

    results
}

fn extract_mv_name(stmt: &str) -> String {
    let stmt_upper = stmt.to_uppercase();
    let search_start = if let Some(pos) = stmt_upper.find("CREATE MATERIALIZED VIEW") {
        pos + "CREATE MATERIALIZED VIEW".len()
    } else {
        return String::new();
    };

    let rest = &stmt[search_start..];

    let name_start = if rest.to_uppercase().contains("IF NOT EXISTS") {
        let if_pos = rest.to_uppercase().find("IF NOT EXISTS").unwrap();
        if_pos + "IF NOT EXISTS".len()
    } else {
        0
    };

    let name_part = &rest[name_start..];
    let trimmed = name_part.trim();
    let name_end = trimmed
        .find(|c: char| c.is_whitespace() || c == ';')
        .unwrap_or(trimmed.len());
    trimmed[..name_end].to_string()
}

fn find_materialized_views_for_table(
    schema: &str,
    table_name: &str,
    _keyspace: &str,
) -> Vec<String> {
    let mut results = Vec::new();

    for stmt in schema.split(';') {
        let stmt_upper = stmt.to_uppercase();
        if stmt_upper.contains("CREATE MATERIALIZED VIEW") {
            let from_pattern_with_keyspace = format!(
                "FROM {}.{}",
                _keyspace.to_uppercase(),
                table_name.to_uppercase()
            );
            let from_pattern_without_keyspace = format!("FROM {}", table_name.to_uppercase());
            if stmt_upper.contains(&from_pattern_with_keyspace)
                || stmt_upper.contains(&from_pattern_without_keyspace)
            {
                let trimmed = format!("{};", stmt.trim());
                if !results.contains(&trimmed) {
                    results.push(trimmed);
                }
            }
        }
    }

    results
}

fn get_table_statements(
    schema: &str,
    table_name: &str,
    keyspace: &str,
) -> Result<(Vec<String>, Vec<String>)> {
    let table_stmts = find_create_table(schema, table_name);
    if table_stmts.is_empty() {
        anyhow::bail!("Table '{}' not found in schema", table_name);
    }

    let mv_stmts = find_materialized_views_for_table(schema, table_name, keyspace);
    Ok((table_stmts, mv_stmts))
}

async fn cmd_init(session: &Session, keyspace: &str) -> Result<()> {
    println!("Initializing database '{}'...", keyspace);

    println!("Creating keyspace...");
    session.query_unpaged(
        format!("CREATE KEYSPACE IF NOT EXISTS {} WITH replication = {{'class': 'SimpleStrategy', 'replication_factor': 1}}", keyspace),
        &[]
    ).await?;

    session.use_keyspace(keyspace, false).await?;

    println!("Creating tables...");
    let schema = read_schema(SCHEMA_PATH)?;
    let stmts = split_statements(&schema);

    execute_statements(session, &stmts).await?;

    println!("Database initialized successfully.");
    Ok(())
}

async fn cmd_reset(session: &Session, keyspace: &str) -> Result<()> {
    println!("Resetting database '{}'...", keyspace);

    println!("Dropping keyspace...");
    session
        .query_unpaged(format!("DROP KEYSPACE IF EXISTS {}", keyspace), &[])
        .await?;

    println!("Creating keyspace...");
    session.query_unpaged(
        format!("CREATE KEYSPACE IF NOT EXISTS {} WITH replication = {{'class': 'SimpleStrategy', 'replication_factor': 1}}", keyspace),
        &[]
    ).await?;

    session.use_keyspace(keyspace, false).await?;

    println!("Recreating schema...");
    let schema = read_schema(SCHEMA_PATH)?;
    let stmts = split_statements(&schema);

    execute_statements(session, &stmts).await?;

    println!("Database reset successfully.");
    Ok(())
}

async fn cmd_recreate_table(session: &Session, table_name: &str, keyspace: &str) -> Result<()> {
    println!("Recreating table '{}'...", table_name);

    session.use_keyspace(keyspace, false).await?;

    let schema = read_schema(SCHEMA_PATH)?;
    let (table_stmts, mv_stmts) = get_table_statements(&schema, table_name, keyspace)?;

    for mv in &mv_stmts {
        let mv_name = extract_mv_name(mv);
        if !mv_name.is_empty() {
            println!("Dropping materialized view '{}'...", mv_name);
            session
                .query_unpaged(
                    format!("DROP MATERIALIZED VIEW IF EXISTS {}.{}", keyspace, mv_name),
                    &[],
                )
                .await?;
        }
    }

    println!("Dropping table '{}'...", table_name);
    session
        .query_unpaged(
            format!("DROP TABLE IF EXISTS {}.{}", keyspace, table_name),
            &[],
        )
        .await?;

    println!("Creating table...");
    for stmt in &table_stmts {
        let full_stmt = if stmt.ends_with(';') {
            stmt.clone()
        } else {
            format!("{};", stmt)
        };
        session.query_unpaged(full_stmt.as_str(), &[]).await?;
    }

    let mv_count = mv_stmts.len();
    if mv_count > 0 {
        println!("Creating {} materialized view(s)...", mv_count);
        for stmt in &mv_stmts {
            let full_stmt = if stmt.ends_with(';') {
                stmt.clone()
            } else {
                format!("{};", stmt)
            };
            session.query_unpaged(full_stmt.as_str(), &[]).await?;
        }
    }

    println!(
        "Table '{}' recreated successfully ({} MV recreated).",
        table_name, mv_count
    );
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let session = create_session(&cli.uri).await?;

    match cli.command {
        Command::Init => {
            cmd_init(&session, &cli.keyspace).await?;
        }
        Command::Reset => {
            cmd_reset(&session, &cli.keyspace).await?;
        }
        Command::RecreateTable { table_name } => {
            cmd_recreate_table(&session, &table_name, &cli.keyspace).await?;
        }
    }

    Ok(())
}
