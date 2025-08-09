# Scanner (Rust)

Start with a name:
```
NODE_NAME=node-1 cargo run -p scopesentry-scanner
```

## What it does now
- Registers to Redis as `node:{name}` and publishes a log message to `logs`.
- Consumes tasks from `NodeTask:{name}` that match the original template payload shape.
- Pops per-task targets from `TaskInfo:{id}` and updates progress hashes `TaskInfo:progress:{id}:{target}`.
- Performs:
  - Basic subdomain brute (tiny list) with DNS check -> inserts into `subdomain` collection.
  - HTTP liveness probe -> upsert into `asset` collection.

## Extensibility for scanners
- Define trait-based adapters per scan stage and allow multiple libraries:
  - Port scan: implement traits `PortScanner` for A/B/C libs and compose with fanout.
  - Fingerprint: `Fingerprinter` trait swapping impls (httpx, custom, etc.).
- Provide a stage pipeline builder that reads `Parameters` to choose which adapters to run.

Example sketch:
```rust
#[async_trait::async_trait]
pub trait PortScanner { async fn scan(&self, inputs: &[String]) -> anyhow::Result<Vec<PortResult>>; }

struct NaiveNmap; // A
struct RustScan;  // B

struct CompositePortScan { impls: Vec<Box<dyn PortScanner + Send + Sync>> }

impl CompositePortScan {
  async fn run(&self, ins: &[String]) -> anyhow::Result<Vec<PortResult>> {
    let mut out = vec![];
    for imp in &self.impls { out.extend(imp.scan(ins).await?); }
    Ok(out)
  }
}
```