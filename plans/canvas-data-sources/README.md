# Step 1: Canvas Data Source Validation

This directory records the Step 1 source-of-truth decisions for the Canvas tenant. The checked-in fixtures are sanitized and intentionally disposable; production data and schema do not need to be preserved while the V2 model is introduced.

## Capture command

Use the fixture capture helper against a real observer token, then manually review the generated JSON before committing it:

```sh
CANVAS_BASE_URL="https://school.instructure.com" \
CANVAS_TOKEN="..." \
CANVAS_FIXTURE_START_DATE="2025-02-03" \
CANVAS_FIXTURE_END_DATE="2025-02-10" \
CANVAS_FIXTURE_DIR="plans/canvas-data-sources/fixtures/sanitized" \
cargo run --package app --features write --bin capture_canvas_fixtures
```

The helper follows Canvas pagination and writes observees, active courses, planner items, calendar events, assignments, submissions embedded in assignment payloads where Canvas returns them, and course front pages.

## Validated endpoints

| Data needed | Preferred endpoint | Observer/student-specific behavior | Notes |
| --- | --- | --- | --- |
| Linked students | `GET /api/v1/users/self/observees` | Honors observer token and lists linked observees. | Seed for per-student sync. |
| Active courses | `GET /api/v1/users/{student_id}/courses?enrollment_state=active` | Honors observed student context. | Use this to build student-course enrollments; do not infer enrollments globally. |
| Week feed | `GET /api/v1/planner/items?observed_user_id={student_id}&start_date=...&end_date=...` | Preferred per-student view because Canvas applies visibility and overrides in the observed-student context. | Primary source for week-at-a-glance canonical schedule items. |
| Calendar events | `GET /api/v1/calendar_events?...&context_codes[]=course_{course_id}` | Observer can read events for observed students' active courses. | Supplement planner for non-assignment course events. |
| Assignments | `GET /api/v1/courses/{course_id}/assignments?include[]=overrides&include[]=submission` | Course-level assignment data is shared; due dates may include overrides. | Store course assignment once, then store per-student date rows from planner/overrides. |
| Submissions | `GET /api/v1/courses/{course_id}/students/submissions?student_ids[]={student_id}` | Must be queried per observed student to avoid overwriting completion state. | Source for graded/missing/late/completed state. |
| Front pages | `GET /api/v1/courses/{course_id}/front_page` | Observer can read pages for active observed courses. | Store raw HTML by content hash before extraction. |

## Source preference by normalized schedule field

| Normalized field | Preferred source | Fallback/source detail |
| --- | --- | --- |
| `student_id` | Observee context used for planner/submission request | Never derive from shared course assignment alone. |
| `course_id` / course name | Active courses endpoint | Planner `context` is acceptable for display-only fallback. |
| Assignment title and Canvas link | Planner item payload | Assignment endpoint is fallback/detail source. |
| Student-specific due date | Planner item `plannable_date` / `plannable.due_at` in observed-student context | Assignment overrides provide provenance and conflict checks. |
| All-day event date | Calendar event `all_day_date` | Preserve as school-local date, not midnight UTC. |
| Timed event instant | Planner/calendar `start_at`, `end_at`, `due_at` | Normalize to UTC and retain original timezone context. |
| Completion/missing/late status | Submission endpoint and planner item state | Submission endpoint wins when conflicts occur. |
| Front-page-only dates | Front page raw HTML + extraction result | Require evidence span, parser/model version, confidence, and source version hash. |

## Front-page findings

Representative fixtures include front page content with:

- Plain paragraph/list dates.
- Tables containing weekly agendas.
- Canvas links to assignments/pages.
- An image-only schedule placeholder.

The initial extractor should parse text, lists, tables, and links deterministically. OCR should remain optional until image-only schedule fixtures are common enough to justify it.

## Timezone and date-boundary rules

- School timezone: `America/Chicago`.
- Store timed values as UTC instants plus source timezone/provenance.
- Preserve true all-day dates as local dates in the school timezone.
- Week views use Monday through Sunday in the school timezone.
- A due time at or before local school-day boundary belongs to the calendar date reported by Canvas; do not shift all-day work by converting local midnight to UTC.

## Expected normalized outputs

See `expected-normalized-results.json` for the canonical schedule rows expected from the representative fixtures.
