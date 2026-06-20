pub mod auth;
pub mod chats;
pub mod event_logs;
pub mod messages;
pub mod users;

pub mod events;
pub mod meta_messages;
pub mod services;

pub trait RequestHandler {
    type Request;
    type Output;
    type Error;

    async fn handle(&self, request: Self::Request) -> Result<Self::Output, Self::Error>;
}
