pub mod auth;
pub mod chats;
pub mod messages;
pub mod meta_messages;
pub mod services;
pub mod users;

pub trait RequestHandler {
    type Request;
    type Output;
    type Error;

    async fn handle(&self, request: Self::Request) -> Result<Self::Output, Self::Error>;
}
