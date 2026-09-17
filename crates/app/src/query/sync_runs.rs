use sqlx::SqlitePool;

use super::Query;

#[derive(Debug)]
pub struct StartSyncRun {
    pub sync_kind: String,
}

impl Query for StartSyncRun {
    type Value = i64;

    async fn exec(&self, pool: &SqlitePool) -> anyhow::Result<Self::Value> {
        let id = sqlx::query_scalar(
            r#"
            insert into sync_runs (sync_kind, status)
            values (?, 'running')
            returning id
            "#,
        )
        .bind(&self.sync_kind)
        .fetch_one(pool)
        .await?;

        Ok(id)
    }
}

#[derive(Debug)]
pub struct FinishSyncRun {
    pub id: i64,
    pub status: SyncRunStatus,
    pub students_seen: i64,
    pub courses_seen: i64,
    pub assignments_seen: i64,
    pub submissions_seen: i64,
    pub schedule_items_seen: i64,
    pub error: Option<String>,
}

impl Query for FinishSyncRun {
    type Value = ();

    async fn exec(&self, pool: &SqlitePool) -> anyhow::Result<Self::Value> {
        sqlx::query(
            r#"
            update sync_runs
            set status = ?,
                finished_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                students_seen = ?,
                courses_seen = ?,
                assignments_seen = ?,
                submissions_seen = ?,
                schedule_items_seen = ?,
                error = ?
            where id = ?
            "#,
        )
        .bind(self.status.as_str())
        .bind(self.students_seen)
        .bind(self.courses_seen)
        .bind(self.assignments_seen)
        .bind(self.submissions_seen)
        .bind(self.schedule_items_seen)
        .bind(&self.error)
        .bind(self.id)
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SyncRunStatus {
    Succeeded,
    Failed,
}

impl SyncRunStatus {
    fn as_str(self) -> &'static str {
        match self {
            SyncRunStatus::Succeeded => "succeeded",
            SyncRunStatus::Failed => "failed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn records_sync_run_lifecycle(pool: SqlitePool) {
        let id = StartSyncRun {
            sync_kind: "canvas_full".to_string(),
        }
        .exec(&pool)
        .await
        .unwrap();

        FinishSyncRun {
            id,
            status: SyncRunStatus::Succeeded,
            students_seen: 2,
            courses_seen: 4,
            assignments_seen: 6,
            submissions_seen: 8,
            schedule_items_seen: 10,
            error: None,
        }
        .exec(&pool)
        .await
        .unwrap();

        let row: (String, i64, i64, i64, i64, i64, Option<String>) = sqlx::query_as(
            r#"
            select status,
                   students_seen,
                   courses_seen,
                   assignments_seen,
                   submissions_seen,
                   schedule_items_seen,
                   error
            from sync_runs
            where id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(row, ("succeeded".to_string(), 2, 4, 6, 8, 10, None));
    }
}
