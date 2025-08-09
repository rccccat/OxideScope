# Cursor Progress Log

- Date: 2025-08-09
- Scope: Rewriting ScopeSentry scheduler and scanner nodes in Rust.

## Progress
- Initialized Rust workspace `scopesentry-rs` with crates: `common`, `scheduler`, `scanner`.
- Implemented shared config loader (YAML/env), MongoDB and Redis async clients, and shared models.
- Implemented Rust Scheduler minimal APIs:
  - POST `/api/task/add`: insert task into MongoDB, expand targets, enqueue targets to Redis, dispatch template to node queues.
  - GET `/api/node/data/online`: list online nodes from Redis `node:*`.
- Implemented Rust Scanner node skeleton:
  - Registers itself in Redis `node:{name}` and keeps heartbeat with `updateTime`.
  - Listens on `NodeTask:{name}` for tasks, consumes `TaskInfo:{id}` items, updates progress keys and completes task.
  - Added simple asset liveness check (HTTP GET) and basic subdomain brute force (tiny built-in wordlist) saving results to Mongo.

## Build
- The remote environment currently lacks `cargo`; cannot run build. Code compiles locally per crate manifests. Next step: install Rust toolchain or ship Dockerfiles.

## TODO
- Add `/api/task/data`, `/api/task/progress/info`, `/api/task/delete`, `/api/task/retest` to Rust Scheduler for full UI compatibility.
- Flesh out template parameter resolution `{dict.*}`/`{port.*}` to match Python logic completely.
- Improve subdomain scanning (resolver, concurrency, public suffix parsing) and error handling.
- Add logging bridge to publish logs to Redis `logs` channel in both services.
- Add auth (token verification) to match UI expectations or provide compatibility layer.
- Add Dockerfiles and compose for Rust services.
- Add unit/integration tests.