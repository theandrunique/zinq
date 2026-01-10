use std::sync::Arc;

use crate::{
    domain::events::EventBus,
    infra::id_generator::{IdGenerator, SnowflakeIdGenerator},
};

#[derive(Clone)]
pub struct AppState {
    pub event_bus: Arc<EventBus>,
    pub id_gen: Arc<dyn IdGenerator>,
}

pub fn init_state() -> AppState {
    AppState {
        event_bus: Arc::new(EventBus::new()),
        id_gen: Arc::new(SnowflakeIdGenerator::new()),
    }
}
