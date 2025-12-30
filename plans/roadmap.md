# CGRB CLI + API Expansion Plan (Post-Toy)

## Overview

This plan expands the current toy implementation into a scalable CGRB system with dual frontends (CLI and API) over a unified command model. The key principle is that every CLI command has a direct API equivalent, both dispatching to the same core handlers.

## Current State Analysis

The existing toy system provides a foundation with:
- Unified `Command` enum and `execute()` function
- Toy CLI with clap-based subcommands
- Simple Axum-based API with single `/v1/command` endpoint
- Basic commands: ListInstances, RunInstance, Replay

## Expanded Architecture

### Core Command Model

Extend the `Command` enum to support full CGRB operations:

```rust
pub enum Command {
    // Existing toy commands
    ListInstances,
    RunInstance(RunInstanceArgs),
    Replay(ReplayArgs),

    // New CGRB commands
    Compile(CompileReq),
    Run(RunReq),
    Verify(VerifyReq),
    Hash(HashReq),
    Inspect(InspectReq),
    Generate(GenerateReq),
}
```

Each command has:
- Request struct (e.g., `CompileReq`)
- Response struct (e.g., `CompileRes`)
- No side effects beyond declared outputs

### CLI Surface

Expand CLI with new subcommands, maintaining consistent argument patterns:

```bash
# Compile
cgrb compile --instance instance.yaml --out compiled.cgrb --canonical cbor

# Run
cgrb run --compiled compiled.cgrb --solver reference --level 2 --out run.json

# Verify
cgrb verify --output run.json --level 2 --strict

# Replay
cgrb replay --compiled compiled.cgrb --run run.json --out replay.json

# Hash
cgrb hash --compiled compiled.cgrb

# Inspect
cgrb inspect instance.yaml
cgrb inspect compiled.cgrb
cgrb inspect run.json

# Generate
cgrb generate --family geometry_gated --seed 42 --out instance.yaml
```

### API Surface

Shift from single command endpoint to command-specific endpoints:

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET    | /capabilities | Get supported features and versions |
| POST   | /compile | Compile instance |
| POST   | /run | Execute solver |
| POST   | /verify | Verify run output |
| POST   | /replay | Deterministic replay |
| POST   | /hash | Compute canonical hash |
| POST   | /inspect | Analyze artifact |
| POST   | /generate | Generate benchmark |

Example request:

```http
POST /run
Content-Type: application/json

{
  "compiled_id": "abc123",
  "solver": "reference",
  "level": 2
}
```

### Solver Integration

Support multiple solver types:

1. **In-process plugins**: Rust traits implementing `Solver` trait
2. **External executables**: Stdio contract with CBOR/JSON I/O

```rust
pub trait Solver {
    fn name(&self) -> &'static str;
    fn supported_level(&self) -> u8;
    fn run(&self, compiled: &CompiledInstance, cfg: RunConfig) -> Result<RunCapsule>;
}
```

### Hardware Backend Abstraction

```rust
pub trait Backend {
    fn enqueue_event(&mut self, e: Event) -> Result<()>;
    fn step(&mut self) -> Result<StepResult>;
    fn snapshot(&self) -> Result<Snapshot>;
}
```

Implementations for software reference, simulators, and hardware.

### Versioning and Compatibility

- Kernel spec versioned separately (currently v0.1)
- RunCapsule includes version metadata
- Capability negotiation via `/capabilities` endpoint
- Verifier rejects incompatible versions

```json
{
  "kernel_version": "0.1",
  "trace_versions": ["1"],
  "supported_solvers": ["reference", "hw_emulator"],
  "max_arity": 8
}
```

## Implementation Roadmap

1. Extend Command enum with new variants
2. Define request/response structs for all commands
3. Update CLI with new subcommands
4. Refactor API to multiple endpoints
5. Implement solver plugin system
6. Add backend abstraction layer
7. Integrate verification and replay
8. Add comprehensive error handling
9. Implement capability negotiation
10. Create integration tests

## Migration Path

- Preserve existing toy commands during transition
- Add new commands incrementally
- Maintain backwards compatibility in API
- Update documentation and examples
