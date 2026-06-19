use std::{collections::HashMap, sync::Arc};

use scylla::{
    client::session::Session, deserialize::row::DeserializeRow,
    response::query_result::QueryResult, serialize::row::SerializeRow,
    statement::prepared::PreparedStatement,
};
use tokio::sync::RwLock;

pub struct ScyllaCommon {
    session: Arc<Session>,
    prepared: RwLock<HashMap<String, Arc<PreparedStatement>>>,
}

impl ScyllaCommon {
    pub fn new(session: Arc<Session>) -> Self {
        Self {
            session,
            prepared: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_or_prepare(
        &self,
        query: &str,
    ) -> Result<Arc<PreparedStatement>, anyhow::Error> {
        {
            let guard = self.prepared.read().await;
            if let Some(stmt) = guard.get(query) {
                return Ok(Arc::clone(stmt));
            }
        }

        let prepared = self
            .session
            .prepare(query)
            .await
            .map_err(anyhow::Error::new)?;
        let prepared = Arc::new(prepared);

        let mut guard = self.prepared.write().await;
        let entry = guard
            .entry(query.to_string())
            .or_insert_with(|| Arc::clone(&prepared));

        Ok(Arc::clone(entry))
    }

    pub async fn exec<T: SerializeRow>(
        &self,
        query: &str,
        values: T,
    ) -> Result<QueryResult, anyhow::Error> {
        let prepared = Self::get_or_prepare(self, query).await?;
        self.session
            .execute_unpaged(&prepared, values)
            .await
            .map_err(anyhow::Error::new)
    }

    pub async fn exec_first<R, T>(&self, query: &str, values: T) -> Result<Option<R>, anyhow::Error>
    where
        T: SerializeRow,
        for<'frame, 'meta> R: DeserializeRow<'frame, 'meta>,
    {
        let res = self.exec(query, values).await?;
        let rows = res.into_rows_result().map_err(anyhow::Error::new)?;
        rows.maybe_first_row::<R>().map_err(anyhow::Error::new)
    }

    pub async fn exec_all<R, T>(&self, query: &str, values: T) -> Result<Vec<R>, anyhow::Error>
    where
        T: SerializeRow,
        for<'frame, 'meta> R: DeserializeRow<'frame, 'meta>,
    {
        let res = self.exec(query, values).await?;
        let rows = res.into_rows_result().map_err(anyhow::Error::new)?;

        let mut vec = Vec::new();
        let iter = rows.rows::<R>().map_err(anyhow::Error::new)?;

        for row_result in iter {
            let row = row_result.map_err(anyhow::Error::new)?;
            vec.push(row);
        }

        Ok(vec)
    }
}
