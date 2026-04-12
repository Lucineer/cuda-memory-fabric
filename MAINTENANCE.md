# cuda-memory-fabric — Maintenance Notes

## Purpose
Shared memory for multi-VM fleets. Multiple FLUX VMs communicating through typed, permission-gated memory.

## Architecture
- MemoryCell: value + owner + permissions + version + timestamp
- MemoryRegion: contiguous allocation with type (Private, Shared, Broadcast, RingBuffer, Stack, Heap)
- Fence: synchronization primitive (AllArrived, ValueEquals, ConfidenceAbove, Quorum)
- Conflict resolution: LastWriterWins, TrustWeighted, QuorumVote, Reject

## Key Design Decision: Version Tracking
Every write increments the cell version. This enables:
1. Conflict detection (concurrent writes to same cell)
2. Cache invalidation (stale reads detected by version mismatch)
3. Audit trail (who changed what, when)

## Memory Barrier Semantics
memory_barrier(vessel_id) ensures all writes by that vessel are visible to all other vessels BEFORE any subsequent reads. This is the fleet equivalent of a CPU memory barrier. Without it, vessel A might read stale data from vessel B.

## Related Crates
- flux-runtime-c: VM instances sharing memory
- cuda-capability-ports: I/O ports as special memory regions
- cuda-trust: trust scores gate memory permissions
- cuda-vm-scheduler: scheduler coordinates memory access
