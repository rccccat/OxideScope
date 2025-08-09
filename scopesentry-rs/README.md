# ScopeSentry Rust Nodes

This workspace provides a Rust rewrite of the Scheduler and Scanner nodes for ScopeSentry.

- scheduler: HTTP server exposing minimal APIs to accept tasks, store in MongoDB, and dispatch to Redis.
- scanner: Worker node that registers to Redis, consumes node tasks, executes basic subdomain scan and asset liveness, and stores results to MongoDB.

## Config

Both services read YAML configuration compatible with the original Python app. By default it loads from `../ScopeSentry/config.yaml`.

Override via environment variable:

```
SCOPESENTRY_CONFIG=/absolute/path/to/config.yaml
```

## Run

- Scheduler:
```
cargo run -p scopesentry-scheduler
```
- Scanner:
```
NODE_NAME=node-1 cargo run -p scopesentry-scanner
```

Ensure MongoDB and Redis are reachable as configured.