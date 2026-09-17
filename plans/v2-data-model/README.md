# Step 2: V2 Data Model

Step 2 reshapes storage around a per-student schedule while allowing shared Canvas objects to remain shared. Existing production data and the V1 schema are intentionally not preserved; deployments should rebuild from a fresh SQLite database and resync Canvas.

## Schema overview

The V2 schema is created by `crates/app/migrations/20250330193857_create_students.sql`.

Core entities:

- `students`: one row per observed Canvas user.
- `courses`: one row per Canvas course, shared by all students in that course.
- `student_course_enrollments`: per-student enrollment state for shared courses.
- `assignments`: one row per Canvas assignment, owned by a course and shared across students.
- `student_assignment_dates`: per-student due/unlock/lock/all-day date facts for assignments.
- `submissions`: per-student assignment completion, grading, late, and missing state.
- `raw_source_documents`: Canvas source documents such as course front pages.
- `raw_source_document_versions`: immutable content versions keyed by content hash.
- `schedule_items`: canonical, provenance-bearing rows for UI/API/CLI/MCP schedule queries.
- `extraction_runs`: parser/model extraction attempts with output, confidence, and errors.
- `sync_runs`: observable sync attempts with status, counts, timings, and errors.

## Key constraints

- `courses.canvas_course_id` and `assignments.canvas_assignment_id` are unique so shared Canvas objects cannot be duplicated per student.
- `student_course_enrollments` is keyed by `(student_id, course_id)`.
- `student_assignment_dates` is keyed by `(student_id, assignment_id)`, so one student's override cannot overwrite another student's due date.
- `submissions` is unique by `(student_id, assignment_id)` in addition to Canvas submission ID.
- Source document versions are immutable by `(document_id, content_hash)`.
- `schedule_items` retain source, confidence, evidence, and `provenance_json`.

## Query compatibility

The current application query/update layer still exposes the V1 Rust structs while reading/writing V2 tables:

- Course queries join `courses` to `student_course_enrollments`.
- Assignment queries join `assignments` to `student_assignment_dates`.
- Assignment upserts write course-level assignment data separately from per-student date data.
- Submission upserts store per-student completion state without mutating assignment rows.

This keeps existing loaders/tests working while making the database safe for multiple observed students sharing courses and assignments.

## Time handling

- Timed columns use `*_utc` names and store UTC instants as text.
- True all-day dates are stored separately as school-local `all_day_date` values.
- Canonical schedule rows include `school_timezone`, defaulting to `America/Chicago`.

## Reset procedure

Because no V1 production data is preserved, reset local state with:

```sh
rm -f crates/app/db/db.sqlite
cd crates/app
sqlx database create --database-url sqlite://db.sqlite
sqlx migrate run --database-url sqlite://db.sqlite
```
