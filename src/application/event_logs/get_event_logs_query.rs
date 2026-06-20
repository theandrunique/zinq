use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    application::RequestHandler,
    domain::event_log::{EventLog, data::EventLogRepository},
    error::Error,
    state::AppState,
};

pub struct GetEventLogsQuery {
    pub current_user_id: i64,
    pub after: Option<i64>,
    pub limit: i32,
}

pub struct GetEventLogsQueryHandler {
    pub event_log_repository: Arc<dyn EventLogRepository>,
}

impl GetEventLogsQueryHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            event_log_repository: Arc::clone(&state.event_log_repository),
        }
    }
}

impl RequestHandler for GetEventLogsQueryHandler {
    type Request = GetEventLogsQuery;
    type Output = Vec<EventLog>;
    type Error = Error;

    async fn handle(&self, request: Self::Request) -> Result<Self::Output, Self::Error> {
        let after = request.after.unwrap_or(0);

        let event_logs = self
            .event_log_repository
            .get_event_logs(request.current_user_id, after, request.limit)
            .await
            .map_err(Error::InternalServerError)?;

        Ok(event_logs)
    }
}
