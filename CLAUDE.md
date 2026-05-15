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

This is a Cargo workspace (`task-manager`) studying Domain-Driven Design (DDD) patterns in Rust. Currently one crate:

- **`crates/domain`** — pure domain layer, no I/O or infrastructure dependencies

### DDD patterns in use

**Aggregate (`task/aggregate.rs`)**: `Task` is the aggregate root. It enforces all business rules and owns the state machine. Two constructors exist for a reason:
- `Task::create(...)` — used when creating a new task; emits a `TaskCreated` domain event
- `Task::recover(...)` — used when reconstituting from persistence; emits no events

**Value objects (`task/value_objects.rs`)**: `TaskId`, `TaskTitle`, `TaskDescription`, `TaskStatus` are all newtypes. `TaskTitle` and `TaskDescription` are constructed via `TryFrom<&str>` which enforces invariants (non-empty title, max lengths: 255 / 2000 chars). They implement `Deref` to their inner type for ergonomic use.

**Domain events (`task/events.rs`)**: `TaskEvent` variants (`Created`, `StatusChanged`, `Updated`, `Deleted`) are accumulated in `Task::domain_events` (marked `#[serde(skip)]`). Call `task.pending_events()` to read them. Events are never cleared automatically — the caller (application layer) is responsible for draining and dispatching them.

**Errors (`task/errors.rs`)**: `DomainError` uses `thiserror`. All domain rule violations return specific variants (e.g., `TaskAlreadyCanceled`, `TaskNotStarted`).

**Repository (`task/repository.rs`)**: Currently a stub — the trait will go here.

### Task state machine

```
Todo ──start()──► InProgress ──complete()──► Completed ──reopen()──► Todo
 │                    │
 └───cancel()────────►┘──► Cancelled
```

`TaskStatus::is_completed()` returns `true` for both `Completed` and `Cancelled` (both are terminal for edits). Cancelled tasks cannot be reopened.

### Module visibility

`task/aggregate.rs`, `errors.rs`, `events.rs`, `value_objects.rs` are all private submodules of `task`. Only `Task` is re-exported from `crates/domain/src/task.rs`. Keep domain internals encapsulated — application and infrastructure layers should only interact through the public API of `Task` and the repository trait.
