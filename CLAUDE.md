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

This is a Cargo workspace (`task-manager`) studying Domain-Driven Design (DDD) patterns in Rust. Two crates:

- **`crates/domain`** — pure domain layer, no I/O or infrastructure dependencies
- **`crates/application`** — application layer; orchestrates domain objects, defines ports, implements use cases

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
