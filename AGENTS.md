# Zinq Agent Instructions

## Rust Edition

Uses `edition = "2024"` (unstable). Requires nightly or recent stable Rust (1.90+ confirmed).

## Dev Commands

```bash
# Run the server (requires ScyllaDB running)
cargo run

# Run tests (spins up a local ScyllaDB via testcontainers)
cargo run --bin test

# DB schema management (requires ScyllaDB at 127.0.0.1:9042)
cargo run --bin db -- init
cargo run --bin db -- reset
cargo run --bin db -- recreate-table <TABLE_NAME>
```

## Database

- **ScyllaDB** (Cassandra-compatible). Start with `docker compose up -d` or the `db init` command.
- Schema lives in `migrations/1.cql`. All tests read this file and create a test keyspace per test.
- Tests require env vars `TEST_SCYLLA_HOST` and `TEST_SCYLLA_PORT` — the `scripts/test.rs` runner sets these automatically via testcontainers.

## Architecture

Приложение написано с использованием **чистой архитектуры** (Clean Architecture).

### Основные модули

#### `src/domain/` — Domain Layer
- Анемичные доменные модели (struct с данными, без логики)
- Трейты для репозиториев (например, `UserRepository`, `ChatRepository`)
- Доменные события в `events.rs` (DomainEvent enum)

#### `src/application/` — Application Layer (CQRS)
- Основной слой с бизнес-логикой
- Содержит команды и запросы (command/query handlers)
- Примеры:
  - `src/application/auth/login_command.rs` — логика входа
  - `src/application/auth/register_command.rs` — логика регистрации
  - `src/application/chats/` — команды для работы с чатами
  - `src/application/services/` — доменные сервисы (AttachmentService, AvatarService, ChannelImageService)
- Все команды реализуют трейт `RequestHandler`:
  ```rust
  impl RequestHandler for LoginCommandHandler {
      type Request = LoginCommand;
      type Output = LoginCommandResult;
      type Error = Error;
  }
  ```

#### `src/infra/` — Infrastructure Layer
- Реализации репозиториев (Scylla)
- Внешние сервисы:
  - `smtp_client` — отправка email
  - `id_generator` — генерация ID (Snowflake)
  - `s3/` — работа с S3 хранилищем
  - `auth/` — hash_handler, jwt_handler, jwks_service, totp_handler

#### `src/routers/` — Presentation Layer
- Модульная структура: `auth_router`, `chat_router`, `emoji_router`, `user_router` и т.д.
- Каждый модуль содержит функцию создания роутера:
  ```rust
  pub fn auth_router(state: AppState) -> Router { ... }
  ```
- Вложенный модуль `schemas/` — структуры для клиентов
  - `api_error.rs` — преобразование доменной ошибки в API ответ

#### `src/state.rs` — AppState
- Глобальное состояние приложения
- Содержит все сервисы и репозитории:
  ```rust
  pub struct AppState {
      pub event_bus: Arc<EventBus>,
      pub id_gen: Arc<dyn IdGenerator>,
      pub user_repository: Arc<dyn UserRepository>,
      pub s3_service: Arc<dyn S3Service>,
      pub attachment_service: Arc<AttachmentService>,
      pub avatar_service: Arc<AvatarService>,
      pub channel_image_service: Arc<ChannelImageService>,
      // ... другие сервисы
  }
  ```

#### `src/config.rs` — Configuration
- Одна функция `init_config()` для инициализации
- Загружает переменные окружения через `dotenvy`
- Парсит значения при необходимости

#### `src/error.rs` — Domain Errors
- Enum `Error` содержит все доменные ошибки
- Только коды и данные, без сообщений
- Пример:
  ```rust
  pub enum Error {
      AuthInvalidCredentials,
      UserNotFound(i64),
      Error::InternalServerError(anyhow::Error),
      // ...
  }
  ```

### Соглашения об ошибках

1. **Доменные ошибки** — используются `Error` enum из `error.rs`
2. **Не доменные ошибки** — используются `anyhow::Error`
3. **Преобразование** — инфраструктурные ошибки (например, от репозитория) конвертируются в `Error::InternalServerError(e)`:
   ```rust
   .map_err(|e| Error::InternalServerError(e))?
   ```

### Auth / JWT

- JWT uses **RSA keys** loaded from the `keys/` directory.
- `keys/` is gitignored. The server **panics on startup** if keys are missing.
- The `keys/key.pem` file exists in the repo and is required for runtime.
- Tests load keys from `"keys"` (relative to working directory at repo root).

### Config

- `.env` contains `PORT` and `SCYLLA_NODE`. All other config (SMTP, JWT, S3, keys dir) has sensible defaults or reads from env vars.
- Config is loaded via `dotenvy` at startup (`src/config.rs`).

### Tests

- Интеграционные тесты в `src/tests/`
- Тестируют команды и запросы из application слоя
- Используют реальную ScyllaDB (без моков)
- Создают `TestContext` для каждого теста:
  - Инициализирует AppState
  - Создает fresh keyspace для теста
  - Выполняет команду и проверяет результат
- Проверка доменных ошибок через макрос `assert_err!(err, Error::UserNotFound)`
- Запуск: `cargo run --bin test` — инициализирует инфраструктуру и передает env vars (DB host/port)
