# Step 3: Synchronization Hardening

Step 3 makes the existing Canvas synchronization safer and more observable while keeping the sync path sequential and database-owned by the core app.

## Implemented changes

- Canvas pagination is centralized in `crates/app/src/lms/canvas.rs`.
- Pagination follows Canvas's opaque `Link: rel=\"next\"` URLs.
- Canvas responses now check HTTP status codes before JSON decoding.
- Endpoint-specific context is attached to Canvas errors.
- 429 and 5xx responses are retried with bounded exponential backoff, honoring `Retry-After` when present.
- Course assignment/submission refreshes are deliberately sequential instead of concurrent.
- `AppReadWrite::from_env` and `AppReadonly::from_env` run embedded migrations explicitly before use.
- `load_all` now uses `App::sync_all`, which records a `sync_runs` row with status, timings, counts, and failure text.

## Current sync order

`load_all` / `sync_all` runs:

1. Observees.
2. Active courses per observed student.
3. Assignments per student-course enrollment.
4. Submissions per student-course enrollment.

Planner ingestion remains the preferred canonical week-view feed from Step 1, but it should be wired into `schedule_items` as part of the first query/API work rather than bolted onto the legacy assignment display structs. The V2 schema already has `schedule_items` and `sync_runs` fields needed for that ingestion.

## Failure behavior

- A sync run starts with `status = 'running'`.
- Successful syncs finish with `status = 'succeeded'` and observed row counts.
- Failed syncs finish with `status = 'failed'` and the error string.
- Failed syncs return the original error to the caller.
- Requests fail loudly on non-retryable Canvas statuses instead of silently decoding error pages.

## Operational notes

- Keep sync concurrency at one full sync per database.
- Back up SQLite before replacing a known-good database.
- Restore by stopping services, replacing the SQLite file, and restarting services so migrations are checked before reads/writes.
- Because Step 2 intentionally allowed a fresh schema, stale V1 data should be discarded and Canvas should be resynced.

## Follow-up work

The remaining hardening items should land with the schedule-item ingestion/query work:

- Ingest planner items as the primary `schedule_items` feed.
- Reconcile removals/unpublishing by marking affected `schedule_items.active = 0`.
- Add sync health endpoints/CLI commands over `sync_runs`.
- Add backup/restore automation around the documented procedure.
