use std::future::Future;

use crate::{prelude::Stable, query::Query};
use sqlx::SqlitePool;

pub trait Db: Stable {
    fn get_pool(&self) -> &SqlitePool;

    fn query<Q>(&self, query: &Q) -> impl Future<Output = anyhow::Result<Q::Value>> + Send
    where
        Q: Query + 'static,
    {
        async move { query.exec(self.get_pool()).await }
    }
}

#[derive(Debug)]
pub struct SqlDatabase {
    pool: SqlitePool,
}

impl SqlDatabase {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl Db for SqlDatabase {
    fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }
}
