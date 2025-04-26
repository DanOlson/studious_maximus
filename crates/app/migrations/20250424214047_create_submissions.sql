create table if not exists submissions (
  id INTEGER PRIMARY KEY NOT NULL,
  student_id INTEGER NOT NULL,
  assignment_id INTEGER NOT NULL,
  grade TEXT,
  score REAL,
  submitted_at TEXT,
  graded_at TEXT,
  posted_at TEXT,
  late INTEGER NOT NULL,
  missing INTEGER NOT NULL
);
