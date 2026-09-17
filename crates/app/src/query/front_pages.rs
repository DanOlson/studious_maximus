use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool};

use crate::{
    extraction::ExtractedFrontPageItem,
    models::{CourseFrontPage, StoredDocumentVersion},
};

use super::Query;

#[derive(Debug)]
pub struct StoreFrontPageVersion {
    pub page: CourseFrontPage,
}

impl Query for StoreFrontPageVersion {
    type Value = StoredDocumentVersion;

    async fn exec(&self, pool: &SqlitePool) -> anyhow::Result<Self::Value> {
        let mut tx = pool.begin().await?;
        let metadata = serde_json::json!({
            "page_id": self.page.page_id,
            "url": self.page.url,
            "updated_at": self.page.updated_at,
            "title": self.page.title,
            "published": self.page.published,
            "front_page": self.page.front_page,
        })
        .to_string();

        let url_key = self
            .page
            .url
            .clone()
            .unwrap_or_else(|| format!("canvas://courses/{}/front_page", self.page.course_id));

        let document_id: i64 = sqlx::query_scalar(
            r#"
            insert into raw_source_documents
              (source_kind, canvas_course_id, canvas_object_id, url, title, metadata_json)
            values ('canvas_front_page', ?, ?, ?, ?, ?)
            on conflict(source_kind, canvas_course_id, canvas_object_id, url) do update
            set last_seen_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                title = excluded.title,
                metadata_json = excluded.metadata_json
            returning id
            "#,
        )
        .bind(self.page.course_id)
        .bind(self.page.page_id.unwrap_or(self.page.course_id))
        .bind(&url_key)
        .bind(&self.page.title)
        .bind(&metadata)
        .fetch_one(&mut *tx)
        .await?;

        let rows = sqlx::query(
            r#"
            insert into raw_source_document_versions
              (document_id, content_hash, content_type, body, metadata_json)
            values (?, ?, 'text/html', ?, ?)
            on conflict(document_id, content_hash) do nothing
            "#,
        )
        .bind(document_id)
        .bind(&self.page.content_hash)
        .bind(&self.page.body)
        .bind(&metadata)
        .execute(&mut *tx)
        .await?
        .rows_affected();

        let version_id: i64 = sqlx::query_scalar(
            "select id from raw_source_document_versions where document_id = ? and content_hash = ?",
        )
        .bind(document_id)
        .bind(&self.page.content_hash)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(StoredDocumentVersion {
            document_id,
            version_id,
            is_new: rows == 1,
        })
    }
}

#[derive(Debug)]
pub struct RecordExtractionRun {
    pub source_document_version_id: i64,
    pub parser_version: String,
    pub output_json: String,
    pub confidence: Option<f64>,
    pub error: Option<String>,
}

impl Query for RecordExtractionRun {
    type Value = i64;

    async fn exec(&self, pool: &SqlitePool) -> anyhow::Result<Self::Value> {
        let status = if self.error.is_some() {
            "failed"
        } else {
            "succeeded"
        };
        let id = sqlx::query_scalar(
            r#"
            insert into extraction_runs
              (source_document_version_id, parser_version, status, finished_at_utc, output_json, confidence, error)
            values (?, ?, ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), ?, ?, ?)
            returning id
            "#,
        )
        .bind(self.source_document_version_id)
        .bind(&self.parser_version)
        .bind(status)
        .bind(&self.output_json)
        .bind(self.confidence)
        .bind(&self.error)
        .fetch_one(pool)
        .await?;

        Ok(id)
    }
}

#[derive(Debug)]
pub struct UpsertExtractedScheduleItems {
    pub course_id: i64,
    pub source_document_version_id: i64,
    pub extraction_run_id: i64,
    pub school_timezone: String,
    pub items: Vec<ExtractedFrontPageItem>,
}

impl Query for UpsertExtractedScheduleItems {
    type Value = usize;

