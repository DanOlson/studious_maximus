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

