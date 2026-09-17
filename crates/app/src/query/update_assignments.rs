use sqlx::QueryBuilder;

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

        QueryBuilder::new(
            r#"
            insert into assignments
            (id, canvas_assignment_id, course_id, name, points_possible, grading_type)
            "#,
        )
        .push_values(self.assignments.iter(), |mut bld, a| {
            bld.push_bind(a.id);
            bld.push_bind(a.id);
            bld.push_bind(a.course_id);
            bld.push_bind(&a.name);
            bld.push_bind(a.points_possible);
            bld.push_bind(&a.grading_type);
        })
        .push(
            r#"
            on conflict(id) do update
            set name = EXCLUDED.name,
              course_id = EXCLUDED.course_id,
              points_possible = EXCLUDED.points_possible,
              grading_type = EXCLUDED.grading_type,
              updated_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .build()
        .execute(pool)
        .await?;

        QueryBuilder::new(
            r#"
            insert into student_assignment_dates
            (student_id, assignment_id, due_at_utc, source)
            "#,
        )
        .push_values(self.assignments.iter(), |mut bld, a| {
            bld.push_bind(a.student_id);
            bld.push_bind(a.id);
            bld.push_bind(&a.due_at);
            bld.push_bind("canvas_assignment");
        })
        .push(
            r#"
            on conflict(student_id, assignment_id) do update
            set due_at_utc = EXCLUDED.due_at_utc,
              source = EXCLUDED.source,
              observed_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .build()
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::support::all_assignments;

    use super::*;

    use sqlx::SqlitePool;

    #[sqlx::test]
    async fn insert_new_assignments(pool: SqlitePool) {
        sqlx::query("insert into students (id, canvas_user_id, name) values (11, 11, 'Alice'), (22, 22, 'Bob')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("insert into courses (id, canvas_course_id, name) values (111, 111, 'Weather'), (222, 222, 'Cheese')")
            .execute(&pool)
            .await
            .unwrap();

        let assignments = vec![
            Assignment {
                id: 1,
                student_id: 11,
                course_id: 111,
                name: "Weather Report".to_string(),
                due_at: None,
                points_possible: Some(100.0),
                grading_type: Some("pass_fail".to_string()),
            },
            Assignment {
                id: 2,
                student_id: 22,
                course_id: 222,
                name: "Cheese analysis".to_string(),
                due_at: Some("2025-12-25".to_string()),
                points_possible: Some(10.0),
                grading_type: Some("pass_fail".to_string()),
            },
        ];
        let upsert = UpdateAssignments { assignments };

        upsert.exec(&pool).await.unwrap();

        let ids: Vec<i64> = all_assignments(&pool).await.iter().map(|a| a.id).collect();
        assert_eq!(ids, vec![1, 2]);
    }

    #[sqlx::test(fixtures("../../fixtures/assignments.sql"))]
    async fn update_existing_assignments(pool: SqlitePool) {
        let assignments = vec![
            Assignment {
                id: 1,
                student_id: 123,
                course_id: 555,
                name: "Fish Communication Exercise".to_string(),
                due_at: Some("2025-05-20".to_string()),
                points_possible: Some(100.0),
                grading_type: Some("percent".to_string()),
            },
            Assignment {
                id: 2,
                student_id: 123,
                course_id: 555,
                name: "Bird Migratory Patterns".to_string(),
                due_at: Some("2025-05-16".to_string()),
                points_possible: Some(10.0),
                grading_type: Some("pass_fail".to_string()),
            },
        ];
        let update = UpdateAssignments { assignments };
        update.exec(&pool).await.unwrap();

        let assignments = all_assignments(&pool).await;
        assert_eq!(assignments.len(), 6);
        assert_eq!(assignments[0].id, 1);
        assert_eq!(assignments[0].name, "Fish Communication Exercise");
        assert_eq!(assignments[0].due_at, Some("2025-05-20".to_string()));
        assert_eq!(assignments[0].points_possible, Some(100.0));
        assert_eq!(assignments[0].grading_type, Some("percent".to_string()));

        assert_eq!(assignments[1].id, 2);
        assert_eq!(assignments[1].name, "Bird Migratory Patterns");
        assert_eq!(assignments[1].due_at, Some("2025-05-16".to_string()));
        assert_eq!(assignments[1].points_possible, Some(10.0));
        assert_eq!(assignments[1].grading_type, Some("pass_fail".to_string()));
    }
}
