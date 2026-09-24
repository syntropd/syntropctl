# Architecture Specification: syntropctl

## 1. Subsystem Role and Purpose
`syntropctl` is the unified operator and diagnostic client for the `syntropd` suite.
Similar to how `systemctl` oversees standard systemd services, `syntropctl` serves
as the central interface across the six specialized system AI daemons:

| Daemon | Unit | Socket Interface | Subsystem Function |
| :--- | :--- | :--- | :--- |
| `sentry` | `sentry.service` | `io.syntrop.Sentry1` | Zero-trust supervisor, journal analysis, incident triage |
| `inferenced` | `inferenced.service` | `io.syntrop.Inference1` | Heterogeneous hardware arbiter, GPU/NPU device broker |
| `modeld` | `modeld.service` | `io.syntrop.Model1` | CAS layer storage, model image deduplication |
| `contextd` | `contextd.service` | `io.syntrop.Context1` | System chronology, /etc change tracking, state drift |
| `toold` | `toold.service` | `io.syntrop.Tool1` | Sandboxed action execution, cgroup/landlock isolation |
| `runtimed` | `runtimed.service` | `io.syntrop.Runtime1` | Headless model execution, token generation, embeddings |

## 2. IPC Protocol Architecture
All communication between `syntropctl` and daemons occurs over Unix domain sockets using
the Varlink protocol:
- Framing: Delimited with ASCII NUL (`\0`).
- Transport: Local Unix domain stream sockets (`/run/syntrop/*.sock`).
- Zero Dynamic C Dependencies: Pure Rust socket framing without `libsystemd.so` or `libdbus-1.so`.

## 3. Incident Fusion (`explain`)
The `explain` operation queries two daemons concurrently:
1. `sentry`: Retrieves the root cause diagnosis, failure confidence, and journal logs.
2. `contextd`: Retrieves recent configuration changes and drift correlated with the unit.
The two records are merged into a unified diagnostic report for operators.

## 4. Resource Budgets
- Execution Latency: < 5 ms for local status probing.
- Memory Footprint: < 10 MiB resident set size (RSS).
- Exit Codes: Standard Linux exit codes (0 on success, non-zero on failure).
