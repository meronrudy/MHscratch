# Normalized Trace Hashing v1 (Byte-Level Canonicalization)

## Overview

This specification defines `trace_hash` for **Conformance Level 2**. It hashes only **semantically relevant, replay-sufficient** information:

* Ordered event log (post total-order)
* Applied deltas (post canonical reduction)
* Applied topology operations (post barrier conflict resolution)
* Epoch transitions
* Checkpoint digests (optional but recommended)

It excludes debug counters, wall-clock timings, and any non-replay data.

## 1. Inputs

A solver emits a `RunCapsule` (subset shown):

* `version: u16` (must be `1`)
* `determinism_mode: StrictBitwise | QuantizedEps`
* `quant_eps: Option<f32>` (required if `QuantizedEps`)
* `compile_hash: [u8; 32]` (hash of canonical instance bytes; required for Level 2)
* `ticks_executed: u32`
* `epochs: Vec<EpochRec>` (one per tick, or per tick boundary)
* `events: Vec<EventRec>` (events *popped/processed*, in canonical order)
* `deltas: Vec<DeltaRec>` (applied deltas after reduction, canonical order)
* `topo_ops: Vec<TopoOpRec>` (applied at barrier, canonical order)
* `checkpoints: Vec<CheckpointRec>` (canonical order by tick)

## 2. Canonical Ordering Rules (Must Hold; Verifier Checks)

**EventRec total order key** (must already be sorted):
`(time, priority, dst, src, class, seq)`

**DeltaRec order key** (applied/reduced):
`(tick, node, delta_kind, source_id, source_seq)`

Notes:
* If you reduce to one delta per `(tick,node,delta_kind)`, set `source_id=0, source_seq=0`.
* If you retain provenance, `source_id/source_seq` are stable IDs from the ordered event stream.

**TopoOpRec order key** (post resolution):
`(tick, op_kind, target_id, src, seq)`

**EpochRec order key**:
`(tick)`

**CheckpointRec order key**:
`(tick)`

If any list is not sorted, Level 2 conformance fails.

## 3. Canonical Binary Stream Format

Hash input is a single canonical byte stream produced by writing records in a fixed format.

**Endianness:** little-endian for fixed-width integers.  
**Varints:** unsigned LEB128, **minimal encoding only** (no redundant continuation bytes).  
**Signed integers:** zigzag + unsigned LEB128 minimal.

### 3.1 Stream Header

Write exactly:

* Magic: ASCII `CGRBTRC1` (8 bytes)
* `u16` version = `1`
* `u8` determinism_mode:
  * `0 = StrictBitwise`
  * `1 = QuantizedEps`
* `f32` `quant_eps` encoded as:
  * StrictBitwise: write `0.0f32` (bits 0)
  * QuantizedEps: write eps as IEEE-754 `f32` bits
* `compile_hash`: 32 bytes (raw)
* `ticks_executed`: `u32`

### 3.2 Tick Framing

For each `tick` from `0..ticks_executed` inclusive (or `1..ticks_executed`, pick one and keep it fixed), write a tick frame:

* Record tag `0x01` (u8)
* `tick` as `u32`

Then write, in this exact order:

#### 1. Epoch Record
* tag `0x02`
* `manifold_epoch` as `u64`
* `graph_epoch` as `u64`

#### 2. Event Block
* tag `0x10`
* `n_events` as varint u64
* then `n_events` event records:
  * tag `0x11`
  * `time` varint u64
  * `priority` varint u64
  * `dst` varint u64
  * `src` varint u64
  * `class` u8 (0=GEOM,1=FIELD,2=TOPO)
  * `seq` varint u64
  * `payload_hash` 32 bytes (BLAKE3 of canonical payload bytes; can be all-zero if unused)

#### 3. Delta Block
* tag `0x20`
* `n_deltas` varint u64
* then `n_deltas` delta records:
  * tag `0x21`
  * `node` varint u64
  * `delta_kind` u8 (0=PointUpdate,1=Constraint)
  * `source_id` varint u64
  * `source_seq` varint u64
  * delta payload:
    * PointUpdate:
      * `tx, ty, tz` as **canonical scalar** (see 3.3)
      * `weight` as canonical scalar
    * Constraint (if used):
      * `constraint_id` varint u64
      * `strength` canonical scalar

