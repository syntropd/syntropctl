# syntropctl

Unified administration and diagnostic CLI for the **syntropd** AI subsystem suite.

Part of the [syntropd](https://github.com/syntropd) Linux AI systemd initiative.

## Subsystem Architecture

`syntropctl` coordinates and inspects the 6 specialized daemons in the syntropd suite:

- **sentry**: Autonomous supervisor, journal slicing, and failure triage (`io.syntrop.Sentry1`).
- **inferenced**: Heterogeneous GPU/NPU/AMX hardware arbiter and lease broker (`io.syntrop.Inference1`).
- **modeld**: Content-addressable storage (CAS) model layer cache (`io.syntrop.Model1`).
- **contextd**: System chronology and configuration drift tracker (`io.syntrop.Context1`).
- **toold**: Sandboxed command and tool execution agent (`io.syntrop.Tool1`).
- **runtimed**: Headless neural model execution and embedding engine (`io.syntrop.Runtime1`).

## Core Features

- **Fleet Health Matrix**: `syntropctl status` queries all 6 daemons and inspects socket connectivity, latency, and Varlink responsiveness.
- **Incident Explanation**: `syntropctl explain <unit>` merges sentry failure diagnostics with contextd configuration drift.
- **Model Catalog**: `syntropctl models` combines CAS disk inventory with loaded memory footprints.
- **Accelerator Inspection**: `syntropctl devices` reports GPU/NPU memory allocations and PSI memory pressure.
- **Sandboxed Execution**: `syntropctl run <tool> [args...]` executes commands within toold's isolated cgroup and landlock sandbox.
- **Zero Dynamic C Dependencies**: Pure Rust implementation linking only libc and libm.

## Building and Testing

```bash
# Build binary
cargo build --release

# Run unit and edge QA suites
cargo test --workspace
```

## License

Apache-2.0
