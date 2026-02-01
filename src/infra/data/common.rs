use std::sync::Arc;

use scylla::{
    client::session::Session, deserialize::row::DeserializeRow,
    response::query_result::QueryResult, serialize::{batch::BatchValues, row::SerializeRow}, statement::batch::{Batch, BatchType},
};

pub struct ScyllaCommon {
    session: Arc<Session>,
}

impl ScyllaCommon {
    pub fn new(session: Arc<Session>) -> Self {
        Self { session }
    }

    pub async fn exec<T: SerializeRow>(
        &self,
        query: &str,
        values: T,
    ) -> Result<QueryResult, anyhow::Error> {
        let prepared = self
            .session
            .prepare(query)
            .await
            .map_err(|e| anyhow::Error::new(e))?;
        self.session
            .execute_unpaged(&prepared, values)
            .await
            .map_err(|e| anyhow::Error::new(e))
    }

    pub async fn exec_first<R, T>(&self, query: &str, values: T) -> Result<Option<R>, anyhow::Error>
    where
        T: SerializeRow,
        for<'frame, 'meta> R: DeserializeRow<'frame, 'meta>,
    {
        let res = self.exec(query, values).await?;
        let rows = res.into_rows_result().map_err(|e| anyhow::Error::new(e))?;
        rows.maybe_first_row::<R>()
            .map_err(|e| anyhow::Error::new(e))
    }

    pub async fn exec_all<R, T>(&self, query: &str, values: T) -> Result<Vec<R>, anyhow::Error>
    where
        T: SerializeRow,
        for<'frame, 'meta> R: DeserializeRow<'frame, 'meta>,
    {
        let res = self.exec(query, values).await?;
        let rows = res.into_rows_result().map_err(|e| anyhow::Error::new(e))?;

        let mut vec = Vec::new();
        let mut iter = rows.rows::<R>().map_err(|e| anyhow::Error::new(e))?;
        while let Some(row_result) = iter.next() {
            let row = row_result.map_err(|e| anyhow::Error::new(e))?;
            vec.push(row);
        }

        Ok(vec)
    }
}
