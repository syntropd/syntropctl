# syntropctl CLI Command Reference

## Global Options
- `--json`: Format all output as structured JSON.
- `-v, --verbose`: Enable debug trace logging to stderr.
- `-h, --help`: Display help information.
- `-V, --version`: Display version information.

## Subcommands

### 1. `status`
Inspect daemon socket connectivity, latency, and service responsiveness.
```bash
syntropctl status [DAEMON]
```

### 2. `explain`
Diagnose failure or state of a target systemd service unit.
```bash
syntropctl explain <UNIT>
```

### 3. `models`
List all models currently cached in storage or loaded in memory.
```bash
syntropctl models
```

### 4. `devices`
Inspect compute accelerators, VRAM allocation, and PSI pressure.
```bash
syntropctl devices
```

### 5. `drift`
Query recent configuration modifications and system chronology.
```bash
syntropctl drift [UNIT]
```

### 6. `run`
Execute an isolated tool or command within the toold sandbox.
```bash
syntropctl run [-p PROFILE] <TOOL> [ARGS...]
```

### 7. `generate`
Generate text completions from a prompt via runtimed.
```bash
syntropctl generate [-m MODEL] [-n MAX_TOKENS] [-t TEMPERATURE] <PROMPT>
```

### 8. `embed`
Compute semantic vector embeddings for input text.
```bash
syntropctl embed [-m MODEL] <TEXT>
```

### 9. `info`
Introspect Varlink interface definitions and vendor metadata.
```bash
syntropctl info [DAEMON]
```

### 10. `completions`
Generate shell auto-completion script for bash, zsh, or fish.
```bash
syntropctl completions <SHELL>
```
