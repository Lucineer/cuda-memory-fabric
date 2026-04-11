# cuda-memory-fabric

Multi-layer agent memory — working, episodic, semantic, procedural with forgetting curves (Rust)

Part of the Cocapn memory layer — how agents remember, forget, and recall.

## What It Does

### Key Types

- `MemoryEntry` — core data structure
- `WorkingMemory` — core data structure
- `WorkingItem` — core data structure
- `EpisodicMemory` — core data structure
- `Episode` — core data structure
- `SemanticMemory` — core data structure
- _and 7 more (see source)_

## Quick Start

```bash
# Clone
git clone https://github.com/Lucineer/cuda-memory-fabric.git
cd cuda-memory-fabric

# Build
cargo build

# Run tests
cargo test
```

## Usage

```rust
use cuda_memory_fabric::*;

// See src/lib.rs for full API
// 14 unit tests included
```

### Available Implementations

- `WorkingMemory` — see source for methods
- `EpisodicMemory` — see source for methods
- `SemanticMemory` — see source for methods
- `ProceduralMemory` — see source for methods
- `MemoryFabric` — see source for methods

## Testing

```bash
cargo test
```

14 unit tests covering core functionality.

## Architecture

This crate is part of the **Cocapn Fleet** — a git-native multi-agent ecosystem.

- **Category**: memory
- **Language**: Rust
- **Dependencies**: See `Cargo.toml`
- **Status**: Active development

## Related Crates

- [cuda-temporal](https://github.com/Lucineer/cuda-temporal)
- [cuda-adaptation](https://github.com/Lucineer/cuda-adaptation)
- [cuda-context-window](https://github.com/Lucineer/cuda-context-window)

## Fleet Position

```
Casey (Captain)
├── JetsonClaw1 (Lucineer realm — hardware, low-level systems, fleet infrastructure)
├── Oracle1 (SuperInstance — lighthouse, architecture, consensus)
└── Babel (SuperInstance — multilingual scout)
```

## Contributing

This is a fleet vessel component. Fork it, improve it, push a bottle to `message-in-a-bottle/for-jetsonclaw1/`.

## License

MIT

---

*Built by JetsonClaw1 — part of the Cocapn fleet*
*See [cocapn-fleet-readme](https://github.com/Lucineer/cocapn-fleet-readme) for the full fleet roadmap*
