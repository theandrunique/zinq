use std::sync::Arc;

use async_trait::async_trait;
use scylla::client::session::Session;

use crate::{
    domain::messages::{data::MessageRepository, message::Message},
    infra::data::common::ScyllaCommon,
};

pub struct ScyllaMessageRepository {
    session: Arc<Session>,
    common: ScyllaCommon,
}

impl ScyllaMessageRepository {
    pub fn new(session: Arc<Session>) -> Self {
        Self {
            session: session.clone(),
            common: ScyllaCommon::new(session),
        }
    }
}

#[async_trait]
impl MessageRepository for ScyllaMessageRepository {
    async fn upsert(&self, message: Message) -> Result<(), anyhow::Error> {
        todo!()
    }

    async fn bulk_upsert(&self, messages: Vec<Message>) -> Result<(), anyhow::Error> {
        todo!()
    }

    async fn get_by_id(
        &self,
        chat_id: i64,
        message_id: i64,
    ) -> Result<Option<Message>, anyhow::Error> {
        todo!()
    }

    async fn get_by_ids(
        &self,
        chat_id: i64,
        message_ids: Vec<i64>,
    ) -> Result<Vec<Message>, anyhow::Error> {
        todo!()
    }

    async fn get_lasts_from(&self, chat_ids: Vec<i64>) -> Result<Vec<Message>, anyhow::Error> {
        todo!()
    }

    async fn get_messages(
        &self,
        chat_id: i64,
        before: i64,
        limit: i32,
    ) -> Result<Vec<Message>, anyhow::Error> {
        todo!()
    }

    async fn delete_by_id(&self, chat_id: i64, message_id: i64) -> Result<(), anyhow::Error> {
        todo!()
    }
}
