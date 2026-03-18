# Clawbake

**Generate, evaluate, and iteratively improve identity-based system prompts for AI agents.**

Clawbake is a TUI application that takes a persona definition through a guided wizard, then runs an automated eval loop to produce a high-quality system prompt markdown file. All LLM calls are routed through `claude -p` (Claude Code CLI in headless mode) with tiered model selection.

## How It Works

```
Wizard (5 steps)  -->  Eval Loop  -->  Optimized Identity (.md)
```

1. **Define** your agent's role, personality, tools, and guardrails via an interactive wizard
2. **Plan** diverse eval cases (core tasks, personality probes, edge cases, guardrail tests, tool usage)
3. **Run** the agent against each case in a sandboxed environment with stub tools
4. **Evaluate** transcripts across persona fidelity, task quality, and efficiency
5. **Optimize** the identity document based on scores, then repeat until convergence

The live dashboard shows scores, budget usage, and mutation history in real time:

```
┌─ Iteration 3/10 ─ Running ─────────────────────────────┐
│ ┌─ Scores ─────────────────┐ ┌─ Budget ──────────────┐ │
│ │ Fidelity: ▁▃▅▇ 0.82     │ │ ████████░░ 78%        │ │
│ │ Quality:  ▁▄▆▇ 0.85     │ │ 780K / 1M tokens      │ │
│ │ Efficiency: ▂▃▅▆ 0.71   │ └───────────────────────┘ │
│ │ Overall:  ▁▃▅▇ 0.79     │                           │
│ └──────────────────────────┘                           │
│ ┌─ Mutation Log ──────────────────────────────────────┐ │
│ │ iter 2: Strengthened tool selection heuristics      │ │
│ │ iter 1: Added explicit personality voice markers    │ │
│ └─────────────────────────────────────────────────────┘ │
│ [q] Quit  [Tab] Focus  [s] Stop                        │
└─────────────────────────────────────────────────────────┘
```

## Prerequisites

- [Rust](https://rustup.rs/) 1.85+
- [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) (`claude`) installed and authenticated

## Installation

### From source

```bash
git clone https://github.com/your-username/clawbake.git
cd clawbake
cargo install --path .
```

### Build only

```bash
cargo build --release
# Binary at target/release/clawbake
```

## Usage

### Initialize a project

```bash
clawbake init
```

Launches the setup wizard to define your agent persona:

| Step | Description |
|------|-------------|
| **1. Role** | What the agent does and its core responsibility |
| **2. Personality** | Pick from 9 built-in traits or add custom ones |
| **3. Tools** | Define tools the agent can use (stub scripts are auto-generated) |
| **4. Name** | Give the agent a name |
| **5. Config** | Set eval count, iteration limit, token budget, guardrails, and reference material |

Configuration is saved to `.clawbake/config.toml`.

### Run the eval loop

```bash
clawbake run
```

Starts the eval-optimize loop with a live TUI dashboard. The loop stops when any convergence condition is met:

- Maximum iterations reached
- Token budget exhausted
- Score plateau detected (window of 3, epsilon 0.01)
- Perfect score achieved (>= 0.98)
- Manually stopped with `s`

To skip the wizard and reuse existing config:

```bash
clawbake run --no-wizard
```

### Check status

```bash
clawbake status
```

Shows the current best score, iteration count, tokens consumed, and last mutation.

### Export the result

```bash
clawbake export -o ./output
```

Writes the best identity document to the specified directory. Supports single-file (`IDENTITY.md`) or multi-file output split by section.

### Options

```
clawbake [OPTIONS] <COMMAND>

Commands:
  init    Initialize a new project with the setup wizard
  run     Run the eval loop to optimize the identity
  status  Show current status and best score
  export  Export the best identity to a directory

Options:
  -d, --dir <DIR>  Working directory (defaults to current)
  -h, --help       Print help
  -V, --version    Print version
```

## Architecture

```
src/
├── main.rs, cli.rs, config.rs      Core entry, CLI, TOML config
├── types.rs, error.rs               Shared types, error handling
├── claude/                          Claude Code CLI wrapper
│   ├── client.rs                    Builder-pattern invocations with budget tracking
│   └── models.rs                    Tier enum, response parsing
├── eval/                            Eval loop engine
│   ├── planner.rs                   Generate eval cases
│   ├── runner.rs                    Execute cases in parallel
│   ├── evaluator.rs                 Score transcripts
│   ├── optimizer.rs                 Mutate the identity
│   ├── convergence.rs               Stop condition detection
│   └── loop_runner.rs               Orchestrator
├── sandbox/                         Tool simulation
│   ├── stubs.rs                     Generate stub shell scripts
│   └── environment.rs               Sandboxed PATH + tempdir
├── tui/                             Terminal UI (ratatui)
│   ├── wizard/                      5-step setup wizard
│   └── dashboard/                   Live progress display
├── io/                              State directory management
│   ├── state.rs                     .clawbake/ layout
│   ├── identity.rs                  Identity doc generation
│   └── history.rs                   Run history persistence
└── export.rs                        Identity file export
```

### Model tiers

Clawbake uses tiered model selection to balance cost and capability:

| Tier | Default Model | Purpose |
|------|---------------|---------|
| Planner | `sonnet` | Generate eval cases |
| Optimizer | `sonnet` | Mutate the identity document |
| Evaluator | `haiku` | Score transcripts |
| Persona | `haiku` | Run as the agent under test |
| Stub | `haiku` | Simulate tool outputs |

Tiers are configurable in `.clawbake/config.toml` under `[models]`.

### State directory

```
.clawbake/
├── config.toml          Project configuration
├── reference.md         Reference material for the eval loop
├── evals/cases.json     Generated eval cases
├── runs/{iteration}/    Per-iteration scores, transcripts, identity
├── best/identity.md     Best-scoring identity
└── history.json         Full run history
```

## Configuration

All settings live in `.clawbake/config.toml`:

```toml
[persona]
name = "CodeBot"
role = "Senior code reviewer"
responsibility = "Review pull requests for correctness and style"
personality_traits = ["Analytical", "Direct"]
guardrails = ["Never approve code with security vulnerabilities"]

[eval]
eval_count = 5           # Cases per iteration
max_iterations = 10      # Max optimization rounds
max_budget_tokens = 1000000  # Token budget across all tiers
max_parallel = 2         # Concurrent eval case runs

[models]
planner = "sonnet"
optimizer = "sonnet"
evaluator = "haiku"
persona = "haiku"
stub = "haiku"

[output]
format = "single"        # "single" or "multi"
workspace_files = ["IDENTITY.md"]
```

## Scoring

Each eval case is scored on three dimensions (0.0 - 1.0):

| Dimension | Weight | Description |
|-----------|--------|-------------|
| Persona Fidelity | 40% | Does the response match the defined personality, role, and guardrails? |
| Task Quality | 40% | How well does the response accomplish the task? |
| Efficiency | 20% | Is the response concise and focused? |

The **overall score** is the weighted composite. The optimizer targets the lowest-scoring dimensions each iteration.

## Contributing

Contributions are welcome. Please open an issue to discuss your idea before submitting a pull request.

```bash
# Development workflow
cargo build              # Build
cargo run -- init        # Test the wizard
cargo run -- status      # Check project state
RUST_LOG=debug cargo run -- run  # Run with debug logging
```