#### 4. Topology Operations Block
* tag `0x30`
* `n_ops` varint u64
* then `n_ops` operation records:
  * tag `0x31`
  * `op_kind` u8 (0=AddEdge,1=RemoveEdge,2=UpdateFootprint)
  * `target_id` varint u64 (edge id for remove/update; new edge id for add if stable)
  * `src` varint u64
  * `seq` varint u64
  * operation payload:
    * AddEdge:
      * `head` varint u64
      * `arity` u8
      * `tails[arity]` each varint u64
      * `signature_id` varint u64
      * `footprint` (v0.1):
        * `footprint_kind` u8 (0=InfluenceRadius,1=SimplexProxy)
        * InfluenceRadius:
          * `radius` canonical scalar
          * `anchor_present` u8 (0/1)
          * if 1: `anchor` varint u64
        * SimplexProxy:
          * `radius` canonical scalar
          * `k` u8
          * `pts[k]` each varint u64
    * RemoveEdge:
      * (no payload)
    * UpdateFootprint:
      * same `footprint` encoding as above

#### 5. Checkpoint Block
* tag `0x40`
* `n_checkpoints` varint u64 (usually 0 or 1 per tick)
* for each:
  * tag `0x41`
  * `checkpoint_kind` u8 (0=StateDigest)
  * `digest` 32 bytes

Finally, after the last tick frame, write end tag `0xFF`.

### 3.3 Canonical Scalar Encoding

You encode `f32`-like values in one of two modes:

**Mode A: StrictBitwise**
* Encode `f32` as raw IEEE-754 bits: `u32_le(to_bits(value))`

**Mode B: QuantizedEps**
* Quantize: `q = round(value / eps)` to a signed 32-bit integer
* Encode `q` as zigzag varint (minimal LEB128)

This is the only supported float canonicalization in v1.

## 4. Hash Function and Output

* Hash algorithm: **BLAKE3**
* Domain separation: hash the header first (as above), so trace hashes never collide with other artifacts.
* Output: 32-byte digest, rendered as lowercase hex.

`trace_hash = hex(blake3(canonical_trace_bytes))`

---

# Rust `cgrb_verify` CLI Skeleton (Level 0-2 Ready)

This is a buildable skeleton with:

* Input parsing
* Ordering checks
* Normalized trace hashing (streaming into blake3)
* Verdict JSON emission

## Cargo.toml

```toml
[package]
name = "cgrb_verify"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
blake3 = "1.5"
hex = "0.4"
```

## src/main.rs