    async fn exec(&self, pool: &SqlitePool) -> anyhow::Result<Self::Value> {
        if self.items.is_empty() {
            return Ok(0);
        }

        QueryBuilder::<Sqlite>::new(
            r#"
            insert into schedule_items
              (course_id, source_document_version_id, kind, title, all_day_date, school_timezone,
               status, source, confidence, evidence, provenance_json)
            "#,
        )
        .push_values(self.items.iter(), |mut bld, item| {
            bld.push_bind(self.course_id);
            bld.push_bind(self.source_document_version_id);
            bld.push_bind("front_page_event");
            bld.push_bind(&item.title);
            bld.push_bind(&item.all_day_date);
            bld.push_bind(&self.school_timezone);
            bld.push_bind("pending");
            bld.push_bind("canvas_front_page");
            bld.push_bind(item.confidence);
            bld.push_bind(&item.evidence);
            bld.push_bind(
                serde_json::json!({
                    "extraction_run_id": self.extraction_run_id,
                    "parser_version": crate::extraction::PARSER_VERSION,
                    "evidence": item.evidence,
                })
                .to_string(),
            );
        })
        .build()
        .execute(pool)
        .await?;

        Ok(self.items.len())
    }
}

#[derive(Debug, Default)]
pub struct LatestExtractionRunsQuery;

#[derive(Clone, Debug, FromRow, serde::Serialize, serde::Deserialize)]
pub struct ExtractionRunSummary {
    pub id: i64,
    pub source_document_version_id: i64,
    pub parser_version: String,
    pub status: String,
    pub started_at_utc: String,
    pub finished_at_utc: Option<String>,
    pub confidence: Option<f64>,
    pub error: Option<String>,
}

impl Query for LatestExtractionRunsQuery {
    type Value = Vec<ExtractionRunSummary>;

    async fn exec(&self, pool: &SqlitePool) -> anyhow::Result<Self::Value> {
        let runs = sqlx::query_as::<_, ExtractionRunSummary>(
            r#"
            select id, source_document_version_id, parser_version, status, started_at_utc,
                   finished_at_utc, confidence, error
            from extraction_runs
            order by started_at_utc desc, id desc
            limit 25
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(runs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn stores_only_new_document_versions_for_changed_content(pool: SqlitePool) {
        seed_course(&pool).await;
        let page = CourseFrontPage {
            course_id: 10,
            page_id: Some(99),
            title: "Home".to_string(),
            url: Some("front-page".to_string()),
            body: "<p>10/1 Quiz</p>".to_string(),
            content_hash: crate::extraction::content_hash("<p>10/1 Quiz</p>"),
            updated_at: None,
            published: Some(true),
            front_page: Some(true),
        };

        let first = StoreFrontPageVersion { page: page.clone() }
            .exec(&pool)
            .await
            .unwrap();
        let second = StoreFrontPageVersion { page }.exec(&pool).await.unwrap();

        assert!(first.is_new);
        assert!(!second.is_new);
        assert_eq!(first.version_id, second.version_id);
    }

    #[sqlx::test]
    async fn records_extraction_and_schedule_items_with_provenance(pool: SqlitePool) {
        seed_course(&pool).await;
        let run_id = RecordExtractionRun {
            source_document_version_id: 33,
            parser_version: "test-parser".to_string(),
            output_json: "[]".to_string(),
            confidence: Some(0.8),
            error: None,
        }
        .exec(&pool)
        .await
        .unwrap();

        let inserted = UpsertExtractedScheduleItems {
            course_id: 10,
            source_document_version_id: 33,
            extraction_run_id: run_id,
            school_timezone: "America/Chicago".to_string(),
            items: vec![ExtractedFrontPageItem {
                title: "Quiz".to_string(),
                all_day_date: "2025-10-01".to_string(),
                evidence: "10/1 Quiz".to_string(),
                confidence: 0.8,
            }],
        }
        .exec(&pool)
        .await
        .unwrap();

        let row: (String, String, String) = sqlx::query_as(
            "select title, source, provenance_json from schedule_items where course_id = 10",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(inserted, 1);
        assert_eq!(row.0, "Quiz");
        assert_eq!(row.1, "canvas_front_page");
        assert!(row.2.contains("extraction_run_id"));
    }

    async fn seed_course(pool: &SqlitePool) {
        sqlx::query("insert into courses (id, canvas_course_id, name) values (10, 10, 'Science')")
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("insert into raw_source_documents (id, source_kind, canvas_course_id, canvas_object_id, url, title) values (30, 'canvas_front_page', 10, 10, 'seed', 'Seed')")
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("insert into raw_source_document_versions (id, document_id, content_hash, content_type, body) values (33, 30, 'abc', 'text/html', '<p>seed</p>')")
            .execute(pool)
            .await
            .unwrap();
    }
}
