# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p domain
cargo test -p application

# Run a single test by name
cargo test -p domain <test_name>

# Check without building
cargo check

# Lint
cargo clippy

# Coverage (requires: cargo install cargo-llvm-cov)
cargo llvm-cov                        # summary in terminal
cargo llvm-cov --html                 # HTML report in target/llvm-cov/html/
cargo llvm-cov --lcov --output-path lcov.info  # LCOV for editor integration
```

## Architecture

This is a Cargo workspace (`task-manager`) studying Domain-Driven Design (DDD) patterns in Rust. Five crates:

- **`crates/domain`** — pure domain layer, no I/O or infrastructure dependencies
- **`crates/application`** — application layer; orchestrates domain objects, defines ports, implements use cases
- **`crates/infrastructure`** — adapters for the application's ports: `PgTaskRepository` (sqlx + Postgres) and `RedisPublisher` (redis pub/sub)
- **`crates/interface`** — interface layer; HTTP (Axum) and CLI (Clap) adapters that delegate to the application layer via `AppState`
- **`crates/server`** — binary entry point that wires everything together; supports `serve` (HTTP) and `cli` modes

Local Postgres + Redis are provided via `compose.yml` (`docker compose up -d`). Migrations live at the workspace root in `migrations/`.

### DDD patterns in use

**Aggregate (`task/aggregate.rs`)**: `Task` is the aggregate root. It enforces all business rules and owns the state machine. Two constructors exist for a reason:
- `Task::create(...)` — used when creating a new task; emits a `TaskCreated` domain event
- `Task::recover(...)` — used when reconstituting from persistence; emits no events

**Value objects (`task/value_objects.rs`)**: `TaskId`, `TaskTitle`, `TaskDescription`, `TaskStatus` are all newtypes. `TaskTitle` and `TaskDescription` are constructed via `TryFrom<&str>` which enforces invariants (non-empty title, max lengths: 255 / 2000 chars). They implement `Deref` to their inner type for ergonomic use.

**Domain events (`task/events.rs`)**: `TaskEvent` variants (`Created`, `StatusChanged`, `Updated`, `Deleted`) are accumulated in `Task::domain_events` (marked `#[serde(skip)]`). Call `task.extract_events()` to drain and take ownership of pending events. Events are never cleared automatically — the caller (application layer) is responsible for draining and dispatching them.

**Errors (`task/errors.rs`)**: `DomainError` uses `thiserror`. All domain rule violations return specific variants (e.g., `TaskAlreadyCanceled`, `TaskNotStarted`).

**Repository (`task/repository.rs`)**: `TaskRepository` is an async trait (via `async-trait`) defined in the domain layer. Methods: `save`, `find_by_id`, `find_all`, `update`, `delete_by_id`, `exists`.

### Task state machine

```
Todo ──start()──► InProgress ──complete()──► Completed ──reopen()──► Todo
 │                    │
 └───cancel()────────►┘──► Cancelled
```

`TaskStatus::is_completed()` returns `true` for both `Completed` and `Cancelled` (both are terminal for edits). Cancelled tasks cannot be reopened.

### Module visibility

`task/aggregate.rs`, `errors.rs`, `events.rs`, `value_objects.rs` are all private submodules of `task`. The domain crate re-exports: `Task`, `DomainError`, `TaskEvent`, `TaskRepository`, and all value objects (`TaskId`, `TaskTitle`, `TaskDescription`, `TaskStatus`). Keep domain internals encapsulated — application and infrastructure layers should only interact through the public API of `Task` and the repository trait.

### Application layer (`crates/application`)

**`UseCase` trait (`use_case.rs`)**: Generic async trait with associated `Input`/`Output` types. All command and query handlers implement it.

**`ApplicationError` (`error.rs`)**: Wraps `DomainError` (via `#[from]`), plus `NotFound(String)` and `Repository(String)` variants.

**Ports (`ports.rs`)**: `EventPublisher` — async trait for dispatching domain events after a use case completes.

**Commands (`commands/`)**: Each command is a plain DTO struct + a handler struct that holds `Arc<R: TaskRepository>` (and `Arc<P: EventPublisher>` where needed):
- `CreateTaskHandler` — validates input, calls `Task::create`, saves, publishes events; returns `TaskId`
- `ChangeStatusHandler` — loads task, dispatches `start`/`complete`/`cancel`/`reopen`, updates, publishes events
- `DeleteTaskHandler` — checks existence via `repo.exists`, then `repo.delete_by_id`