```rust
use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;

mod trace_hash;

use trace_hash::{compute_trace_hash_v1, HashMode};

#[derive(Parser)]
#[command(name = "cgrb_verify")]
#[command(about = "CGRB verifier (kernel conformance + normalized trace hashing)")]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Verify solver output against declared conformance level
    Verify {
        /// Path to solver output JSON (RunCapsule)
        #[arg(long)]
        output: String,

        /// Conformance level claimed by solver (0,1,2)
        #[arg(long, default_value = "2")]
        level: u8,

        /// If set, require solver-provided trace_hash to match computed hash
        #[arg(long, default_value_t = true)]
        require_hash_match: bool,
    },

    /// Compute normalized trace hash only (prints hex)
    HashTrace {
        #[arg(long)]
        output: String,
    },
}

#[derive(Debug, Clone, Deserialize)]
struct RunCapsule {
    version: u16,
    determinism_mode: DeterminismMode,
    quant_eps: Option<f32>,

    // Required for Level 2
    compile_hash: Option<String>, // hex(32B)

    ticks_executed: u32,

    epochs: Vec<EpochRec>,
    events: Vec<EventRec>,
    deltas: Vec<DeltaRec>,
    topo_ops: Vec<TopoOpRec>,
    checkpoints: Vec<CheckpointRec>,

    // Solver may provide these (verifier recomputes)
    trace_hash: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DeterminismMode {
    StrictBitwise,
    QuantizedEps,
}

#[derive(Debug, Clone, Deserialize)]
struct EpochRec {
    tick: u32,
    manifold_epoch: u64,
    graph_epoch: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct EventRec {
    tick: u32,
    time: u64,
    priority: u32,
    dst: u32,
    src: u32,
    class: EventClass,
    seq: u64,
    payload_hash: Option<String>, // hex(32B) or omitted
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EventClass {
    Geom,
    Field,
    Topo,
}

#[derive(Debug, Clone, Deserialize)]
struct DeltaRec {
    tick: u32,
    node: u32,
    kind: DeltaKind,
    source_id: u32,
    source_seq: u64,

    // PointUpdate payload
    tx: Option<f32>,
    ty: Option<f32>,
    tz: Option<f32>,
    weight: Option<f32>,

    // Constraint payload (optional)
    constraint_id: Option<u32>,
    strength: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DeltaKind {
    PointUpdate,
    Constraint,
}

#[derive(Debug, Clone, Deserialize)]
struct TopoOpRec {
    tick: u32,
    op_kind: TopoOpKind,
    target_id: u32,
    src: u32,
    seq: u64,

    // payload is toy/minimal; extend as needed
    head: Option<u32>,
    arity: Option<u8>,
    tails: Option<Vec<u32>>,
    signature_id: Option<u32>,

    footprint: Option<FootprintRec>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TopoOpKind {
    AddEdge,
    RemoveEdge,
    UpdateFootprint,
}

#[derive(Debug, Clone, Deserialize)]
struct FootprintRec {
    kind: FootprintKind,
    radius: f32,
    anchor: Option<u32>,
    simplex_pts: Option<Vec<u32>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum FootprintKind {
    InfluenceRadius,
    SimplexProxy,
}

#[derive(Debug, Clone, Deserialize)]
struct CheckpointRec {
    tick: u32,
    digest: String, // hex(32B)
}

#[derive(Debug, Serialize)]
struct Verdict {
    pass: bool,
    level: u8,
    booleans: serde_json::Value,
    hashes: serde_json::Value,
    counts: serde_json::Value,
    errors: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Command::Verify {
            output,
            level,
            require_hash_match,
        } => {
            let capsule = load_capsule(&output)?;
            let verdict = verify_capsule(&capsule, level, require_hash_match);
            println!("{}", serde_json::to_string_pretty(&verdict)?);
            if !verdict.pass {
                std::process::exit(1);
            }
            Ok(())
        }

        Command::HashTrace { output } => {
            let capsule = load_capsule(&output)?;
            let computed = compute_hash_only(&capsule)?;
            println!("{computed}");
            Ok(())
        }
    }
}

fn load_capsule(path: &str) -> Result<RunCapsule> {
    let s = fs::read_to_string(path).with_context(|| format!("read output: {path}"))?;
    let capsule: RunCapsule = serde_json::from_str(&s).context("parse output JSON")?;
    Ok(capsule)
}

fn verify_capsule(c: &RunCapsule, level: u8, require_hash_match: bool) -> Verdict {
    let mut errors = Vec::<String>::new();

    // Basic version check
    if c.version != 1 {
        errors.push(format!("unsupported capsule version: {}", c.version));
    }

    // Level requirements
    if level >= 2 && c.compile_hash.is_none() {
        errors.push("level>=2 requires compile_hash".to_string());
    }

    // Ordering checks (required for level>=1)
    if level >= 1 {
        if let Err(e) = check_ordering(c) {
            errors.push(e.to_string());
        }
    }

    // Hash check (required for level>=2)
    let mut computed_trace_hash: Option<String> = None;
    if level >= 2 {
        match compute_hash_only(c) {
            Ok(h) => computed_trace_hash = Some(h),
            Err(e) => errors.push(format!("trace hash compute failed: {e}")),
        }

        if require_hash_match {
            match (&c.trace_hash, &computed_trace_hash) {
                (Some(given), Some(comp)) if given.eq_ignore_ascii_case(comp) => {}
                (Some(given), Some(comp)) => {
                    errors.push(format!("trace_hash mismatch: given={given} computed={comp}"));
                }
                (None, Some(_)) => errors.push("solver output missing trace_hash".to_string()),
                _ => {}
            }
        }
    }

    let pass = errors.is_empty();

    Verdict {
        pass,
        level,
        booleans: serde_json::json!({
            "ordering": level < 1 || errors.iter().all(|e| !e.contains("ordering")),
            "hashes": level < 2 || computed_trace_hash.is_some(),
        }),
        hashes: serde_json::json!({
            "compile": c.compile_hash,
            "trace_given": c.trace_hash,
            "trace_computed": computed_trace_hash,
        }),
        counts: serde_json::json!({
            "ticks_executed": c.ticks_executed,
            "events": c.events.len(),
            "deltas": c.deltas.len(),
            "topo_ops": c.topo_ops.len(),
            "checkpoints": c.checkpoints.len(),
        }),
        errors,
    }
}

fn compute_hash_only(c: &RunCapsule) -> Result<String> {
    let compile_hash_hex = c.compile_hash.clone().ok_or_else(|| anyhow!("missing compile_hash"))?;
    let compile_hash = decode_32(&compile_hash_hex).context("decode compile_hash")?;

    let mode = match c.determinism_mode {
        DeterminismMode::StrictBitwise => HashMode::StrictBitwise,
        DeterminismMode::QuantizedEps => {
            let eps = c.quant_eps.ok_or_else(|| anyhow!("quantized_eps mode requires quant_eps"))?;
            HashMode::QuantizedEps { eps }
        }
    };

    let trace_hash = compute_trace_hash_v1(
        &compile_hash,
        c.ticks_executed,
        &c.epochs,
        &c.events,
        &c.deltas,
        &c.topo_ops,
        &c.checkpoints,
        mode,
    )?;

    Ok(trace_hash)
}

fn check_ordering(c: &RunCapsule) -> Result<()> {
    // epochs by tick
    if !is_sorted_by_key(&c.epochs, |x| x.tick) {
        return Err(anyhow!("ordering: epochs not sorted by tick"));
    }

    // events by canonical key
    if !is_sorted_by_key(&c.events, |e| {
        (
            e.tick,
            e.time,
            e.priority,
            e.dst,
            e.src,
            class_ord(&e.class),
            e.seq,
        )
    }) {
        return Err(anyhow!("ordering: events not sorted by canonical key"));
    }

    // deltas by canonical key
    if !is_sorted_by_key(&c.deltas, |d| {
        (
            d.tick,
            d.node,
            delta_kind_ord(&d.kind),
            d.source_id,
            d.source_seq,
        )
    }) {
        return Err(anyhow!("ordering: deltas not sorted by canonical key"));
    }

    // topo ops by canonical key
    if !is_sorted_by_key(&c.topo_ops, |t| {
        (t.tick, topo_kind_ord(&t.op_kind), t.target_id, t.src, t.seq)
    }) {
        return Err(anyhow!("ordering: topo_ops not sorted by canonical key"));
    }

    // checkpoints by tick
    if !is_sorted_by_key(&c.checkpoints, |x| x.tick) {
        return Err(anyhow!("ordering: checkpoints not sorted by tick"));
    }

    Ok(())
}

fn class_ord(c: &EventClass) -> u8 {
    match c {
        EventClass::Geom => 0,
        EventClass::Field => 1,
        EventClass::Topo => 2,
    }
}

fn delta_kind_ord(k: &DeltaKind) -> u8 {
    match k {
        DeltaKind::PointUpdate => 0,
        DeltaKind::Constraint => 1,
    }
}

fn topo_kind_ord(k: &TopoOpKind) -> u8 {
    match k {
        TopoOpKind::AddEdge => 0,
        TopoOpKind::RemoveEdge => 1,
        TopoOpKind::UpdateFootprint => 2,
    }
}

fn is_sorted_by_key<T, K: Ord>(xs: &[T], mut f: impl FnMut(&T) -> K) -> bool {
    xs.windows(2).all(|w| f(&w[0]) <= f(&w[1]))
}

fn decode_32(hex_str: &str) -> Result<[u8; 32]> {
    let bytes = hex::decode(hex_str)?;
    if bytes.len() != 32 {
        return Err(anyhow!("expected 32 bytes, got {}", bytes.len()));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}
```

