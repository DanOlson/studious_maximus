# App

Loads student data from Canvas into a local Sqlite DB

## Binaries

### Load students

Fetches the students to which your observer account has access

`cargo run --bin load_students`

### Load Courses

Loads and persists the courses in which your students are currently enrolled

`cargo run -- bin load_courses`

### Load Assignments

Loads and persists the assignments and submissions for the courses in which your
students are enrolled.

`cargo run -- bin load_assignments`

## Live Canvas API smoke tests

A few Canvas client tests are marked `#[ignore]` so they compile in CI but only
run when explicitly requested. They load `.env`, which is ignored by git.

Required variables:

- `CANVAS_BASE_URL`
- `CANVAS_TOKEN`
- `CANVAS_LIVE_STUDENT_ID`
- `CANVAS_LIVE_COURSE_ID`

Find suitable live IDs with:

```sh
cargo run -p app --bin list_canvas_ids --features write
```

Run locally with:

```sh
cargo test -p app live_ --features write -- --ignored
```

