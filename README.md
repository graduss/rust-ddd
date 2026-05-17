# rust-ddd

A study project exploring Domain-Driven Design (DDD) patterns in Rust, built around a task management domain.

## Crates

| Crate | Description |
|-------|-------------|
| `domain` | Pure domain layer — aggregates, value objects, domain events, repository traits |
| `application` | Application layer — use cases, commands, queries, ports |

## Getting started

```bash
cargo build
cargo test
```

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
