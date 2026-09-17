use std::collections::HashMap;

use chrono::Datelike;
use sqlx::SqlitePool;

use crate::{
    db::{self, SqlDatabase},
    extraction,
    lms::{Lms, noop},
    models::{self, AppDataFilters},
    query,
};

#[cfg(feature = "write")]
pub use crate::lms::canvas::Client;

#[derive(Clone)]
pub struct App<L, D>
where
    L: Lms,
    D: db::Db,
{
    lms: L,
    database: D,
}

struct SyncCounts {
    students_seen: usize,
    courses_seen: usize,
    assignments_seen: usize,
    submissions_seen: usize,
    schedule_items_seen: usize,
}

impl<L, D> App<L, D>
where
    L: Lms,
    D: db::Db,
{
    pub fn new(lms: L, database: D) -> Self {
        Self { lms, database }
    }

    pub async fn get_students(
        &self,
        filters: Option<AppDataFilters>,
    ) -> anyhow::Result<Vec<models::Student>> {
        let query = query::StudentsQuery::from(filters);

        let students = self.database.query(&query).await?;

        Ok(students)
    }

    pub async fn update_students(&self) -> anyhow::Result<usize> {
        let students = self.lms.get_students().await?;
        let seen = students.len();
        let update = query::UpdateStudents {
            students: students
                .into_iter()
                .map(|s| models::Student {
                    id: s.id as i64,
                    name: s.name,
                })
                .collect(),
        };
        self.database.query(&update).await?;

        Ok(seen)
    }

    pub async fn get_courses(
        &self,
        filters: Option<AppDataFilters>,
    ) -> anyhow::Result<Vec<models::Course>> {
        let query = query::CoursesQuery::from(filters);
        let courses = self.database.query(&query).await?;

        Ok(courses)
    }

    pub async fn update_courses(&self) -> anyhow::Result<usize> {
        let students = self.get_students(None).await?;
        let mut seen = 0;
        for student in students {
            let courses = self.lms.get_active_courses(student.id).await?;
            seen += courses.len();
            let update = query::UpdateCourses {
                courses: courses
                    .into_iter()
                    .map(|c| models::Course {
                        id: c.id as i64,
                        student_id: student.id,
                        name: c.name,
                        enrollment_status: models::EnrollmentStatus::Active,
                    })
                    .collect(),
            };
            self.database.query(&update).await?;
        }

        Ok(seen)
    }

    pub async fn get_assignments(
        &self,
        filters: Option<AppDataFilters>,
    ) -> anyhow::Result<Vec<models::Assignment>> {
        let query = query::AssignmentsQuery::from(filters);
        let assignments = self.database.query(&query).await?;

        Ok(assignments)
    }

    pub async fn update_assignments(&self) -> anyhow::Result<(usize, usize)> {
        let courses = self.get_courses(None).await?;
        let mut assignments_seen = 0;
        let mut submissions_seen = 0;
        for course in courses {
            // Deliberately sequential for Canvas friendliness and easier failure attribution.
            assignments_seen += self.upsert_assignments(&course).await?;
            submissions_seen += self.upsert_submissions(&course).await?;
        }

        Ok((assignments_seen, submissions_seen))
    }

    pub async fn sync_front_pages(&self) -> anyhow::Result<usize> {
        let courses = self.get_courses(None).await?;
        let mut schedule_items_seen = 0;

        for course in courses {
            let Some(page) = self.lms.get_course_front_page(course.id).await? else {
                continue;
            };
            let body = page.body.unwrap_or_default();
            if body.trim().is_empty() {
                continue;
            }

            let body_for_extraction = body.clone();
            let stored = self
                .database
                .query(&query::StoreFrontPageVersion {
                    page: models::CourseFrontPage {
                        course_id: course.id,
                        page_id: page.page_id,
                        title: page.title.unwrap_or_else(|| "Front page".to_string()),
                        url: page.url,
                        content_hash: extraction::content_hash(&body),
                        body,
                        updated_at: page.updated_at,
                        published: page.published,
                        front_page: page.front_page,
                    },
                })
                .await?;

            if !stored.is_new {
                continue;
            }

            let school_year_start = std::env::var("SCHOOL_YEAR_START")
                .ok()
                .and_then(|year| year.parse::<i32>().ok())
                .unwrap_or_else(|| chrono::Utc::now().date_naive().year());
            let items =
                extraction::extract_deterministic_items(&body_for_extraction, school_year_start);
            let confidence = if items.is_empty() {
                None
            } else {
                Some(items.iter().map(|item| item.confidence).sum::<f64>() / items.len() as f64)
            };
            let output_json = serde_json::to_string(&items)?;
            let extraction_run_id = self
                .database
                .query(&query::RecordExtractionRun {
                    source_document_version_id: stored.version_id,
                    parser_version: extraction::PARSER_VERSION.to_string(),
                    output_json,
                    confidence,
                    error: None,
                })
                .await?;
            schedule_items_seen += self
                .database
                .query(&query::UpsertExtractedScheduleItems {
                    course_id: course.id,
                    source_document_version_id: stored.version_id,
                    extraction_run_id,
                    school_timezone: "America/Chicago".to_string(),
                    items,
                })
                .await?;
        }

        Ok(schedule_items_seen)
    }

    pub async fn sync_all(&self) -> anyhow::Result<()> {
        let sync_run_id = self
            .database
            .query(&query::StartSyncRun {
                sync_kind: "canvas_full".to_string(),
            })
            .await?;

        let result = async {
            let students_seen = self.update_students().await?;
            let courses_seen = self.update_courses().await?;
            let (assignments_seen, submissions_seen) = self.update_assignments().await?;
            let schedule_items_seen = self.sync_front_pages().await?;
            anyhow::Ok(SyncCounts {
                students_seen,
                courses_seen,
                assignments_seen,
                submissions_seen,
                schedule_items_seen,
            })
        }
        .await;

        let finish = match &result {
            Ok(counts) => query::FinishSyncRun {
                id: sync_run_id,
                status: query::SyncRunStatus::Succeeded,
                students_seen: counts.students_seen as i64,
                courses_seen: counts.courses_seen as i64,
                assignments_seen: counts.assignments_seen as i64,
                submissions_seen: counts.submissions_seen as i64,
                schedule_items_seen: counts.schedule_items_seen as i64,
                error: None,
            },
            Err(error) => query::FinishSyncRun {
                id: sync_run_id,
                status: query::SyncRunStatus::Failed,
                students_seen: 0,
                courses_seen: 0,
                assignments_seen: 0,
                submissions_seen: 0,
                schedule_items_seen: 0,
                error: Some(error.to_string()),
            },
        };
        self.database.query(&finish).await?;

        result.map(|_| ())
    }

    async fn upsert_assignments(&self, course: &models::Course) -> anyhow::Result<usize> {
        let assignments = self
            .lms
            .get_course_assignments(course.student_id, course.id)
            .await?;
        let seen = assignments.len();
        let update = query::UpdateAssignments {
            assignments: assignments
                .into_iter()
                .map(|a| models::Assignment {
                    id: a.id as i64,
                    student_id: course.student_id,
                    course_id: course.id,
                    name: a.name,
                    due_at: a.due_at,
                    points_possible: a.points_possible,
                    grading_type: a.grading_type,
                })
                .collect(),
        };
        self.database.query(&update).await?;

        Ok(seen)
    }

    async fn upsert_submissions(&self, course: &models::Course) -> anyhow::Result<usize> {
        let submissions = self
            .lms
            .get_course_submissions(course.id, course.student_id)
            .await?;
        let seen = submissions.len();
        let update = query::UpdateSubmissions {
            submissions: submissions
                .into_iter()
                .map(|s| models::Submission {
                    id: s.id as i64,
                    student_id: s.student_id as i64,
                    assignment_id: s.assignment_id as i64,
                    grade: s.grade,
                    score: s.score,
                    submitted_at: s.submitted_at,
                    graded_at: s.graded_at,
                    posted_at: s.posted_at,
                    late: s.late,
                    missing: s.missing,
                })
                .collect(),
        };
        self.database.query(&update).await?;

        Ok(seen)
    }

    pub async fn get_submissions(
        &self,
        filters: Option<AppDataFilters>,
    ) -> anyhow::Result<Vec<models::Submission>> {
        let query = query::SubmissionsQuery::from(filters);
        let res = self.database.query(&query).await?;

        Ok(res)
    }

    pub async fn get_assignments_with_submissions(
        &self,
        filters: Option<AppDataFilters>,
    ) -> anyhow::Result<Vec<models::AssignmentWithSubmissions>> {
        let (assignments, submissions) = tokio::try_join!(
            self.get_assignments(filters.clone()),
            self.get_submissions(filters)
        )?;
        let mut submissions_by_assignment_id: HashMap<i64, Vec<models::Submission>> =
            HashMap::new();

        for s in submissions {
            submissions_by_assignment_id
                .entry(s.assignment_id)
                .or_default()
                .push(s);
        }

        let x = assignments
            .into_iter()
            .map(|assignment| {
                let submissions = submissions_by_assignment_id
                    .remove(&assignment.id)
                    .unwrap_or_default();
                models::AssignmentWithSubmissions {
                    assignment,
                    submissions,
                }
            })
            .collect();

        Ok(x)
    }

    pub async fn get_schedule_items(
        &self,
        filters: models::ScheduleQueryFilters,
        statuses: Vec<query::ScheduleStatusFilter>,
    ) -> anyhow::Result<Vec<models::ScheduleItem>> {
        let query = query::ScheduleItemsQuery {
            filters,
            statuses,
            now_utc: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        };

        self.database.query(&query).await
    }

    pub async fn get_sync_health(&self) -> anyhow::Result<models::SyncHealth> {
        self.database.query(&query::SyncHealthQuery).await
    }

    pub async fn get_extraction_runs(&self) -> anyhow::Result<Vec<query::ExtractionRunSummary>> {
        self.database.query(&query::LatestExtractionRunsQuery).await
    }

    pub async fn get_app_data(
        &self,
        filters: Option<AppDataFilters>,
    ) -> anyhow::Result<models::AppData> {
        let (students, courses, assignments) = tokio::try_join!(
            self.get_students(filters.clone()),
            self.get_courses(filters.clone()),
            self.get_assignments_with_submissions(filters)
        )?;

        Ok(models::AppData {
            students,
            courses,
            assignments,
        })
    }
}

#[cfg(feature = "write")]
impl AppReadWrite {
    pub async fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let db_url = std::env::var("DATABASE_URL")?;
        let canvas_token = std::env::var("CANVAS_TOKEN")?;
        let canvas_base_url = std::env::var("CANVAS_BASE_URL")?;
        let pool = SqlitePool::connect(&db_url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        let database = SqlDatabase::new(pool);
        let lms = crate::lms::canvas::Client::new(canvas_base_url, &canvas_token);

        Ok(Self::new(lms, database))
    }
}

impl AppReadonly {
    pub async fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let db_url = std::env::var("DATABASE_URL")?;
        let pool = SqlitePool::connect(&db_url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        let database = SqlDatabase::new(pool);
        let lms = crate::lms::noop::Noop;

        Ok(Self::new(lms, database))
    }
}

#[cfg(feature = "write")]
pub type AppReadWrite = App<Client, SqlDatabase>;

pub type AppReadonly = App<noop::Noop, SqlDatabase>;
