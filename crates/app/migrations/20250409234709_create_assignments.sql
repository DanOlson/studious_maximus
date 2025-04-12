create table if not exists assignments (
  id INTEGER PRIMARY KEY NOT NULL,
  student_id INTEGER NOT NULL,
  course_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  due_at TEXT
);
