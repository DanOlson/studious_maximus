use sqlx::{QueryBuilder, Sqlite};

use crate::models::Assignment;

use super::Query;

#[derive(Debug)]
pub struct UpdateAssignments {
    pub assignments: Vec<Assignment>,
}

impl Query for UpdateAssignments {
    type Value = ();

    async fn exec(&self, pool: &sqlx::SqlitePool) -> anyhow::Result<Self::Value> {
        if self.assignments.is_empty() {
            return Ok(());
        }

        let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "insert into assignments (id, student_id, course_id, name, due_at, points_possible, grading_type) values",
        );
        for (i, assignment) in self.assignments.iter().enumerate() {
            builder.push(" (");
            builder.push_bind(assignment.id);
            builder.push(", ");
            builder.push_bind(assignment.student_id);
            builder.push(", ");
            builder.push_bind(assignment.course_id);
            builder.push(", ");
            builder.push_bind(&assignment.name);
            builder.push(", ");
            builder.push_bind(&assignment.due_at);
            builder.push(", ");
            builder.push_bind(assignment.points_possible);
            builder.push(", ");
            builder.push_bind(&assignment.grading_type);
            builder.push(")");

            if i < self.assignments.len() - 1 {
                builder.push(",");
            }
        }

        builder.push(" on conflict(id) do nothing");
        builder.build().execute(pool).await?;

        Ok(())
    }
}
