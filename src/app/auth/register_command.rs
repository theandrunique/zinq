use std::sync::Arc;

use crate::{
    core::ValidateExt,
    domain::{
        auth::{
            User,
            validation::{
                validate_email, validate_global_name, validate_password, validate_username,
            },
        },
        events::{DomainEvent, EventBus},
    },
    error::Error,
    infra::id_generator::IdGenerator,
    state::AppState,
};

#[derive(Debug, validator::Validate)]
pub struct RegisterCommand {
    #[validate(custom(function = validate_username))]
    pub username: String,

    #[validate(custom(function = validate_password))]
    pub password: String,

    #[validate(custom(function = validate_global_name))]
    pub global_name: String,

    #[validate(custom(function = validate_email))]
    pub email: String,
}

pub struct RegisterComandHandler {
    event_bus: Arc<EventBus>,
    id_gen: Arc<dyn IdGenerator>,
}

impl RegisterComandHandler {
    pub fn new(state: &AppState) -> Self {
        Self {
            event_bus: state.event_bus.clone(),
            id_gen: state.id_gen.clone(),
        }
    }

    pub async fn handle(&self, command: RegisterCommand) -> Result<User, Error> {
        command.validate()?;

        let new_user = User::create(
            self.id_gen.gen_id().await,
            command.username,
            command.password,
            command.global_name,
            command.email,
        );

        self.event_bus.publish(DomainEvent::UserCreate {
            user: new_user.clone(),
        });

        return Ok(new_user);
    }
}
