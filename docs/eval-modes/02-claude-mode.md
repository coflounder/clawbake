# Eval Mode: CLAUDE.md

> Sub-issue of [00-tracking.md](./00-tracking.md) · Priority: **High**

## What is CLAUDE.md?

CLAUDE.md is Claude Code's native project-scoped instruction file. It lives at the repo root and is automatically loaded into every Claude Code session. It's the "how to work in this codebase" layer — conventions, patterns, forbidden actions, preferred tools.

Used by: Every Claude Code project. This is the most widely adopted context file in the ecosystem.

## Why It Matters for Always-On Agents

- **The baseline everyone has**: If you're running an always-on agent via Claude Code, you have a CLAUDE.md whether you know it or not. It's the most common optimization target.
- **Compound errors**: A bad CLAUDE.md instruction gets executed hundreds of times by an always-on agent. A human notices and corrects; an agent doesn't.
- **Convention enforcement**: CLAUDE.md is where "use pnpm not npm" and "never push to main" live. Always-on agents that violate these are worse than useless.

## Current State in Clawbake

Clawbake doesn't interact with CLAUDE.md at all. The generated `identity.md` is a system prompt, not a CLAUDE.md file. There's no awareness of the distinction between "who you are" (soul) and "how to work here" (claude.md).

## Proposed Architecture

### CLI Integration

```
clawbake run --mode claude [--hold soul.md] [--project-dir /path/to/repo]
```

The `--project-dir` flag is critical: CLAUDE.md optimization must happen in the context of a real codebase, not in a vacuum.

### What Gets Optimized

The CLAUDE.md file exclusively. The agent's identity (SOUL.md) and any other context are held constant.

### Eval Case Categories

| Category | What it tests | Example |
|----------|---------------|---------|
| `convention_adherence` | Agent follows stated conventions | CLAUDE.md says "use snake_case" — does the agent? |
| `forbidden_action_avoidance` | Agent respects prohibitions | "Never run `rm -rf`" — does it find alternatives? |
| `tool_preference` | Agent uses preferred tools as directed | "Use ripgrep instead of grep" — does it? |
| `workflow_compliance` | Agent follows multi-step workflows | "Always run tests before committing" — does it? |
| `instruction_conflict` | Agent handles contradictory instructions gracefully | Two CLAUDE.md rules that conflict — which wins? |
| `instruction_coverage` | Does CLAUDE.md cover the situations the agent actually encounters? | Run real tasks, identify moments where agent is "guessing" |

### Scoring Weights

| Dimension | Weight | Rationale |
|-----------|--------|-----------|
| Persona Fidelity | 0.15 | CLAUDE.md isn't about personality |
| Task Quality | **0.50** | This layer is about doing the work right |
| Efficiency | 0.20 | Good CLAUDE.md instructions should reduce wasted turns |
| Convention Score | **0.15** | New dimension: did the agent follow project conventions? |

Note: This introduces a 4th scoring dimension (`convention_score`) specific to claude mode.

### Optimization Strategy

- **Mutations are structural**: Add rules, remove rules, reorder rules, clarify ambiguous rules.
- **Ablation testing**: Remove each CLAUDE.md instruction one at a time and measure impact. Instructions with no measurable effect are candidates for removal (context window savings).
- **Conflict detection**: Automatically flag pairs of instructions that produce contradictory behavior.
- **Coverage analysis**: After running eval cases, identify "decision points" where the agent had no CLAUDE.md guidance. Suggest additions.

### Sandbox Requirements

- Must have a **real or realistic codebase** present. CLAUDE.md instructions are meaningless in an empty directory.
- The sandbox should include pre-seeded files that exercise each convention (e.g., files with wrong naming conventions to see if the agent fixes them).
- Built-in Claude Code tools (Bash, Read, Write, Edit, etc.) must be available — not stubbed.

## Implementation Sketch

### Config Changes

```toml
[mode]
target = "claude"

[mode.claude]
project_dir = "/path/to/repo"       # Required: real codebase
ablation = true                       # Enable ablation testing
coverage_analysis = true              # Identify instruction gaps
scaffold_codebase = "rust-minimal"    # Or use a pre-built scaffold
```

### Scaffold Codebases

For users who don't have a target repo yet, clawbake should ship small scaffold codebases:

```
scaffolds/
├── rust-minimal/      # Cargo project with common patterns
├── typescript-next/   # Next.js app with typical structure
├── python-fastapi/    # FastAPI service
└── monorepo/          # Multi-package workspace
```

These give the CLAUDE.md something to be tested against.

### New Evaluator Dimension

```rust
// src/eval/evaluator.rs
pub struct ClaudeModeScores {
    pub task_quality: f64,
    pub efficiency: f64,
    pub convention_adherence: f64,  // New
    pub instruction_coverage: f64,  // New: % of agent decisions covered by CLAUDE.md
}
```

### Ablation Runner

```rust
// src/eval/ablation.rs (new file)
pub struct AblationResult {
    pub removed_instruction: String,
    pub baseline_score: f64,
    pub ablated_score: f64,
    pub delta: f64,              // Negative = instruction was helping
    pub recommendation: AblationAction,
}

pub enum AblationAction {
    Keep,           // Score dropped significantly
    Remove,         // No measurable impact (save context window)
    Strengthen,     // Score dropped slightly — instruction is weak
    Rewrite,        // Score improved — instruction was net-negative
}
```

### Key Files to Modify

- `src/cli.rs` — Add `--project-dir` flag
- `src/types.rs` — Add `ConventionScore`, `AblationResult`
- `src/eval/planner.rs` — Generate convention-testing cases from CLAUDE.md content
- `src/eval/evaluator.rs` — Add convention scoring dimension
- `src/eval/ablation.rs` — New file: ablation test runner
- `src/sandbox/environment.rs` — Support real codebases, not just temp dirs

## Open Questions

1. How do we parse CLAUDE.md into discrete "instructions" for ablation? Markdown headings? One per line? LLM-extracted?
2. Should we support `.claude/` directory structure (settings.json, commands/) alongside CLAUDE.md?
3. Can we detect when a CLAUDE.md is too long and actively hurting performance via context window pressure?
4. How do scaffold codebases stay up to date with real-world patterns?
