# Usage Guide

This comprehensive guide provides practical examples and usage patterns for all MHG engine components, from basic execution to advanced hardware exploration.

## 🚀 Quick Start

### Installation and Setup

```bash
# Clone the repository
git clone <repository-url>
cd mhg-engine

# Build all components
cargo build --release

# Run basic tests
cargo test
```

### First Execution

```bash
# List available toy instances
cargo run --bin toy-cli -- list

# Run triangle attractor simulation
cargo run --bin toy-cli -- run --instance triangle_attractor_v0

# Output:
# {
#   "ok": true,
#   "message": "run_complete",
#   "data": {
#     "meta": {
#       "instance_id": "triangle_attractor_v0",
#       "ticks_executed": 10,
#       "nodes": 3,
#       "edges": 2
#     },
#     "counts": {
#       "edges_fired": 15,
#       "nodes_moved": 10
#     }
#   }
# }
```

## 💻 CLI Usage Patterns

### Backend Selection and Comparison

```bash
# Compare all backends on same instance
for backend in "sw" "hw" "gesc-fabric" "gesc-barrier" "gesa-loci" "multi-lane"; do
    echo "=== $backend ==="
    toy run --instance triangle_attractor_v0 --backend $backend --ticks 20
done
```

### Performance Analysis

```bash
# Run with timing and detailed metrics
toy run --instance triangle_attractor_v0 --backend hw --trace

# Extract specific metrics
toy run --instance triangle_attractor_v0 --backend hw | jq '.data.counts.hw_counts'
```

### Custom Tick Counts

```bash
# Short simulation
toy run --instance triangle_attractor_v0 --ticks 5

# Long simulation with convergence analysis
toy run --instance triangle_attractor_v0 --ticks 100 --backend sw
```

### Batch Processing

```bash
# Process multiple instances
for instance in $(toy list | jq -r '.data.instances[]'); do
    echo "Processing $instance..."
    toy run --instance "$instance" --backend hw > "results_$instance.json"
done
```

## 🌐 REST API Usage

### Server Startup

```bash
# Start API server on default port (8080)
toy-api

# Start on custom port
toy-api --addr 127.0.0.1:9000
```

### Basic API Calls

```bash
# List instances
curl -X POST http://localhost:8080/v1/command \
  -H "Content-Type: application/json" \
  -d '{"ListInstances": {}}'

# Run simulation
curl -X POST http://localhost:8080/v1/command \
  -H "Content-Type: application/json" \
  -d '{
    "RunInstance": {
      "instance": "triangle_attractor_v0",
      "backend": "ToyHardware",
      "ticks": 50,
      "trace": true
    }
  }'
```

### Integration with Other Tools

#### Python Integration

```python
import requests
import json

class MHGClient:
    def __init__(self, base_url="http://localhost:8080"):
        self.base_url = base_url

    def list_instances(self):
        response = requests.post(f"{self.base_url}/v1/command",
                               json={"ListInstances": {}})
        return response.json()

    def run_simulation(self, instance, backend="ToyHardware", ticks=10):
        payload = {
            "RunInstance": {
                "instance": instance,
                "backend": backend,
                "ticks": ticks,
                "trace": False
            }
        }
        response = requests.post(f"{self.base_url}/v1/command", json=payload)
        return response.json()

# Usage
client = MHGClient()
result = client.run_simulation("triangle_attractor_v0", ticks=20)
print(f"Simulation took {result['data']['meta']['duration_ns']} ns")
```

#### JavaScript/Node.js Integration

