# Studious Maximus: High-Level Plan

## Goal

Build a home-hosted system that helps a parent and students understand what is
due, what is coming up, and what needs attention. Canvas is the primary source,
but the system should also recover important dates that teachers publish only
in course front-page content.

The same underlying schedule and status rules should power:

- A mobile-friendly, week-at-a-glance web interface.
- A CLI for administration, troubleshooting, scripting, and data access.
- A read-only MCP server with focused tools for AI agents.

## Proposed Architecture

Use Rust for the main application. Keep SQLite initially, with one core service
owning database access. Other interfaces should consume a stable internal HTTP
API rather than reading the SQLite file directly.

```text
Canvas
   |
   v
studious-web/core
  |- scheduled synchronization
  |- front-page extraction
  |- SQLite ownership and migrations
  |- week-at-a-glance web UI
  `- internal JSON API
          |
          |- studious CLI
          `- studious MCP server
```

The web/core service and MCP server can be deployed as separate home-server
services. A narrowly scoped Python extraction worker remains an option if local
OCR or model tooling eventually makes it worthwhile, but it is not required for
the initial design.

## Step 0: Modernize the Rust Foundation

Before making architectural changes, bring the project onto a current,
reproducible Rust foundation:

- Select and pin a current stable Rust toolchain with `rust-toolchain.toml`.
- Update Rust versions used by builder images to match the pinned toolchain.
- Inventory direct and transitive crate updates, including breaking releases.
- Upgrade crates in controlled groups and address deprecated APIs.
- Review feature flags so read-only binaries do not pull in sync-only concerns.
- Run formatting, clippy, tests, and a dependency/security audit in CI.
- Add or update CI so the supported toolchain and all workspace targets are
  checked consistently.
- Confirm release container builds remain reproducible after the upgrades.

Exit criteria: the workspace builds cleanly on the pinned toolchain, tests pass,
clippy has no unreviewed warnings, known dependency advisories are addressed or
documented, and both service images build successfully.

## Step 1: Validate Canvas Data Sources

Status: implemented in `plans/canvas-data-sources/` with sanitized representative
fixtures, expected normalized results, endpoint/source-preference notes, timezone
rules, and a reusable fixture-capture helper.

- Capture sanitized fixtures from the real Canvas tenant for:
  - Linked observees and active courses.
  - Planner items for each observed student.
  - Calendar events.
  - Assignments, student-specific dates, and submissions.
  - Course front pages, including representative HTML structures.
- Confirm which endpoints honor observer permissions and student-specific due
  date overrides in this school configuration.
- Determine whether relevant front-page dates appear in text, tables, linked
  documents, or images.
- Document the school timezone and date-boundary rules.

Exit criteria: representative fixtures and expected normalized results exist,
and the preferred source for each schedule field is documented.

## Step 2: Introduce the V2 Data Model

Status: implemented in `plans/v2-data-model/` and the fresh-start SQLite
migrations. The schema now separates shared Canvas courses/assignments from
per-student enrollments, dates, submissions, schedule items, raw source document
versions, extraction runs, and sync runs.

Reshape storage around a per-student schedule rather than treating assignments
as the complete product model. Existing production data/schema do not need to be
preserved; the V2 work may replace migrations and start from a fresh database.
The schema should include concepts equivalent to:

- Students, courses, and student-course enrollments.
- Course-level assignments and student-specific assignment dates.
- Student submissions and completion state.
- Raw source documents and immutable document versions.
- Canonical schedule items used by all user interfaces.
- Extraction runs with parser/model versions, output, confidence, and errors.
- Sync runs with status, timings, counts, and errors.

Add foreign keys, uniqueness constraints, and query indexes. Preserve raw Canvas
payloads where useful. Normalize timed values to UTC while retaining school
timezone and genuine all-day dates. Every derived item must retain source and
provenance information.

Exit criteria: multiple students can safely share courses and Canvas objects,
student-specific dates cannot overwrite each other, and schedule queries have a
clear canonical representation.

## Step 3: Harden Synchronization

Status: implemented foundation in `plans/sync-hardening/` and the sync code:
centralized Canvas pagination, status checks, bounded retries/backoff, sequential
assignment/submission refresh, explicit migrations, and recorded `sync_runs`.
Planner-to-`schedule_items` ingestion and stale-item reconciliation are queued to
land with the schedule query/API work.

- Centralize Canvas pagination and always follow opaque `Link` URLs.
- Check HTTP status codes and provide useful endpoint-specific errors.
- Add bounded retries and backoff for throttling and transient failures.
- Prefer sequential or deliberately limited request concurrency.
- Ingest Planner items as the primary week-view feed.
- Supplement Planner data with assignments, submissions, and calendar events.
- Reconcile removals, unpublishing, concluded enrollments, and changed dates
  instead of only upserting records forever.
