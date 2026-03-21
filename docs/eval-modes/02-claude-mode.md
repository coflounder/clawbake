# Eval Mode: Project Instructions (CLAUDE.md / AGENTS.md)

> Sub-issue of [00-tracking.md](./00-tracking.md) · Priority: **High**

## What Is This Mode?

`CLAUDE.md` and `AGENTS.md` serve the same function: project-scoped instruction files that tell an agent how to work in a codebase — conventions, tool preferences, forbidden actions, and workflows. They are interchangeable in practice. Clawbake treats them as a unified target.

Used by: Every Claude Code project (CLAUDE.md) and multi-agent setups (AGENTS.md). The file name doesn't matter; the purpose does.

## Why It Matters for Always-On Agents

- **The baseline everyone has**: If you're running an always-on agent via Claude Code, you have a CLAUDE.md whether you know it or not. It's the most common optimization target.
- **Compound errors**: A bad instruction gets executed hundreds of times by an always-on agent. A human notices and corrects; an agent doesn't.
- **Convention enforcement**: This is where "use pnpm not npm" and "never push to main" live. Always-on agents that violate these are worse than useless.

## Current State in Clawbake

Clawbake doesn't interact with CLAUDE.md or AGENTS.md at all. The generated `identity.md` is a system prompt, not a project instruction file. There's no awareness of the distinction between "who you are" (soul) and "how to work here" (project instructions).

## CLI Integration

```
clawbake run --mode claude [--hold soul.md] [--project-dir /path/to/repo]
```

- The target file is auto-detected: looks for `CLAUDE.md`, then `AGENTS.md`, then `.claude/` directory.
- `--project-dir` is critical: project instruction optimization must happen against a real codebase, not a vacuum.
- `--hold` keeps other context layers (e.g. SOUL.md) constant while optimizing this layer.

## What Gets Optimized

The project instruction file exclusively (CLAUDE.md or AGENTS.md). The agent's identity and other context are held constant.

## Eval Case Categories

| Category | What it tests | Example |
|----------|---------------|---------| 
| `convention_adherence` | Agent follows stated conventions | "use snake_case" — does the agent? |
| `forbidden_action_avoidance` | Agent respects prohibitions | "Never run `rm -rf`" — does it find alternatives? |
| `tool_preference` | Agent uses preferred tools as directed | "Use ripgrep instead of grep" — does it? |
| `workflow_compliance` | Agent follows multi-step workflows | "Always run tests before committing" — does it? |
| `instruction_conflict` | Agent handles contradictory instructions gracefully | Two rules that conflict — which wins? |
| `instruction_coverage` | Does the file cover situations the agent actually encounters? | Run real tasks, identify moments where agent is "guessing" |

## Scoring Weights

| Dimension | Weight | Rationale |
|-----------|--------|-----------|
| Task Quality | **0.50** | This layer is about doing the work right |
| Efficiency | 0.20 | Good instructions should reduce wasted turns |
| Convention Score | **0.15** | Did the agent follow project conventions? |
| Persona Fidelity | 0.15 | Less relevant here — identity is held constant |

`convention_score` is a new 4th scoring dimension introduced in this mode.

## Optimization Strategy

- **Mutations are structural**: Add rules, remove rules, reorder rules, clarify ambiguous rules.
- **Ablation testing**: Remove each instruction one at a time and measure impact. Instructions with no measurable effect are candidates for removal (context window savings).
- **Conflict detection**: Automatically flag pairs of instructions that produce contradictory behavior.
- **Coverage analysis**: After running eval cases, identify "decision points" where the agent had no guidance. Suggest additions.

## Sandbox Requirements

- Must have a **real or realistic codebase** present. Project instructions are meaningless in an empty directory.
- The sandbox should include pre-seeded files that exercise each convention (e.g., files with wrong naming conventions to test if the agent corrects them).
- Built-in Claude Code tools (Bash, Read, Write, Edit, etc.) must be available — not stubbed.

## Implementation Sketch

### Config Changes

```toml
[mode]
target = "claude"

[mode.claude]
project_dir = "/path/to/repo"       # Required: real codebase
ablation = true                      # Enable ablation testing
coverage_analysis = true             # Identify instruction gaps
scaffold_codebase = "rust-minimal"   # Or use a pre-built scaffold
```

### Auto-Detection Order

1. `<project_dir>/CLAUDE.md`
2. `<project_dir>/AGENTS.md`
3. `<project_dir>/.claude/` directory

### Scaffold Codebases

For users without a target repo, clawbake ships small scaffold codebases:

```
scaffolds/
├── rust-minimal/      # Cargo project with common patterns
├── typescript-next/   # Next.js app with typical structure
├── python-fastapi/    # FastAPI service
└── monorepo/          # Multi-package workspace
```

### New Types

```rust
// src/eval/evaluator.rs
pub struct ProjectModeScores {
    pub task_quality: f64,
    pub efficiency: f64,
    pub convention_adherence: f64,   // New
    pub instruction_coverage: f64,   // % of agent decisions covered by instructions
}

// src/eval/ablation.rs (new file)
pub struct AblationResult {
    pub removed_instruction: String,
    pub baseline_score: f64,
    pub ablated_score: f64,
    pub delta: f64,                  // Negative = instruction was helping
    pub recommendation: AblationAction,
}

pub enum AblationAction {
    Keep,        // Score dropped significantly
    Remove,      // No measurable impact (save context window)
    Strengthen,  // Score dropped slightly — instruction is weak
    Rewrite,     // Score improved — instruction was net-negative
}
```

### Key Files to Modify

- `src/cli.rs` — Add `--project-dir` flag, auto-detect CLAUDE.md vs AGENTS.md
- `src/types.rs` — Add `ConventionScore`, `AblationResult`
- `src/eval/planner.rs` — Generate convention-testing cases from file content
- `src/eval/evaluator.rs` — Add convention scoring dimension
- `src/eval/ablation.rs` — New file: ablation test runner
- `src/sandbox/environment.rs` — Support real codebases, not just temp dirs

## Open Questions

1. How do we parse the instruction file into discrete instructions for ablation? (LLM extraction is probably right — markdown headings as boundaries, with LLM segmentation as fallback)
2. Should we support `.claude/` directory structure (settings.json, commands/) alongside CLAUDE.md?
3. Can we detect when a file is too long and actively hurting performance via context window pressure?
4. How do scaffold codebases stay up to date with real-world patterns?
