# rust-ddd

A study project exploring Domain-Driven Design (DDD) patterns in Rust, built around a task management domain.

## Crates

| Crate | Description |
|-------|-------------|
| `domain` | Pure domain layer — aggregates, value objects, domain events, repository traits |
| `application` | Application layer — use cases, commands, queries, ports |
| `infrastructure` | Port adapters — `PgTaskRepository` (sqlx/Postgres) and `RedisPublisher` (Redis pub/sub) |
| `interface` | Interface layer — HTTP (Axum) and CLI (Clap) adapters |
| `server` | Binary entry point — wires everything together and exposes `serve` / `cli` modes |

## Getting started

```bash
cargo build
cargo test
```

### Running the server locally

Start Postgres and Redis via Docker Compose, then point the server at them:

```bash
docker compose up -d

export DATABASE_URL=postgres://tasks:tasks@localhost:5432/tasks
export REDIS_URL=redis://localhost:6379
export EVENTS_CHANNEL=task.events

cargo run -p server -- serve --addr 0.0.0.0:8080
# or run a one-off CLI command against the same backend:
cargo run -p server -- cli list
```

Migrations live in `migrations/` at the workspace root and are applied via `sqlx` (e.g. `sqlx migrate run`). A `.env` file is loaded automatically by the server binary.

## Tools

| Tool | Command |
|------|---------|
| Lint | `cargo clippy` |
| Format | `cargo fmt` |
| Coverage | `cargo llvm-cov --html` |

> `cargo-llvm-cov` must be installed once: `cargo install cargo-llvm-cov`

## Domain model

The `Task` aggregate enforces a strict state machine:

```
Todo ──start()──► InProgress ──complete()──► Completed
 │                    │                          │
 └────────────────────┴──cancel()──► Cancelled   └──reopen()──► Todo
```

Both `Completed` and `Cancelled` are terminal states for edits. Cancelled tasks cannot be reopened.

Value objects (`TaskTitle`, `TaskDescription`) are validated at construction via `TryFrom<&str>` and carry their invariants into the type system.

Domain events are accumulated inside the aggregate and drained via `task.extract_events()`. The application layer is responsible for dispatching them after each use case completes.

## Application layer

Use cases implement the `UseCase` trait (`async fn execute(input) -> Result<Output, ApplicationError>`). Handlers are generic over `TaskRepository` and optionally `EventPublisher`.

**Commands**

| Handler | Input | Output |
|---------|-------|--------|
| `CreateTaskHandler` | `CreateTaskCommand { title, description }` | `TaskId` |
| `ChangeStatusHandler` | `ChangeStatusCommand { task_id, action }` | `()` |
| `DeleteTaskHandler` | `DeleteTaskCommand { task_id }` | `()` |

`StatusAction` variants: `Start`, `Complete`, `Cancel`, `Reopen`.

**Queries**

| Handler | Input | Output |
|---------|-------|--------|
| `GetTaskByIdHandler` | `GetTaskByIdQuery { id }` | `TaskDto` |
| `ListTaskQueryHandler` | `ListTaskQuery { status_filter }` | `Vec<TaskDto>` |

## Interface layer

Adapters wire `AppState` (holding type-erased `Arc<dyn UseCase<...>>` handles) to two delivery mechanisms:

**HTTP API** (Axum)

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/tasks` | Create a task |
| `GET` | `/tasks` | List tasks (optional `?status=` filter) |
| `GET` | `/tasks/:id` | Get a task by id |
| `PATCH` | `/tasks/:id/:action` | Change task status (`start`/`complete`/`cancel`/`reopen`) |
| `DELETE` | `/tasks/:id` | Delete a task |
| `GET` | `/health` | Health check |

**CLI** (Clap)

```
task create --title <TITLE> [--description <DESC>]
task list [<STATUS>]
task show <ID>
task update <ID> <ACTION>
task delete <ID>
```
