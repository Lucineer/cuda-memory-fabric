# cuda-memory-fabric

**Four-layer memory: Working, Episodic, Semantic, Procedural.**

> An agent without memory is a reflex machine.
> An agent with the right memory is an expert.

## Four Layers

1. **Working Memory** - Fast, limited capacity, decays in seconds. Current task context.
2. **Episodic Memory** - Specific experiences with timestamps and emotional valence.
3. **Semantic Memory** - General knowledge extracted from episodes. The "wisdom" layer.
4. **Procedural Memory** - How to do things. Skills, patterns, automatic behaviors.

### Forgetting Curves

Each layer has its own decay rate:
- Working: seconds (half-life ~30s)
- Episodic: days (half-life ~1 week)
- Semantic: months (half-life ~6 months)
- Procedural: years (half-life ~5 years)

## Ecosystem Integration

- `cuda-persistence` - Checkpoint/restore for durable memory
- `cuda-immutable` - Persistent data structures for semantic memory
- `cuda-learning` - Extracts lessons from episodic memory
- `cuda-narrative` - Stories stored in episodic memory
- `cuda-skill` - Procedural memory for skill acquisition
- `cuda-attention` - Salience determines what enters working memory
- `cuda-emotion` - Emotional valence strengthens episodic encoding

## See Also

- [cuda-persistence](https://github.com/Lucineer/cuda-persistence) - State persistence
- [cuda-learning](https://github.com/Lucineer/cuda-learning) - Experience to lesson extraction
- [cuda-skill](https://github.com/Lucineer/cuda-skill) - Skill management
- [cuda-attention](https://github.com/Lucineer/cuda-attention) - Attention drives memory formation

## License

MIT OR Apache-2.0