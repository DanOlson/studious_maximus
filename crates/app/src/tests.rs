use sqlx::SqlitePool;

use crate::{App, db::SqlDatabase, lms::MockLms};

#[sqlx::test(fixtures("../fixtures/students.sql"))]
async fn test_get_students(pool: SqlitePool) {
    let database = SqlDatabase::new(pool);
    let lms = MockLms::new();
    let app = App::new(lms, database);

    let students = app.get_students(None).await.unwrap();
    assert_eq!(students.len(), 2);
}
