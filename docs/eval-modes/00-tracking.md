# Evaluation Modes: Which Agent Context Yields the Greatest Impact?

> Tracking issue for clawbake's transition from generic identity optimization to targeted always-on agent context evaluation.

## Problem Statement

Clawbake currently generates and optimizes a single `identity.md` file — a generic system prompt document. But real always-on agents (openclaw, nanobot, hermes-agent) bootstrap their identity through a **constellation of context files**, each serving a distinct purpose:

| File | Purpose | Who uses it |
|------|---------|-------------|
| `SOUL.md` | Deep identity, values, voice — the "who" | openclaw, custom agents |
| `CLAUDE.md` | Project-scoped instructions for Claude Code | Every Claude Code project |
| `AGENTS.md` | Multi-agent coordination, delegation rules | Multi-agent systems |
| `MEMORY.md` | Persistent state, learned preferences, session history | nanobot, hermes-agent |
| Skills/Plugins | Executable capabilities (obra/superpowers, bmad-method) | Agents with tool ecosystems |

The specialty Claude tools market is saturated. Clawbake's differentiation should come from being **the** tool for optimizing always-on agent performance — and that means understanding which context layer delivers the most leverage.

## Core Questions

1. **Which context file has the highest marginal impact on agent quality?** Is it the soul (identity), the memory (learned state), or the project instructions?
2. **Which layers are currently undervalued/untested?** SOUL.md and MEMORY.md have almost no benchmarking ecosystem. CLAUDE.md has some informal best practices but no systematic optimization.
3. **Can clawbake runs target specific context layers independently?** Instead of one monolithic identity.md, can we eval-optimize SOUL.md while holding CLAUDE.md constant, and vice versa?
4. **What does convergence look like per layer?** SOUL.md may converge fast (identity is stable), while MEMORY.md may never converge (it grows).

## Proposed Architecture: Evaluation Modes

```
clawbake run --mode soul      # Optimize SOUL.md
clawbake run --mode claude    # Optimize CLAUDE.md
clawbake run --mode agents    # Optimize AGENTS.md
clawbake run --mode memory    # Optimize MEMORY.md
clawbake run --mode skills    # Evaluate skill/plugin impact
```

Each mode would have:
- **Its own eval case categories** (e.g., soul mode tests identity consistency across sessions; memory mode tests recall and preference learning)
- **Its own scoring weights** (e.g., soul mode weights persona fidelity heavily; skills mode weights task quality)
- **Its own optimization strategy** (e.g., soul mode mutates prose; memory mode prunes/restructures entries)
- **Hold-constant layers** (optimize one layer while other context files remain fixed)

## Hypothesis: Undervalued Layers

**MEMORY.md is the most undervalued.** Here's why:
- SOUL.md and CLAUDE.md have clear authoring patterns — they're write-once, tune occasionally
- MEMORY.md is the only layer that **grows with the agent** — it's the difference between a fresh agent and one that "knows" your codebase
- No tooling exists to evaluate whether a MEMORY.md entry actually improves downstream performance
- Always-on agents live or die by accumulated context quality

**Skills/plugins are the most undertested.** Specifically:
- Does adding obra/superpowers actually improve agent output vs. vanilla Claude Code?
- What's the marginal value of each skill? Are some net-negative (consuming context window for little gain)?
- Can clawbake A/B test agent performance with and without specific skills?

## Sub-Issues

| # | Mode | Doc | Priority | GitHub Issue |
|---|------|-----|----------|-------------|
| 1 | SOUL.md | [01-soul-mode.md](./01-soul-mode.md) | High | jhbarnett/clawbake#6 |
| 2 | CLAUDE.md / AGENTS.md | [02-claude-mode.md](./02-claude-mode.md) | High | jhbarnett/clawbake#7 |
| 3 | ~~AGENTS.md~~ | Merged into #2 — CLAUDE.md and AGENTS.md are interchangeable project instruction files | — | — |
| 4 | MEMORY.md | [04-memory-mode.md](./04-memory-mode.md) | **Critical** | jhbarnett/clawbake#8 |
| 5 | Skills/Plugins | [05-skills-mode.md](./05-skills-mode.md) | High | jhbarnett/clawbake#9 |

All 4 eval mode issues have been created on `jhbarnett/clawbake`. Memory mode (#8) is the highest priority — it should be picked up before Soul, Claude, or Skills modes.

> **Note**: `coflounder/clawbake` has issues disabled. All Clawbake issues live on `jhbarnett/clawbake`. PRs are opened from `coflounder/clawbake` (fork) targeting `coflounder:dev`.

## Success Criteria

- Clawbake can run targeted evaluations per context layer
- Users get quantitative signal on which layer to invest authoring effort in
- Always-on agent developers can use clawbake as their optimization loop, not just one-shot prompt engineers
