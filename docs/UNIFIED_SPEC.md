# syntropd Suite Unified IPC Specification

## 1. System Socket Paths
All syntropd daemons listen on dedicated Varlink Unix domain sockets located in `/run/syntrop/`:

- `/run/syntrop/io.syntrop.Sentry1`
- `/run/syntrop/io.syntrop.Inference1`
- `/run/syntrop/io.syntrop.Model1`
- `/run/syntrop/io.syntrop.Context1`
- `/run/syntrop/io.syntrop.Tool1`
- `/run/syntrop/io.syntrop.Runtime1`

## 2. Environment Overrides
To facilitate isolated testing and containerized deployments, socket locations
may be overridden via environment variables:

| Daemon | Environment Variable Override |
| :--- | :--- |
| sentry | `SYNTROP_SENTRY_SOCKET` |
| inferenced | `SYNTROP_INFERENCE_SOCKET` |
| modeld | `SYNTROP_MODELD_SOCKET` |
| contextd | `SYNTROP_CONTEXTD_SOCKET` |
| toold | `SYNTROP_TOOLD_SOCKET` |
| runtimed | `SYNTROP_RUNTIMED_SOCKET` |

## 3. Varlink Introspection Standard
Each daemon must implement the standard `org.varlink.service` interface:
```varlink
interface org.varlink.service

method GetInfo() -> (
  vendor: string,
  product: string,
  version: string,
  url: string,
  interfaces: []string
)

method GetInterfaceDescription(interface: string) -> (description: string)
```
