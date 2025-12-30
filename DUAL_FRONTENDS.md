# Dual Front-Ends

The MHG engine provides two complete front-ends that expose identical functionality through different interfaces: a full-featured CLI and a REST API. Both call the same underlying `toy_core::execute()` function, ensuring consistent behavior across all invocation methods.

## 💻 CLI Interface (`toy-cli`)

### Overview

The CLI provides a complete command-line interface using clap for argument parsing and subcommand dispatch. It supports all engine commands with convenient command-line options.

### CLI Structure

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "toy")]
#[command(about = "Toy hypergraph-manifold runner")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List {},
    Run { /* ... */ },
    Replay { /* ... */ },
    Serve { /* ... */ },
}
```

### Command Mapping

Each CLI subcommand maps directly to a `toy_core::Command` variant:

| CLI Command | Core Command | Description |
|-------------|--------------|-------------|
| `toy list` | `ListInstances` | Discover available instances |
| `toy run` | `RunInstance` | Execute instance with options |
| `toy replay` | `Replay` | Deterministic replay |
| `toy serve` | N/A | Start HTTP API server |

### CLI Usage Examples

#### List Available Instances

```bash
# List all toy instances
toy list

# Output:
# {
#   "instances": ["triangle_attractor_v0"]
# }
```

#### Run Instance with Default Settings

```bash
# Run triangle attractor with software ideal backend
toy run --instance triangle_attractor_v0

# Same as explicit backend specification
toy run --instance triangle_attractor_v0 --backend sw
```

#### Run with Custom Ticks and Tracing

```bash
# Run for specific number of ticks with full tracing
toy run --instance triangle_attractor_v0 --ticks 100 --trace --backend hw
```

#### Backend Selection

```bash
# Software ideal (reference semantics)
toy run --instance triangle_attractor_v0 --backend sw

# Toy hardware (fixed-point, memory counters)
toy run --instance triangle_attractor_v0 --backend hw

# GESC fabric (64-bit events, credit control)
toy run --instance triangle_attractor_v0 --backend gesc-fabric

# GESC barrier (two-phase execution)
toy run --instance triangle_attractor_v0 --backend gesc-barrier

# GESA loci (adaptive field/constraint loci)
toy run --instance triangle_attractor_v0 --backend gesa-loci

# Multi-lane execution (parallel with conflicts)
toy run --instance triangle_attractor_v0 --backend multi-lane
```

#### Replay from Snapshot

```bash
# Replay deterministic execution
toy replay --snapshot path/to/snapshot.json
```

#### Start HTTP API Server

```bash
# Start REST API server on default port
toy serve

# Start on custom port
toy serve --addr 127.0.0.1:9000
```

### CLI Implementation Details

#### Backend String Parsing

The CLI includes user-friendly backend name parsing:

```rust
let backend_kind = match backend.as_str() {
    "sw" | "software" => BackendKind::SoftwareIdeal,
    "hw" | "hardware" => BackendKind::ToyHardware,
    "gesc-fabric" => BackendKind::ToyGescFabric,
    "gesc-barrier" => BackendKind::ToyGescBarrier,
    "gesa-loci" => BackendKind::ToyGesaLoci,
    "multi-lane" => BackendKind::ToyMultiLane,
    _ => {
        eprintln!("Invalid backend: {}. Use 'sw', 'hw', 'gesc-fabric', 'gesc-barrier', 'gesa-loci', or 'multi-lane'", backend);
        std::process::exit(1);
    }
};
```

#### Command Execution

All CLI commands follow the same pattern:

```rust
match cli.command {
    Commands::List {} => {
        let result = execute(Command::ListInstances);
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    }
    Commands::Run { instance, ticks, trace, backend } => {
        let backend_kind = parse_backend(&backend);
        let result = execute(Command::RunInstance(RunInstanceArgs {
            instance, ticks, trace, backend: backend_kind,
        }));
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    }
    // ... other commands
}
```

#### Error Handling

The CLI provides clear error messages for common issues:

- Invalid backend names with usage hints
- Unknown instance names
- JSON serialization errors (rare)

### CLI Testing

```rust
#[test]
fn test_cli_backend_parsing() {
    assert_eq!(parse_backend("sw"), BackendKind::SoftwareIdeal);
    assert_eq!(parse_backend("hardware"), BackendKind::ToyHardware);
    assert_eq!(parse_backend("gesc-fabric"), BackendKind::ToyGescFabric);
}

