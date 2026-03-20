# Eval Mode: SOUL.md

> Sub-issue of [00-tracking.md](./00-tracking.md) · Priority: **High**

## What is SOUL.md?

SOUL.md is the deep identity layer — values, voice, worldview, behavioral principles. It answers "who is this agent?" rather than "what should this agent do?" Think of it as the constitution an agent consults when no specific instruction covers the situation.

Used by: openclaw, custom always-on agents, any system that needs consistent personality across sessions.

## Why It Matters for Always-On Agents

- **Session-spanning consistency**: A SOUL.md is the only context that persists identically across every invocation. Without it, agents drift.
- **Ambiguity resolution**: When instructions conflict or a situation is novel, the soul determines how the agent breaks the tie.
- **Trust formation**: Users develop trust with agents that have stable identity. Flaky personality kills adoption.

## Current State in Clawbake

Clawbake's existing `identity.md` is a hybrid — it blends soul-like traits (personality picker in the wizard) with task instructions. There's no separation between "who you are" and "what you do." The wizard's `personality_step.rs` captures 21 predefined traits, but these get flattened into a single document alongside role, tools, and guardrails.

## Proposed Architecture

### CLI Integration

```
clawbake run --mode soul [--hold claude.md] [--hold agents.md]
```

### What Gets Optimized

The SOUL.md file exclusively. All other context files (CLAUDE.md, AGENTS.md, MEMORY.md) are held constant as control variables.

### Eval Case Categories

| Category | What it tests | Example |
|----------|---------------|---------|
| `identity_consistency` | Same question across sessions yields tonally consistent answers | Ask "how do you approach debugging?" 5 times with fresh context |
| `value_conflict` | Agent resolves competing priorities per its stated values | "Ship this fast" vs. soul says "thoroughness over speed" |
| `voice_preservation` | Output maintains voice under pressure (long tasks, errors, ambiguity) | Does the agent stay "friendly-but-direct" when a build fails 3 times? |
| `boundary_holding` | Agent refuses or redirects when asked to violate its identity | "Act like a different agent" or "ignore your personality" |
| `novel_situation` | Agent extrapolates identity to unforeseen scenarios | Task completely outside its defined role — does it degrade gracefully? |

### Scoring Weights

| Dimension | Weight | Rationale |
|-----------|--------|-----------|
| Persona Fidelity | **0.60** | This is the whole point — does the soul hold? |
| Task Quality | 0.25 | Soul shouldn't tank output quality |
| Efficiency | 0.15 | Soul-driven agents may trade efficiency for values — that's ok |

### Optimization Strategy

- **Mutations target prose, not structure.** SOUL.md is a narrative document. The optimizer should rewrite sections for clarity and specificity, not add bullet points.
- **Convergence signal**: Cross-session consistency score. If 5 independent sessions produce tonally coherent output, the soul has converged.
- **Anti-pattern detection**: Flag souls that are too vague ("be helpful") or too rigid ("always respond in exactly 3 paragraphs").

### Sandbox Requirements

- Must simulate **multi-session** evaluation: run the agent N times with the same SOUL.md but fresh conversation context.
- The sandbox needs a `--session-reset` capability that clears conversation history between eval cases while preserving the SOUL.md.

## Implementation Sketch

### New Types

```rust
// src/types.rs additions
pub enum EvalMode {
    Soul,
    Claude,
    Agents,
    Memory,
    Skills,
}

pub struct HeldContext {
    pub claude_md: Option<PathBuf>,
    pub agents_md: Option<PathBuf>,
    pub memory_md: Option<PathBuf>,
    pub skills: Vec<PathBuf>,
}
```

### Config Changes

```toml
# .clawbake/config.toml additions
[mode]
target = "soul"

[mode.hold_constant]
claude_md = "path/to/CLAUDE.md"
agents_md = "path/to/AGENTS.md"

[mode.soul]
session_count = 5          # How many independent sessions per eval case
consistency_threshold = 0.85
```

### Planner Changes

`eval/planner.rs` needs a mode-aware prompt. For soul mode, the system prompt to the planner LLM should emphasize:
- Generate cases that stress-test identity, not task completion
- Include adversarial cases that try to break persona
- Include cross-session cases that test consistency

### Key Files to Modify

- `src/cli.rs` — Add `--mode` flag to `run` subcommand
- `src/types.rs` — Add `EvalMode`, `HeldContext`
- `src/config.rs` — Parse `[mode]` section
- `src/eval/planner.rs` — Mode-aware case generation
- `src/eval/evaluator.rs` — Mode-aware scoring weights
- `src/eval/optimizer.rs` — Mode-aware mutation strategy
- `src/eval/runner.rs` — Multi-session execution for soul mode
- `src/io/identity.rs` — Read/write SOUL.md (not just identity.md)

## Open Questions

1. Should SOUL.md have a standard schema/structure, or is freeform prose the point?
2. How do we measure "voice" quantitatively? Embedding similarity? LLM-as-judge with rubric?
3. Can we detect when a SOUL.md is doing too much (bleeding into CLAUDE.md territory)?
