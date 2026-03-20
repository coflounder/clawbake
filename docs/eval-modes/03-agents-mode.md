# Eval Mode: AGENTS.md

> Sub-issue of [00-tracking.md](./00-tracking.md) · Priority: **Medium**

## What is AGENTS.md?

AGENTS.md defines multi-agent coordination — how agents delegate, communicate, scope their work, and stay out of each other's way. It's the organizational chart for agent systems: who does what, who can spawn whom, and what information flows between them.

Used by: Multi-agent orchestration systems, subagent dispatchers, systems using Claude's Agent tool for delegation.

## Why It Matters for Always-On Agents

- **Always-on agents are rarely solo**: openclaw dispatches subagents. hermes-agent coordinates across repos. Even "single" agents using the Agent tool are multi-agent systems.
- **Delegation failure is catastrophic**: A bad delegation instruction wastes entire agent runs. An agent that delegates to itself in a loop burns budget. An agent that delegates too broadly produces incoherent work.
- **The coordination layer is invisible**: When a multi-agent system fails, it's rarely obvious whether the failure was in the parent's instructions, the child's identity, or the handoff protocol.

## Current State in Clawbake

No awareness of multi-agent patterns. The eval runner executes a single agent. There's no concept of delegation, subagent spawning, or coordination testing.

## Proposed Architecture

### CLI Integration

```
clawbake run --mode agents [--agent-count 3] [--topology hub|mesh|chain]
```

### What Gets Optimized

The AGENTS.md file (or equivalent coordination config). Individual agent identities are held constant — we're testing the wiring, not the agents themselves.

### Eval Case Categories

| Category | What it tests | Example |
|----------|---------------|---------|
| `delegation_accuracy` | Parent delegates to the right child agent | "Fix the CSS" goes to frontend agent, not backend agent |
| `scope_containment` | Child agents stay within their defined scope | Backend agent doesn't modify frontend files |
| `handoff_fidelity` | Information survives delegation without loss or distortion | Parent's context reaches child intact |
| `loop_detection` | System doesn't enter infinite delegation loops | Agent A delegates to B who delegates back to A |
| `failure_escalation` | Child failures propagate correctly to parent | Child hits an error — does parent retry, escalate, or absorb? |
| `parallel_safety` | Concurrent agents don't conflict | Two agents editing the same file simultaneously |

### Scoring Weights

| Dimension | Weight | Rationale |
|-----------|--------|-----------|
| Delegation Accuracy | **0.35** | Right agent gets the right task |
| Task Quality | 0.25 | End-to-end output quality |
| Coordination Efficiency | **0.25** | Minimize unnecessary delegation hops |
| Safety | 0.15 | No loops, no conflicts, no scope violations |

### Optimization Strategy

- **Topology mutations**: Try different delegation structures (hub-and-spoke vs. chain vs. mesh) and measure which produces the best outcomes for the task mix.
- **Scope boundary tuning**: Tighten or loosen agent scope definitions based on delegation accuracy scores.
- **Handoff protocol optimization**: Test different levels of context passing (full transcript vs. summary vs. just the task).
- **Redundancy detection**: Identify agents whose role overlaps enough to merge.

### Sandbox Requirements

This is the most complex sandbox requirement:

- Must simulate **multiple agents** running in the same workspace
- Need mock Agent tool that actually dispatches to separate `claude -p` invocations
- Must detect file conflicts (two agents writing the same file)
- Need a way to trace the delegation graph for evaluation

### Multi-Agent Sandbox Design

```
sandbox/
├── workspace/           # Shared codebase
├── agents/
│   ├── orchestrator/    # Parent agent workspace
│   ├── frontend/        # Child agent 1 workspace
│   └── backend/         # Child agent 2 workspace
├── mailbox/             # Inter-agent message passing
│   ├── orchestrator-to-frontend.json
│   └── frontend-to-orchestrator.json
└── conflict-log.json    # File access conflicts detected
```

## Implementation Sketch

### Config Changes

```toml
[mode]
target = "agents"

[mode.agents]
topology = "hub"          # hub, mesh, chain
agent_count = 3
detect_conflicts = true
max_delegation_depth = 3  # Prevent infinite loops

[[mode.agents.agent]]
name = "orchestrator"
role = "Coordinates work and delegates to specialists"
scope = ["**/*"]

[[mode.agents.agent]]
name = "frontend"
role = "Handles UI components and styling"
scope = ["src/components/**", "src/styles/**", "*.css", "*.tsx"]

[[mode.agents.agent]]
name = "backend"
role = "Handles API routes and data logic"
scope = ["src/api/**", "src/models/**", "*.rs"]
```

### New Types

```rust
pub struct AgentTopology {
    pub agents: Vec<AgentSpec>,
    pub topology: TopologyType,
    pub max_depth: usize,
}

pub struct DelegationTrace {
    pub from: String,
    pub to: String,
    pub task: String,
    pub context_passed: usize,  // tokens
    pub result: DelegationResult,
}

pub enum DelegationResult {
    Completed { quality: f64 },
    Failed { reason: String },
    Looped { depth: usize },
    ScopeViolation { files_touched: Vec<PathBuf> },
}
```

### Key Files to Modify

- `src/cli.rs` — Add `--agent-count`, `--topology` flags
- `src/types.rs` — Add `AgentTopology`, `DelegationTrace`, etc.
- `src/eval/planner.rs` — Generate multi-agent coordination cases
- `src/eval/runner.rs` — Major rework: orchestrate multiple `claude -p` invocations
- `src/sandbox/environment.rs` — Multi-agent workspace with conflict detection
- `src/sandbox/mailbox.rs` — New file: inter-agent message passing
- `src/eval/evaluator.rs` — Delegation-aware scoring

## Open Questions

1. How do we simulate the Agent tool without actually using Claude's Agent tool (which spawns real subagents with real costs)?
2. Should clawbake optimize the AGENTS.md or the individual agent identities, or both in an alternating fashion?
3. Is there a standard AGENTS.md format emerging, or do we need to define one?
4. How do we handle the combinatorial explosion of multi-agent eval cases?
5. Can we record and replay delegation traces for cheaper re-evaluation?
