# cuda-memory-fabric

**Shared memory fabric for multi-VM FLUX fleets.** Typed, permission-gated, conflict-resolved.

## Overview

Multiple FLUX VMs ("vessels") need to share data through a unified memory space. `cuda-memory-fabric` provides a typed, permission-gated memory system where vessels can read, write, subscribe to, and synchronize across shared memory regions — with built-in conflict detection and resolution.

> *The Deeper Connection.*

## Core Concepts

### Memory Cells
Each address holds a `MemoryCell` with a value, owner, permissions, version counter, and modification timestamp.

### Regions
Contiguous memory regions are allocated with a type:
- **Private** — owner-only read/write
- **Shared** — owner writes, all read
- **Broadcast** — all read/write, subscribers get notified
- **RingBuffer** / **Stack** / **Heap** — structured access patterns

### Fences
Synchronization primitives for coordinating across vessels:
- `AllArrived` — block until all listed vessels signal
- `ValueEquals` — block until a memory cell hits a target value
- `ConfidenceAbove` / `Quorum` — trust-based and consensus gates

### Conflict Resolution
When multiple vessels write to the same cell, conflicts are detected and resolved via pluggable strategies: LastWriterWins, TrustWeighted, QuorumVote, or Reject.

## Cross-Pollination

This crate integrates with the broader CUDA ecosystem:

| Crate | Relationship |
|-------|-------------|
| **cuda-capability-ports** | Memory fabric exposes capabilities through typed ports |
| **cuda-trust** | Trust scores feed directly into `TrustWeighted` conflict resolution |
| **cuda-vm-scheduler** | Scheduler uses fences and barriers to coordinate VM lifecycles |

## Usage

```rust
use cuda_memory_fabric::*;
use cuda_memory_fabric::operations::*;

let mut fabric = MemoryFabric::new();

// Allocate a broadcast region
let base = fabric_alloc_region(&mut fabric, 16, 1, RegionType::Broadcast);

// Subscribe vessel 2 to changes
fabric_subscribe(&mut fabric, 2, base, 16, CallbackType::OnWrite);

// Write and broadcast
let notifications = fabric_broadcast_write(&mut fabric, base, 42, 1);
// notifications delivered to vessel 2
```

## License

MIT