```javascript
const axios = require('axios');

class MHGAPI {
    constructor(baseURL = 'http://localhost:8080') {
        this.client = axios.create({
            baseURL,
            headers: { 'Content-Type': 'application/json' }
        });
    }

    async listInstances() {
        const response = await this.client.post('/v1/command', { ListInstances: {} });
        return response.data;
    }

    async runInstance(instance, options = {}) {
        const payload = {
            RunInstance: {
                instance,
                backend: options.backend || 'ToyHardware',
                ticks: options.ticks || 10,
                trace: options.trace || false
            }
        };
        const response = await this.client.post('/v1/command', payload);
        return response.data;
    }
}

// Usage
const api = new MHGAPI();
const result = await api.runInstance('triangle_attractor_v0', { ticks: 30 });
console.log(`Edges fired: ${result.data.counts.edges_fired}`);
```

## 🔬 Backend-Specific Usage

### Software Ideal Backend

**Use Case**: Reference behavior, algorithmic validation

```bash
# Maximum precision reference
toy run --instance triangle_attractor_v0 --backend sw --ticks 100

# Performance baseline
time toy run --instance triangle_attractor_v0 --backend sw
```

### Toy Hardware Backend

**Use Case**: Embedded systems, fixed-point analysis

```bash
# Analyze memory access patterns
toy run --instance triangle_attractor_v0 --backend hw | jq '.data.counts.hw_counts'

# Fixed-point precision study
for ticks in 10 50 100; do
    echo "Ticks: $ticks"
    toy run --instance triangle_attractor_v0 --backend hw --ticks $ticks \
      | jq '.data.counts.hw_counts.cycles'
done
```

### GESC Fabric Backend

**Use Case**: Event-driven architectures, communication analysis

```bash
# Event throughput analysis
toy run --instance triangle_attractor_v0 --backend gesc-fabric | jq '{
    events_processed: .data.counts.hw_counts.event_q_pops,
    credits_used: .data.counts.hw_counts.credit_consumes,
    queue_overflows: .data.counts.hw_counts.event_q_overflows
}'
```

### GESC Barrier Backend

**Use Case**: Consistency models, structural changes

```bash
# Barrier frequency analysis
toy run --instance triangle_attractor_v0 --backend gesc-barrier --ticks 50 | jq '{
    total_cycles: .data.counts.hw_counts.cycles,
    pending_topology: .data.counts.hw_counts.pending_topo_events
}'
```

### GESA Loci Backend

**Use Case**: Adaptive systems, intelligent behavior

```bash
# Locus activation study
toy run --instance triangle_attractor_v0 --backend gesa-loci --trace
# Analyze trace for locus behavior patterns
```

### Multi-Lane Backend

**Use Case**: Parallel processing, scalability analysis

```bash
# Parallel efficiency analysis
toy run --instance triangle_attractor_v0 --backend multi-lane | jq '{
    lane0_cycles: .data.counts.hw_counts.lane_utilization[0],
    lane1_cycles: .data.counts.hw_counts.lane_utilization[1],
    write_conflicts: .data.counts.hw_counts.node_write_conflicts,
    read_conflicts: .data.counts.hw_counts.edge_read_conflicts
}'
```

## 📊 Analysis and Visualization

### Performance Comparison Script

```bash
#!/bin/bash
# compare_backends.sh

INSTANCE="triangle_attractor_v0"
TICKS=20

echo "Backend Comparison for $INSTANCE ($TICKS ticks)"
echo "================================================="

for backend in sw hw gesc-fabric gesc-barrier gesa-loci multi-lane; do
    echo -n "$backend: "
    result=$(toy run --instance $INSTANCE --backend $backend --ticks $TICKS)

    # Extract key metrics
    cycles=$(echo $result | jq '.data.counts.hw_counts.cycles // 0')
    mem_reads=$(echo $result | jq '.data.counts.hw_counts.node_mem_reads // 0')
    events=$(echo $result | jq '.data.counts.hw_counts.event_q_pops // 0')

    echo "cycles=$cycles, mem_reads=$mem_reads, events=$events"
done
```

### Convergence Analysis

