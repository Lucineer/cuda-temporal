# cuda-temporal

Temporal reasoning — time intervals, causal chains, deadline urgency, scheduling with conflict detection (Rust)

Part of the Cocapn memory layer — how agents remember, forget, and recall.

## What It Does

### Key Types

- `Interval` — core data structure
- `TemporalEvent` — core data structure
- `CausalChain` — core data structure
- `ScheduledTask` — core data structure
- `TemporalScheduler` — core data structure
- `FuturePrediction` — core data structure
- _and 1 more (see source)_

## Quick Start

```bash
# Clone
git clone https://github.com/Lucineer/cuda-temporal.git
cd cuda-temporal

# Build
cargo build

# Run tests
cargo test
```

## Usage

```rust
use cuda_temporal::*;

// See src/lib.rs for full API
// 14 unit tests included
```

### Available Implementations

- `Interval` — see source for methods
- `CausalChain` — see source for methods
- `TemporalScheduler` — see source for methods
- `TemporalReasoner` — see source for methods

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

- [cuda-memory-fabric](https://github.com/Lucineer/cuda-memory-fabric)
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
