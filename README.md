# rust-ddd

A study project exploring Domain-Driven Design (DDD) patterns in Rust, built around a task management domain.

## Crates

| Crate | Description |
|-------|-------------|
| `domain` | Pure domain layer — aggregates, value objects, domain events, repository traits |

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

Domain events are accumulated inside the aggregate and available via `task.pending_events()`. The application layer is responsible for draining and dispatching them.