#[test]
fn test_cli_command_consistency() {
    // CLI and direct execution produce identical results
    let cli_result = run_cli_command("run --instance triangle_attractor_v0");
    let direct_result = execute(Command::RunInstance(default_args()));

    assert_eq!(cli_result.data, direct_result.data);
}
```

## 🌐 REST API Interface (`toy-api`)

### Overview

The REST API provides HTTP access to all engine functionality using axum. It exposes a single endpoint that accepts JSON commands and returns JSON results.

### API Design

#### Single Endpoint Architecture

```rust
// axum router with single command endpoint
let app = Router::new().route("/v1/command", post(handle_command));

async fn handle_command(Json(cmd): Json<Command>) -> Json<CommandResult> {
    let result = execute(cmd);
    Json(result)
}
```

#### Why Single Endpoint?

- **Simplicity**: One endpoint to maintain and document
- **Type Safety**: JSON schema matches Rust types exactly
- **Consistency**: Same command structure as CLI and programmatic use
- **Versioning**: Clear `/v1/` versioning strategy

### API Usage Examples

#### List Instances

```bash
curl -X POST http://localhost:8080/v1/command \
  -H "Content-Type: application/json" \
  -d '{"ListInstances": {}}'
```

Response:
```json
{
  "ok": true,
  "message": "instances",
  "data": {
    "instances": ["triangle_attractor_v0"]
  }
}
```

#### Run Instance

```bash
curl -X POST http://localhost:8080/v1/command \
  -H "Content-Type: application/json" \
  -d '{
    "RunInstance": {
      "instance": "triangle_attractor_v0",
      "ticks": null,
      "trace": false,
      "backend": "ToyHardware"
    }
  }'
```

Response:
```json
{
  "ok": true,
  "message": "run_complete",
  "data": {
    "meta": {
      "instance_id": "triangle_attractor_v0",
      "solver_id": "mhg-toy-0.1.0",
      "determinism_level": "LocalBitwise",
      "ticks_requested": 10,
      "ticks_executed": 10,
      "started_ns": 0,
      "duration_ns": 1500000,
      "nodes": 3,
      "edges": 2
    },
    "counts": {
      "ticks": 10,
      "edges_fired": 15,
      "nodes_moved": 10,
      "gate_eval_calls": 15,
      "hw_counts": {
        "cycles": 150,
        "node_mem_reads": 45,
        "node_mem_writes": 30,
        "edge_mem_reads": 15,
        "event_q_pushes": 15,
        "event_q_pops": 15,
        "credit_consumes": 15,
        "credit_releases": 15,
        "lane_utilization": [75, 75],
        "node_write_conflicts": 0,
        "edge_read_conflicts": 2
      }
    },
    "properties": [
      {
        "name": "head_converged",
        "passed": true,
        "details": "Triangle attractor converged to stable point"
      }
    ]
  }
}
```

#### Replay Execution

```bash
curl -X POST http://localhost:8080/v1/command \
  -H "Content-Type: application/json" \
  -d '{
    "Replay": {
      "snapshot_path": "/path/to/snapshot.json"
    }
  }'