- Make each sync idempotent and record sync progress and failures.
- Avoid publishing partially refreshed results where practical.
- Run migrations explicitly during deployment and establish SQLite backup and
  restore procedures.

Exit criteria: repeated syncs are safe, stale records are handled, failures are
observable, and a partial Canvas outage does not silently corrupt the schedule.

## Step 4: Build the Core Query API and Web Interface

Create stable query operations first, then use them from the web interface:

- Week view for one student or all students.
- Upcoming items over an explicit date range.
- Overdue, missing, completed, and recently graded items.
- Historical views and item/source details.
- Sync freshness and health status.

Build a small server-rendered, responsive interface with progressive enhancement
where useful. The default view should be the current school week, grouped by day
and clearly distinguish student, course, kind, status, source, and confidence.

Exit criteria: the web interface is useful on desktop and mobile and all date and
status calculations come from reusable core queries rather than view-specific
logic.

## Step 5: Add Front-Page Extraction

Use a provenance-preserving, layered pipeline:

1. Fetch each active course's front page.
2. Store its metadata and raw HTML, and create a new version only when its
   content hash changes.
3. Parse the HTML into normalized structured text while preserving headings,
   lists, tables, emphasis, and links.
4. Extract unambiguous dates and linked Canvas objects deterministically.
5. Send only unresolved relevant content to an LLM using a strict structured
   output schema.
6. Validate dates against the school timezone, school year, page update time,
   and surrounding headings.
7. Store evidence, parser/model version, confidence, and validation results.
8. Present ambiguous results for human review rather than silently accepting
   them.

Treat page content as untrusted input. The extraction model should have no tools
or write access. Never overwrite the raw source with model output. Minimize the
student data sent to hosted models, and retain the option to use a local model.
Add OCR only if representative pages show that schedules are embedded in images.

Exit criteria: unchanged pages incur no repeated extraction cost, every derived
event is traceable to source content, and uncertain dates are visible for review.

## Step 6: Complete the CLI

Make the CLI a client of the core API and support both human-readable and JSON
output. Initial commands should cover:

- Week, upcoming, overdue, and history queries.
- Student and course listing.
- Schedule item and source inspection.
- Sync status and an authenticated manual sync trigger.
- Front-page extraction status and review support.

Exit criteria: the CLI exposes the same schedule semantics as the web interface
and can be used to diagnose ingestion without direct database access.

## Step 7: Refine the MCP Server

Replace the broad prose dump with narrow, structured, read-only tools such as:

- `list_students`
- `get_week`
- `get_upcoming`
- `get_overdue`
- `get_item_details`
- `search_schedule`
- `get_sync_status`

Have MCP call the core API rather than open SQLite. Bound all list operations by
student and/or date range where appropriate. Return source, confidence,
completion state, and Canvas links so agents can explain their answers.

Exit criteria: an agent can reliably answer questions about important upcoming
dates without receiving the entire database or parsing custom prose.

## Step 8: Home-Server Operations and Security

- Deploy the web/core and MCP services separately with health checks.
- Keep the Canvas token exclusively in the core/sync service.
- Protect web, API, manual-sync, and network MCP access with authentication.
- Prefer a trusted LAN or private VPN and do not publish unauthenticated MCP
  ports directly to the internet.
- Run containers as non-root users with minimal filesystem access.
- Add structured logs, retention limits, sync failure visibility, and backups.
- Document installation, configuration, upgrades, rollback, and recovery.
- Define retention and privacy expectations for student data and LLM inputs.

Exit criteria: deployment survives restarts, failures are visible, data can be
restored, and no unauthenticated service or Canvas credential is exposed.

## Cross-Cutting Testing

Each step should extend coverage for:

- Pagination, throttling, malformed responses, and partial failures.
- Multiple students sharing a course or assignment.
- Student-specific overrides and timezone boundaries.
- Deleted, unpublished, undated, and rescheduled work.
- Assignments without submissions.
- Front-page HTML variations and ambiguous natural-language dates.
- Consistent results across web, CLI, JSON API, and MCP.
- Migration and backup/restore behavior.

## Early Decisions to Revisit

- Whether scheduled sync runs inside the core service or as a separately invoked
  command. Keep a single database owner either way.
- Which LLM provider or local model to use after representative page samples are
  evaluated.
- Whether SQLite remains sufficient after real usage. PostgreSQL should only be
  introduced if multi-writer operation or deployment constraints warrant it.
- How much human review is necessary for low-confidence extracted events.
- Whether notifications or reminders belong in the first product milestone or a
  later phase.