```python
import json
import matplotlib.pyplot as plt

def analyze_convergence(instance_id, max_ticks=100):
    """Analyze how quickly different backends converge"""

    results = {}
    for backend in ['sw', 'hw', 'gesc-fabric']:
        # Run simulation
        # (implementation would call API)
        pass

    # Plot convergence curves
    # Compare final positions vs. tick count

def plot_memory_usage():
    """Plot memory access patterns"""

    # Collect data from multiple runs
    # Plot reads/writes over time
    # Compare across backends
    pass
```

### Scaling Analysis

```bash
#!/bin/bash
# scaling_study.sh

echo "Scaling Analysis"
echo "================"

for size in 8 16 32; do
    echo "Size $size:"

    # Generate or select appropriately sized instance
    instance="synthetic_size_${size}"

    # Run on different backends
    for backend in sw hw; do
        result=$(toy run --instance $instance --backend $backend)
        cycles=$(echo $result | jq '.data.counts.hw_counts.cycles // 0')
        echo "  $backend: $cycles cycles"
    done
done
```

## 🧪 Testing and Validation

### Running the Test Suite

```bash
# Run all tests
cargo test

# Run specific test categories
cargo test demo      # Demo-style tests
cargo test causality # Causality validation
cargo test freeze    # Graph freezing tests

# Run performance regression tests
cargo test perf --release

# Run with detailed output
cargo test demo::test_triangle_attractor -- --nocapture
```

### Adding New Test Cases

```rust
// In tests/demo/
#[test]
fn test_my_new_case() {
    let case = DemoCase {
        name: "my_new_case",
        points: vec![
            Point2 { x: 0.0, y: 0.0 },
            Point2 { x: 1.0, y: 0.0 },
            Point2 { x: 0.5, y: 1.0 },
        ],
        edges: vec![
            (vec![0, 1], 2),  // 0,1 → 2
        ],
        eps: 1e-6,
        expected_fired: vec![0],
        expected_moved: vec![2],
        expected_fixed: vec![0, 1],
        expected_points: None,  // Exact check if needed
    };

    let result = run_one_step(&case);

    // Assert expectations
    assert_set_eq(result.fired_edges, case.expected_fired);
    assert_set_eq(result.moved_nodes, case.expected_moved);

    if let Some(expected_points) = case.expected_points {
        assert_points_close(result.points, expected_points, case.eps);
    }
}
```

### Validation Scripts

```python
def validate_backend_consistency(instance_id):
    """Ensure all backends produce consistent results"""

    sw_result = run_backend(instance_id, 'sw')
    hw_result = run_backend(instance_id, 'hw')

    # Same edges fired
    assert sw_result['counts']['edges_fired'] == hw_result['counts']['edges_fired']

    # Same nodes moved
    assert sw_result['counts']['nodes_moved'] == hw_result['counts']['nodes_moved']

    # Geometry within epsilon
    assert_geometry_close(sw_result, hw_result, eps=1e-3)

    print(f"✓ {instance_id} consistent across backends")

def validate_determinism(instance_id, backend, runs=3):
    """Ensure deterministic execution"""

    results = [run_backend(instance_id, backend) for _ in range(runs)]

    # All results identical
    for result in results[1:]:
        assert results[0] == result

    print(f"✓ {instance_id} on {backend} is deterministic")
```

## 🔧 Development Workflow

### Adding New Toy Instances

1. **Define the instance in `mhg-testkit::toy_instance`**

```rust
pub fn my_new_instance() -> ToyInstance {
    ToyInstance {
        name: "my_new_instance_v0",
        points_xy: vec![
            Point2 { x: 0.0, y: 0.0 },
            Point2 { x: 1.0, y: 0.0 },
            Point2 { x: 0.5, y: 1.0 },
        ],
        edges: vec![
            ToyEdge {
                tails: vec![0, 1],
                head: 2,
                footprint: ToyFootprint::Simplex {
                    anchors: vec![0, 1],
                    radius: 1.0,
                },
            },
        ],
        schedule: ToySchedule {
            ticks: 10,
            dirty_by_tick: vec![
                vec![0, 1],  // Initial dirty set
            ],
        },
    }
}
```

