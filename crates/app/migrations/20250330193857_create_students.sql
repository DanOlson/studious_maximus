pragma foreign_keys = on;

create table if not exists students (
  id INTEGER PRIMARY KEY NOT NULL,
  canvas_user_id INTEGER NOT NULL UNIQUE,
  name TEXT NOT NULL,
  sortable_name TEXT,
  raw_canvas_payload_json TEXT,
  created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

create table if not exists courses (
  id INTEGER PRIMARY KEY NOT NULL,
  canvas_course_id INTEGER NOT NULL UNIQUE,
  name TEXT NOT NULL,
  course_code TEXT,
  workflow_state TEXT,
  raw_canvas_payload_json TEXT,
  created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

create table if not exists student_course_enrollments (
  student_id INTEGER NOT NULL REFERENCES students(id) ON DELETE CASCADE,
  course_id INTEGER NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
  enrollment_status TEXT NOT NULL,
  observed_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  raw_canvas_payload_json TEXT,
  PRIMARY KEY (student_id, course_id)
);

create table if not exists assignments (
  id INTEGER PRIMARY KEY NOT NULL,
  canvas_assignment_id INTEGER NOT NULL UNIQUE,
  course_id INTEGER NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  points_possible REAL,
  grading_type TEXT,
  html_url TEXT,
  raw_canvas_payload_json TEXT,
  created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

create table if not exists student_assignment_dates (
  student_id INTEGER NOT NULL REFERENCES students(id) ON DELETE CASCADE,
  assignment_id INTEGER NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
  due_at_utc TEXT,
  unlock_at_utc TEXT,
  lock_at_utc TEXT,
  all_day_date TEXT,
  source TEXT NOT NULL,
  raw_canvas_payload_json TEXT,
  observed_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  PRIMARY KEY (student_id, assignment_id)
);

create table if not exists submissions (
  id INTEGER PRIMARY KEY NOT NULL,
  canvas_submission_id INTEGER NOT NULL UNIQUE,
  student_id INTEGER NOT NULL REFERENCES students(id) ON DELETE CASCADE,
  assignment_id INTEGER NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
  grade TEXT,
  score REAL,
  submitted_at_utc TEXT,
  graded_at_utc TEXT,
  posted_at_utc TEXT,
  workflow_state TEXT,
  late INTEGER NOT NULL DEFAULT 0 CHECK (late IN (0, 1)),
  missing INTEGER NOT NULL DEFAULT 0 CHECK (missing IN (0, 1)),
  raw_canvas_payload_json TEXT,
  observed_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  UNIQUE (student_id, assignment_id)
);

create table if not exists raw_source_documents (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  source_kind TEXT NOT NULL,
  canvas_course_id INTEGER REFERENCES courses(id) ON DELETE CASCADE,
  canvas_object_id INTEGER,
  url TEXT,
  title TEXT,
  metadata_json TEXT,
  first_seen_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  last_seen_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  UNIQUE (source_kind, canvas_course_id, canvas_object_id, url)
);

create table if not exists raw_source_document_versions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  document_id INTEGER NOT NULL REFERENCES raw_source_documents(id) ON DELETE CASCADE,
  content_hash TEXT NOT NULL,
  content_type TEXT NOT NULL,
  body TEXT NOT NULL,
  fetched_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  metadata_json TEXT,
  UNIQUE (document_id, content_hash)
);

create table if not exists schedule_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  student_id INTEGER REFERENCES students(id) ON DELETE CASCADE,
  course_id INTEGER REFERENCES courses(id) ON DELETE CASCADE,
  assignment_id INTEGER REFERENCES assignments(id) ON DELETE SET NULL,
  source_document_version_id INTEGER REFERENCES raw_source_document_versions(id) ON DELETE SET NULL,
  kind TEXT NOT NULL,
  title TEXT NOT NULL,
  starts_at_utc TEXT,
  ends_at_utc TEXT,
  due_at_utc TEXT,
  all_day_date TEXT,
  school_timezone TEXT NOT NULL DEFAULT 'America/Chicago',
  status TEXT NOT NULL,
  source TEXT NOT NULL,
  source_canvas_id INTEGER,
  confidence REAL,
  evidence TEXT,
  provenance_json TEXT NOT NULL,
  active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
  created_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  CHECK (due_at_utc IS NOT NULL OR starts_at_utc IS NOT NULL OR all_day_date IS NOT NULL)
);

create table if not exists extraction_runs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  source_document_version_id INTEGER NOT NULL REFERENCES raw_source_document_versions(id) ON DELETE CASCADE,
  parser_version TEXT NOT NULL,
  model_name TEXT,
  model_version TEXT,
  status TEXT NOT NULL,
  started_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  finished_at_utc TEXT,
  output_json TEXT,
  confidence REAL,
  error TEXT
);

create table if not exists sync_runs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  sync_kind TEXT NOT NULL,
  status TEXT NOT NULL,
  started_at_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  finished_at_utc TEXT,
  students_seen INTEGER NOT NULL DEFAULT 0,
  courses_seen INTEGER NOT NULL DEFAULT 0,
  assignments_seen INTEGER NOT NULL DEFAULT 0,
  submissions_seen INTEGER NOT NULL DEFAULT 0,
  schedule_items_seen INTEGER NOT NULL DEFAULT 0,
  error TEXT,
  metadata_json TEXT
);

create index if not exists idx_enrollments_student_status on student_course_enrollments(student_id, enrollment_status);
create index if not exists idx_enrollments_course on student_course_enrollments(course_id);
create index if not exists idx_assignments_course on assignments(course_id);
create index if not exists idx_student_assignment_dates_student_due on student_assignment_dates(student_id, due_at_utc);
create index if not exists idx_submissions_student_assignment on submissions(student_id, assignment_id);
create index if not exists idx_raw_doc_versions_document on raw_source_document_versions(document_id, fetched_at_utc);
create index if not exists idx_schedule_items_student_due on schedule_items(student_id, due_at_utc);
create index if not exists idx_schedule_items_student_day on schedule_items(student_id, all_day_date);
create index if not exists idx_schedule_items_course on schedule_items(course_id);
create index if not exists idx_schedule_items_source on schedule_items(source, source_canvas_id);
create index if not exists idx_extraction_runs_document_version on extraction_runs(source_document_version_id, started_at_utc);
create index if not exists idx_sync_runs_status on sync_runs(status, started_at_utc);
