insert into students (id, canvas_user_id, name) values
  (22, 22, 'Alice'),
  (23, 23, 'Bob');

insert into courses (id, canvas_course_id, name) values
  (1, 1, 'Shared Course');

insert into assignments (id, canvas_assignment_id, course_id, name) values
  (333, 333, 1, 'Unsubmitted Assignment'),
  (456, 456, 1, 'Submitted Assignment');

insert into student_assignment_dates (student_id, assignment_id, source) values
  (22, 333, 'fixture'),
  (22, 456, 'fixture'),
  (23, 456, 'fixture');

insert into submissions (id, canvas_submission_id, student_id, assignment_id, grade, score, submitted_at_utc, graded_at_utc, posted_at_utc, late, missing) values
  (1, 1, 22, 333, null, null, null, null, null, 1, 1),
  (2, 2, 22, 456, 'A', 95.5, '2025-05-05', '2025-05-06', '2025-05-06', 0, 0),
  (3, 3, 23, 456, 'A-', 91.25, '2025-05-05', '2025-05-06', '2025-05-06', 0, 0);
