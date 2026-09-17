insert into students (id, canvas_user_id, name) values
  (123, 123, 'Alice'),
  (234, 234, 'Bob');

insert into courses (id, canvas_course_id, name) values
  (1, 1, 'MU330'),
  (2, 2, 'Biology'),
  (3, 3, 'Math'),
  (4, 4, 'English Literature'),
  (5, 5, 'Geometry'),
  (6, 6, 'History'),
  (7, 7, 'Band');

insert into student_course_enrollments (student_id, course_id, enrollment_status) values
  (123, 1, 'active'),
  (123, 2, 'active'),
  (123, 3, 'active'),
  (123, 4, 'active'),
  (234, 5, 'active'),
  (234, 6, 'active'),
  (234, 7, 'active');