## src/trace_hash.rs

```rust
use anyhow::{anyhow, Result};
use blake3::Hasher;

// Reuse these structs from main via `pub` in a real crate split.
// Here we keep minimal signatures and rely on identical field names/types.
use crate::{CheckpointRec, DeltaKind, DeltaRec, EpochRec, EventClass, EventRec, FootprintKind, TopoOpKind, TopoOpRec};

pub enum HashMode {
    StrictBitwise,
    QuantizedEps { eps: f32 },
}

pub fn compute_trace_hash_v1(
    compile_hash: &[u8; 32],
    ticks_executed: u32,
    epochs: &[EpochRec],
    events: &[EventRec],
    deltas: &[DeltaRec],
    topo_ops: &[TopoOpRec],
    checkpoints: &[CheckpointRec],
    mode: HashMode,
) -> Result<String> {
    let mut h = Hasher::new();

    // Header
    h.update(b"CGRBTRC1"); // 8 bytes
    write_u16_le(&mut h, 1); // version
    match mode {
        HashMode::StrictBitwise => {
            write_u8(&mut h, 0);
            write_f32_bits(&mut h, 0.0);
        }
        HashMode::QuantizedEps { eps } => {
            write_u8(&mut h, 1);
            write_f32_bits(&mut h, eps);
        }
    }
    h.update(compile_hash);
    write_u32_le(&mut h, ticks_executed);

    // Build per-tick slices without allocating big maps:
    // Assume inputs are already sorted and tick-tagged.
    let mut epoch_i = 0usize;
    let mut event_i = 0usize;
    let mut delta_i = 0usize;
    let mut topo_i = 0usize;
    let mut ckpt_i = 0usize;

    // Decide whether you want frames 0..ticks_executed or 1..ticks_executed.
    // v1: include tick 0..ticks_executed-1 (executed ticks)
    for tick in 0..ticks_executed {
        write_u8(&mut h, 0x01);
        write_u32_le(&mut h, tick);

        // Epoch record (required). If absent for a tick, fail.
        if epoch_i >= epochs.len() || epochs[epoch_i].tick != tick {
            return Err(anyhow!("missing epoch for tick {}", tick));
        }
        write_u8(&mut h, 0x02);
        write_u64_le(&mut h, epochs[epoch_i].manifold_epoch);
        write_u64_le(&mut h, epochs[epoch_i].graph_epoch);
        epoch_i += 1;

        // Events block
        write_u8(&mut h, 0x10);
        let n_events = count_while(events, event_i, |e| e.tick == tick);
        write_var_u64(&mut h, n_events as u64);
        for _ in 0..n_events {
            let e = &events[event_i];
            write_u8(&mut h, 0x11);
            write_var_u64(&mut h, e.time);
            write_var_u64(&mut h, e.priority as u64);
            write_var_u64(&mut h, e.dst as u64);
            write_var_u64(&mut h, e.src as u64);
            write_u8(&mut h, class_ord(&e.class));
            write_var_u64(&mut h, e.seq);

            let ph = match &e.payload_hash {
                Some(hex) => decode_32(hex)?,
                None => [0u8; 32],
            };
            h.update(&ph);

            event_i += 1;
        }

        // Deltas block
        write_u8(&mut h, 0x20);
        let n_deltas = count_while(deltas, delta_i, |d| d.tick == tick);
        write_var_u64(&mut h, n_deltas as u64);
        for _ in 0..n_deltas {
            let d = &deltas[delta_i];
            write_u8(&mut h, 0x21);
            write_var_u64(&mut h, d.node as u64);
            write_u8(&mut h, delta_kind_ord(&d.kind));
            write_var_u64(&mut h, d.source_id as u64);
            write_var_u64(&mut h, d.source_seq);

            match d.kind {
                DeltaKind::PointUpdate => {
                    let tx = d.tx.ok_or_else(|| anyhow!("missing tx"))?;
                    let ty = d.ty.ok_or_else(|| anyhow!("missing ty"))?;
                    let tz = d.tz.ok_or_else(|| anyhow!("missing tz"))?;
                    let w = d.weight.ok_or_else(|| anyhow!("missing weight"))?;

                    write_scalar(&mut h, tx, &mode);
                    write_scalar(&mut h, ty, &mode);
                    write_scalar(&mut h, tz, &mode);
                    write_scalar(&mut h, w, &mode);
                }
                DeltaKind::Constraint => {
                    let cid = d.constraint_id.ok_or_else(|| anyhow!("missing constraint_id"))?;
                    let s = d.strength.ok_or_else(|| anyhow!("missing strength"))?;
                    write_var_u64(&mut h, cid as u64);
                    write_scalar(&mut h, s, &mode);
                }
            }

            delta_i += 1;
        }

        // Topology ops block
        write_u8(&mut h, 0x30);
        let n_ops = count_while(topo_ops, topo_i, |t| t.tick == tick);
        write_var_u64(&mut h, n_ops as u64);
        for _ in 0..n_ops {
            let t = &topo_ops[topo_i];
            write_u8(&mut h, 0x31);
            write_u8(&mut h, topo_kind_ord(&t.op_kind));
            write_var_u64(&mut h, t.target_id as u64);
            write_var_u64(&mut h, t.src as u64);
            write_var_u64(&mut h, t.seq);

            match t.op_kind {
                TopoOpKind::AddEdge => {
                    let head = t.head.ok_or_else(|| anyhow!("missing head for AddEdge"))?;
                    let arity = t.arity.ok_or_else(|| anyhow!("missing arity for AddEdge"))?;
                    let tails = t.tails.as_ref().ok_or_else(|| anyhow!("missing tails for AddEdge"))?;
                    if tails.len() != arity as usize {
                        return Err(anyhow!("tails length != arity"));
                    }
                    let sig = t.signature_id.ok_or_else(|| anyhow!("missing signature_id for AddEdge"))?;

                    write_var_u64(&mut h, head as u64);
                    write_u8(&mut h, arity);
                    for &u in tails {
                        write_var_u64(&mut h, u as u64);
                    }
                    write_var_u64(&mut h, sig as u64);

                    write_footprint(&mut h, t.footprint.as_ref(), &mode)?;
                }
                TopoOpKind::RemoveEdge => {
                    // no payload
                }
                TopoOpKind::UpdateFootprint => {
                    write_footprint(&mut h, t.footprint.as_ref(), &mode)?;
                }
            }

            topo_i += 1;
        }

        // Checkpoints block (0 or more per tick)
        write_u8(&mut h, 0x40);
        let n_ck = count_while(checkpoints, ckpt_i, |c| c.tick == tick);
        write_var_u64(&mut h, n_ck as u64);
        for _ in 0..n_ck {
            let c = &checkpoints[ckpt_i];
            write_u8(&mut h, 0x41);
            write_u8(&mut h, 0); // kind = StateDigest
            let dig = decode_32(&c.digest)?;
            h.update(&dig);
            ckpt_i += 1;
        }
    }

    write_u8(&mut h, 0xFF);
    let out = h.finalize();
    Ok(out.to_hex().to_string())
}

fn write_footprint(h: &mut Hasher, fp: Option<&FootprintRec>, mode: &HashMode) -> Result<()> {
    let fp = fp.ok_or_else(|| anyhow!("missing footprint"))?;
    write_u8(h, match fp.kind {
        FootprintKind::InfluenceRadius => 0,
        FootprintKind::SimplexProxy => 1,
    });
    write_scalar(h, fp.radius, mode);

    match fp.kind {
        FootprintKind::InfluenceRadius => {
            match fp.anchor {
                Some(a) => {
                    write_u8(h, 1);
                    write_var_u64(h, a as u64);
                }
                None => write_u8(h, 0),
            }
        }
        FootprintKind::SimplexProxy => {
            let pts = fp.simplex_pts.as_ref().ok_or_else(|| anyhow!("missing simplex_pts"))?;
            write_u8(h, pts.len() as u8);
            for &p in pts {
                write_var_u64(h, p, as u64);
            }
        }
    }
    Ok(())
}

fn write_scalar(h: &mut Hasher, v: f32, mode: &HashMode) {
    match mode {
        HashMode::StrictBitwise => write_f32_bits(h, v),
        HashMode::QuantizedEps { eps } => {
            // q = round(v/eps)
            let q = (v / *eps).round() as i64;
            write_var_i64_zigzag(h, q);
        }
    }
}

// --- Encoding helpers (no intermediate buffer; hash streaming) ---

fn write_u8(h: &mut Hasher, x: u8) {
    h.update(&[x]);
}

fn write_u16_le(h: &mut Hasher, x: u16) {
    h.update(&x.to_le_bytes());
}

fn write_u32_le(h: &mut Hasher, x: u32) {
    h.update(&x.to_le_bytes());
}

fn write_u64_le(h: &mut Hasher, x: u64) {
    h.update(&x.to_le_bytes());
}

fn write_f32_bits(h: &mut Hasher, v: f32) {
    write_u32_le(h, v.to_bits());
}

// Unsigned LEB128 minimal
fn write_var_u64(h: &mut Hasher, mut x: u64) {
    loop {
        let byte = (x & 0x7F) as u8;
        x >>= 7;
        if x == 0 {
            write_u8(h, byte);
            break;
        } else {
            write_u8(h, byte | 0x80);
        }
    }
}

fn write_var_i64_zigzag(h: &mut Hasher, x: i64) {
    let zz = ((x << 1) ^ (x >> 63)) as u64;
    write_var_u64(h, zz);
}

fn class_ord(c: &EventClass) -> u8 {
    match c {
        EventClass::Geom => 0,
        EventClass::Field => 1,
        EventClass::Topo => 2,
    }
}

fn delta_kind_ord(k: &DeltaKind) -> u8 {
    match k {
        DeltaKind::PointUpdate => 0,
        DeltaKind::Constraint => 1,
    }
}

fn topo_kind_ord(k: &TopoOpKind) -> u8 {
    match k {
        TopoOpKind::AddEdge => 0,
        TopoOpKind::RemoveEdge => 1,
        TopoOpKind::UpdateFootprint => 2,
    }
}

fn count_while<T>(xs: &[T], mut i: usize, pred: impl Fn(&T) -> bool) -> usize {
    let mut n = 0;
    while i < xs.len() && pred(&xs[i]) {
        n += 1;
        i += 1;
    }
    n
}

fn decode_32(hex_str: &str) -> Result<[u8; 32]> {
    let bytes = hex::decode(hex_str)?;
    if bytes.len() != 32 {
        return Err(anyhow!("expected 32 bytes, got {}", bytes.len()));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

// --- Shim to avoid import tangles in snippet form ---
// In real code, remove shim and use the actual FootprintRec from main.
#[derive(Debug, Clone)]
pub struct FootprintRecShim {
    pub kind: FootprintKind,
    pub radius: f32,
    pub anchor: Option<u32>,
    pub simplex_pts: Option<Vec<u32>>,
}
```

> In real code, put the shared structs in a `cgrb_types` crate/module and import them from both `main.rs` and `trace_hash.rs`. The shim is only to keep this snippet standalone.

---

## Minimal `solver_output` JSON Expectations (for Level 2)

Your solver output JSON needs, at minimum:

* `version: 1`
* `determinism_mode: "strict_bitwise" | "quantized_eps"`
* `quant_eps` if quantized
* `compile_hash: <hex32>`
* `ticks_executed`
* sorted lists: `epochs/events/deltas/topo_ops/checkpoints`
* optional `trace_hash` (verifier can require it)