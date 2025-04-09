create table if not exists courses (
  id INTEGER PRIMARY KEY NOT NULL,
  student_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  enrollment_status TEXT NOT NULL
);