**Queries (`queries/`)**: Read-only handlers that return `TaskDto` or `Vec<TaskDto>`:
- `GetTaskByIdHandler` — fetches single task by `TaskId`, maps to `TaskDto`
- `ListTaskQueryHandler` — fetches all tasks, applies optional `status_filter`

**`TaskDto` (`queries/dto.rs`)**: Flat read model with `id`, `title`, `description`, `status`, `created_at`, `updated_at`. Converted from `&Task` via `From<&Task>`.

**Test mocks (`mocks.rs`, `#[cfg(test)]`)**: `MockTaskRepository` (in-memory `Mutex<Vec<Task>>`), `MockEventPublisher` (captures published events), `seed_task` helper, and `init_di` factory.

### Interface layer (`crates/interface`)

**`AppState` (`state.rs`)**: Holds type-erased `Arc<dyn UseCase<...>>` handles for all five use cases. Passed via Axum `State` extractor and the CLI runner. Uses type aliases (`DynCreateTask`, `DynChangeStatus`, etc.) for readability.

**HTTP adapter (`http/`)**: Built on Axum 0.8.
- `router.rs` — mounts routes under `/tasks` plus a `/health` check; applies `TraceLayer`
- `handlers.rs` — one handler per route; extracts `State`, `Path`, `Query`, or `Json`; returns `Result<_, ApiError>`
- `dto.rs` — `CreateTaskRequest` / `CreateTaskResponse` / `ListTasksRequest` (serde request/response bodies)
- `error.rs` — `ApiError` implements `IntoResponse`; maps `ApplicationError` and `DomainError` to HTTP status codes

HTTP routes:
| Method | Path | Handler |
|--------|------|---------|
| `POST` | `/tasks` | `create_task` → 201 + `{ id }` |
| `GET` | `/tasks` | `list_tasks` → 200 + `Vec<TaskDto>` (optional `?status=` filter) |
| `GET` | `/tasks/:id` | `get_task` → 200 + `TaskDto` |
| `PATCH` | `/tasks/:id/:action` | `update_task` → 204 |
| `DELETE` | `/tasks/:id` | `delete_task` → 204 |
| `GET` | `/health` | health check → "OK" |

**CLI adapter (`cli.rs`)**: Built on Clap 4.
- `Cli` / `Command` — subcommands: `create`, `list`, `show`, `update`, `delete`
- `run_command(cli, state)` — async dispatcher; returns `anyhow::Result<String>` for the caller to print
- `StatusActionCli` / `TaskStatusFilter` — `ValueEnum` wrappers that convert into domain types

**Integration tests (`tests/api.rs`, `tests/cli.rs`)**: Spin up an in-memory Axum router via `tower::ServiceExt::oneshot`; use `build_test_state()` from `tests/common` which wires mock repo + publisher into `AppState`. The HTTP router is re-exported as `interface::http::router` (the `http` submodule is otherwise private).

### Infrastructure layer (`crates/infrastructure`)

**`PgTaskRepository` (`pg_task_repository.rs`)**: Implements `TaskRepository` against Postgres via `sqlx`. Constructed synchronously from a `PgPool`; re-exports `sqlx::postgres::PgPoolOptions` for callers. Persistence errors are mapped to `DomainError::InfrastructureError`. The sqlx row/enum mapping lives in `maper.rs` (`TaskSqlx`, `TaskStatusSqlx`).

**`RedisPublisher` (`redis_publisher.rs`)**: Implements `EventPublisher` over a Redis pub/sub channel using `ConnectionManager` behind a `tokio::sync::Mutex`. `TaskEvent` is serialized as JSON via `serde_json`. Publish failures are mapped to `ApplicationError::Repository`.

### Server crate (`crates/server`)

`main.rs` is the only binary. It loads `.env` via `dotenvy`, initializes `tracing_subscriber` with `EnvFilter`, then dispatches a top-level Clap CLI with two modes:

- `serve --addr 0.0.0.0:8080` — binds an Axum listener, mounts `interface::http::router`, and shuts down on Ctrl+C / SIGTERM
- `cli <subcommand>` — delegates to `interface::cli::run_command` and prints the result

`build_app_state()` reads `DATABASE_URL`, `REDIS_URL`, and `EVENTS_CHANNEL` from the environment, opens a `PgPool` (max 10 connections) and a `RedisPublisher`, and assembles `AppState` with all five use case handlers.