```

### API Server Configuration

#### Default Configuration

```rust
pub async fn serve(addr: String) -> anyhow::Result<()> {
    let app = Router::new().route("/v1/command", post(handle_command));

    println!("Toy API server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

#### Middleware and Extensions

Future extensions may include:
- Request logging middleware
- Authentication/authorization
- Rate limiting
- Request/response compression
- CORS headers for web clients

### API Testing

#### Integration Tests

```rust
#[tokio::test]
async fn test_api_command_execution() {
    // Start test server
    let addr = "127.0.0.1:0"; // Random port
    let server = tokio::spawn(async move {
        serve(addr.to_string()).await
    });

    // Wait for server to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Send command
    let client = reqwest::Client::new();
    let response = client
        .post("http://127.0.0.1:8080/v1/command")
        .json(&Command::ListInstances)
        .send()
        .await
        .unwrap();

    let result: CommandResult = response.json().await.unwrap();
    assert!(result.ok);
    assert_eq!(result.message, "instances");

    server.abort();
}
```

#### Load Testing

```rust
#[tokio::test]
async fn test_api_concurrent_requests() {
    // Start server
    let addr = "127.0.0.1:0";
    let server = tokio::spawn(serve(addr.to_string()));

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Send multiple concurrent requests
    let mut handles = vec![];
    for _ in 0..10 {
        let handle = tokio::spawn(async {
            reqwest::Client::new()
                .post("http://127.0.0.1:8080/v1/command")
                .json(&Command::ListInstances)
                .send()
                .await
                .unwrap()
                .json::<CommandResult>()
                .await
                .unwrap()
        });
        handles.push(handle);
    }

    // All should succeed
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.ok);
    }

    server.abort();
}
```

## 🔄 Unified Interface Benefits

### Identical Behavior

All three invocation methods (CLI, REST API, programmatic) produce identical results:

```rust
// CLI execution
toy run --instance triangle_attractor_v0 --backend hw

// REST API execution
POST /v1/command {"RunInstance": {"instance": "triangle_attractor_v0", "backend": "ToyHardware"}}

// Programmatic execution
execute(Command::RunInstance(RunInstanceArgs {
    instance: "triangle_attractor_v0".to_string(),
    backend: BackendKind::ToyHardware,
    ticks: None,
    trace: false,
}));
```

All three produce the same `RunSummary` with identical data.

### Consistent Error Handling

All interfaces use the same `CommandResult` structure:

```rust
CommandResult {
    ok: bool,           // Success/failure flag
    message: String,    // Human-readable status
    data: Value,        // Structured result data
}
```

### Type Safety

The command structure is defined once and used everywhere:

- **CLI**: Clap derives command structure from Rust types
- **API**: JSON schema matches Rust `Command` enum exactly
- **Programmatic**: Direct use of Rust types

### Testing Consistency

```rust
#[test]
fn test_interface_consistency() {
    let cmd = Command::RunInstance(RunInstanceArgs { /* ... */ });

    // Test all three paths produce identical results
    let cli_result = execute_via_cli(&cmd);
    let api_result = execute_via_api(&cmd).await;
    let direct_result = execute(cmd);

    assert_eq!(cli_result.data, api_result.data);
    assert_eq!(api_result.data, direct_result.data);
}
```

## 🚀 Extension Patterns

### Adding New CLI Commands

1. Add variant to `Commands` enum
2. Add clap argument parsing
3. Map to `toy_core::Command` variant
4. Handle result formatting

### Adding New API Features

1. Extend `Command` enum in `toy_core`
2. API automatically supports new commands (single endpoint)
3. Add request/response examples to documentation
4. Update OpenAPI specification (future)

### Frontend-Specific Features

- **CLI**: Rich terminal output, progress bars, interactive mode
- **API**: Request batching, streaming responses, webhooks
- **Programmatic**: Direct access to intermediate results, custom backends

## 📋 Frontend Architecture Benefits

### Developer Experience

- **Single Implementation**: Command logic implemented once in `toy_core`
- **Type Safety**: Rust types ensure correctness across all interfaces
- **Documentation**: Self-documenting command structure
- **Testing**: Test commands independently of presentation layer

### User Experience

- **Choice of Interface**: Users can choose CLI, API, or programmatic access
- **Consistent Results**: Same execution semantics regardless of interface
- **Rich Output**: JSON results enable integration with other tools
- **Error Handling**: Clear error messages and structured error data

### Maintenance Benefits

- **DRY Principle**: No duplicated command logic
- **Easier Testing**: Test command execution separately from UI concerns
- **Easier Extension**: Add new commands in one place
- **Version Compatibility**: Interface changes tracked through command versioning

## 🔧 Advanced Usage

### CLI Scripting

```bash
# Chain commands in scripts
for instance in $(toy list | jq -r '.data.instances[]'); do
    echo "Running $instance..."
    toy run --instance "$instance" --backend hw
done
```

### API Integration

```python
import requests

# Python integration
response = requests.post('http://localhost:8080/v1/command', json={
    'RunInstance': {
        'instance': 'triangle_attractor_v0',
        'backend': 'ToyHardware',
        'ticks': 50,
        'trace': True
    }
})

result = response.json()
print(f"Execution took {result['data']['meta']['duration_ns']} ns")
```

### Programmatic Usage

```rust
use toy_core::{execute, Command, RunInstanceArgs, BackendKind};

fn run_simulation(instance: &str, backend: BackendKind) {
    let cmd = Command::RunInstance(RunInstanceArgs {
        instance: instance.to_string(),
        ticks: Some(100),
        trace: false,
        backend,
    });

    let result = execute(cmd);
    if result.ok {
        println!("Simulation completed successfully");
        // Process result.data...
    } else {
        eprintln!("Simulation failed: {}", result.message);
    }
}