2. **Register in `toy_core::execute`**

```rust
fn run_instance(args: RunInstanceArgs) -> CommandResult {
    let inst = match args.instance.as_str() {
        "triangle_attractor_v0" => triangle_attractor_instance(),
        "my_new_instance_v0" => my_new_instance(),  // Add here
        _ => return error_result("unknown instance"),
    };
    // ...
}
```

3. **Test the new instance**

```bash
toy run --instance my_new_instance_v0 --backend sw
```

### Adding New Backends

1. **Add to `BackendKind` enum**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackendKind {
    // ... existing variants
    MyNewBackend,
}
```

2. **Implement execution logic**

```rust
fn run_toy_instance_and_collect_summary(/* ... */) -> RunSummary {
    match backend {
        // ... existing cases
        BackendKind::MyNewBackend => {
            let mut my_hw = MyHardware::new();
            my_hw.load_instance(inst);
            for _ in 0..ticks {
                my_hw.step_custom_logic();
            }
            (Some(my_hw.get_metrics()), start.elapsed())
        }
    }
}
```

3. **Add CLI mapping**

```rust
let backend_kind = match backend.as_str() {
    // ... existing mappings
    "my-backend" => BackendKind::MyNewBackend,
    // ...
};
```

### Debugging and Troubleshooting

#### Common Issues

**Simulation not converging:**
```bash
# Check geometry evolution
toy run --instance problematic_instance --backend sw --trace

# Compare with different backends
toy run --instance problematic_instance --backend hw
```

**Memory issues:**
```bash
# Check memory access patterns
toy run --instance large_instance --backend hw | jq '.data.counts.hw_counts'

# Reduce problem size for debugging
toy run --instance large_instance --backend sw  # Software has no limits
```

**Performance problems:**
```bash
# Profile execution time
time toy run --instance slow_instance --backend sw

# Compare backends
for backend in sw hw; do
    time toy run --instance slow_instance --backend $backend
done
```

#### Logging and Debugging

```rust
// Enable debug logging
env RUST_LOG=debug toy run --instance debug_instance --backend hw

// Check intermediate states
toy run --instance debug_instance --backend sw --trace
```

## 📈 Advanced Analysis

### Custom Metrics Collection

```rust
fn collect_custom_metrics(instance_id: &str, backend: BackendKind) -> CustomMetrics {
    let result = run_backend(instance_id, backend);

    CustomMetrics {
        computation_density: result.counts.edges_fired as f64 / result.meta.ticks_executed as f64,
        memory_efficiency: result.counts.hw_counts
            .map(|hw| hw.node_mem_reads as f64 / hw.cycles as f64)
            .unwrap_or(0.0),
        communication_overhead: result.counts.hw_counts
            .map(|hw| hw.event_q_pushes as f64 / hw.cycles as f64)
            .unwrap_or(0.0),
    }
}
```

### Comparative Benchmarking

```rust
fn benchmark_all_backends(instance_id: &str) -> BenchmarkReport {
    let mut report = BenchmarkReport::new(instance_id);

    for backend in BackendKind::iter() {
        let start = Instant::now();
        let result = run_backend(instance_id, backend);
        let duration = start.elapsed();

        report.add_result(backend, duration, result);
    }

    report.generate_comparison()
}
```

### Parameter Sweep Analysis

```rust
fn parameter_sweep(instance_template: &str, parameter_range: Range<f32>) -> SweepResults {
    parameter_range
        .map(|param| {
            let instance = instantiate_with_param(instance_template, param);
            let result = run_backend(&instance.name, BackendKind::ToyHardware);

            (param, extract_key_metrics(result))
        })
        .collect()
}
```

This usage guide provides comprehensive examples for effectively utilizing all MHG engine components, from basic execution to advanced hardware exploration and analysis workflows.