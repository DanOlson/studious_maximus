insert into students (id, canvas_user_id, name) values
  (123, 123, 'Alice'),
  (234, 234, 'Bob');

insert into courses (id, canvas_course_id, name) values
  (555, 555, 'Biology'),
  (231, 231, 'Music'),
  (330, 330, 'Design'),
  (323, 323, 'Shop'),
  (420, 420, 'Home Ec');

insert into assignments (id, canvas_assignment_id, course_id, name, points_possible, grading_type) values
  (1, 1, 555, 'Fish Communication Report', 100.0, 'percent'),
  (2, 2, 555, 'Bird Migratory Patterns', 100.0, 'percent'),
  (3, 3, 231, 'Etude in E Minor', 16, 'pass_fail'),
  (4, 4, 330, 'Design a T-shirt', 32, 'percent'),
  (5, 5, 323, 'Build a Boat', 24, 'percent'),
  (6, 6, 420, 'Pick a Peck of Berries', 20, 'percent');

insert into student_assignment_dates (student_id, assignment_id, due_at_utc, source) values
  (123, 1, '2025-05-15', 'fixture'),
  (123, 2, null, 'fixture'),
  (123, 3, '2025-05-11', 'fixture'),
  (234, 4, '2025-05-11', 'fixture'),
  (234, 5, '2025-05-11', 'fixture'),
  (234, 6, '2025-05-11', 'fixture');
