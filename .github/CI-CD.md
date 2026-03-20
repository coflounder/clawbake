# Clawbake CI/CD Setup Guide

This guide explains how to set up and run Clawbake evaluations in GitHub Actions.

## Prerequisites

1. **GitHub Repository** with Clawbake checked out
2. **Anthropic API Key** — set as `ANTHROPIC_API_KEY` secret in repo settings
3. **Config File** — `.clawbake/config.toml` in your repo (or use `init-config` to create one)

## Quick Start (3 Steps)

### 1. Create Your Config

Copy the example and customize:

```bash
cp .clawbake/example-ci-config.toml .clawbake/my-config.toml
# Edit .clawbake/my-config.toml with your agent details
```

Or use the interactive wizard locally:

```bash
clawbake init
```

### 2. Set Up API Secret

In your GitHub repo settings → Secrets and variables → Actions:

```
ANTHROPIC_API_KEY = sk-ant-...
```

### 3. Trigger Workflow

Push to main or manually trigger:

```bash
git push origin main  # Triggers on-push workflow
```

Or use workflow_dispatch in GitHub UI.

## Workflows

### `soul-eval.yml` — Single Mode, Soul Focus

**Triggers:**
- Push to `main` with code changes
- Manual trigger (`workflow_dispatch`)
- Can specify mode and iterations

**Features:**
- Builds clawbake binary
- Runs soul eval mode (or specified mode)
- Exports results
- Comments on PR with summary
- Uploads artifacts (30-day retention)

**Usage:**

```yaml
# Automatic on push to main
git push origin main

# Or trigger manually in GitHub UI:
# Actions → Soul Eval Mode → Run workflow
#   - mode: soul (or claude/agents/memory/skills)
#   - iterations: 5
```

### `eval-matrix.yml` — All Modes

**Triggers:**
- Push to `main` or `develop`
- Daily schedule (2 AM UTC)
- Manual trigger

**Features:**
- Matrix strategy: tests all 5 eval modes in parallel
- Configurable timeout per mode
- Automatic PR comments with results
- Collects results into summary report

**Matrix:**
| Mode | Iterations | Timeout |
|------|-----------|---------|
| soul | 5 | 30 min |
| claude | 3 | 20 min |
| agents | 3 | 20 min |
| memory | 2 | 15 min |
| skills | 2 | 15 min |

## Non-Interactive Mode

For CI/CD, use the non-interactive commands:

### Option 1: Pre-existing Config

```bash
clawbake init-config .clawbake/config.toml
clawbake run --no-wizard --mode soul --headless
```

### Option 2: Create Config, Then Run

```bash
# In your CI script:
clawbake init-config .clawbake/my-generated-config.toml
clawbake run --no-wizard --mode soul --headless
```

### Option 3: Override at Runtime

```bash
clawbake run --no-wizard --mode soul --hold path/to/CONTEXT.md --headless
```

## Environment Variables

The Claude Code Action sets up the environment automatically. If running Clawbake manually:

```bash
export ANTHROPIC_API_KEY=sk-ant-...
export CLAUDE_HOME=~/.claude
```

## Outputs

Each workflow produces:

1. **Artifacts** (uploaded for 30 days)
   - `eval-{mode}-output/` — Exported identity files
   - `eval-{mode}.log` — Full eval log
   - `eval-{mode}-summary.md` — Markdown summary

2. **PR Comments** (on pull requests)
   - Best identity preview
   - Scores and metrics
   - Status summary

3. **Step Summary** (visible in GitHub Actions UI)
   - Best score
   - Iteration count
   - Last 50 lines of log

## Example: Soul Eval on Every Push

Add to `.clawbake/config.toml`:

```toml
[persona]
name = "MyAgent"
role = "AI assistant for code review"
responsibility = "Review PRs for quality and correctness"
personality_traits = ["Thorough", "Fair", "Technical"]
guardrails = ["Never approve code with security issues"]

[mode]
target = "soul"

[mode.soul]
session_count = 3
consistency_threshold = 0.85
```

Commit and push:

```bash
git add .clawbake/config.toml
git commit -m "Configure soul eval for MyAgent"
git push origin main
```

GitHub Actions will:
1. Build clawbake
2. Run soul eval (3 sessions, max 5 iterations)
3. Export improved SOUL.md
4. Post results to PR (if open)
5. Save artifacts for review

## Troubleshooting

### "No config found"

Make sure `.clawbake/config.toml` exists in your repo or use `init-config`:

```bash
clawbake init-config .clawbake/config.toml
```

### "API key not set"

Check GitHub repo settings → Secrets → `ANTHROPIC_API_KEY` is set.

### "Eval timed out"

Reduce `max_iterations` or `eval_count` in config.toml:

```toml
[eval]
eval_count = 2        # Lower number of cases
max_iterations = 3    # Lower max iterations
```

### "Out of tokens"

Reduce `max_budget_tokens` or use cheaper models (haiku vs sonnet):

```toml
[models]
planner = "haiku"
optimizer = "haiku"
evaluator = "haiku"
```

## Advanced: Custom Workflow

Create `.github/workflows/custom-eval.yml`:

```yaml
name: Custom Eval

on:
  push:
    branches: [main]

jobs:
  custom:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      
      - uses: anthropics/claude-code-action@v1
        with:
          api_key: ${{ secrets.ANTHROPIC_API_KEY }}
      
      - name: Custom eval
        run: |
          ./target/release/clawbake init-config custom-config.toml
          ./target/release/clawbake run \
            --no-wizard \
            --mode soul \
            --hold extra-context.md \
            --headless
      
      - name: Check results
        run: |
          cat .clawbake/best/scores.json
```

## Best Practices

1. **Config in Repo** — Commit `.clawbake/config.toml` so evals are reproducible
2. **Review PRs** — Always check artifacts before merging eval improvements
3. **Monitor Budget** — Watch `eval-output.log` for token usage
4. **Use Haiku** — For CI, use cheaper models (haiku) to save costs
5. **Schedule Nightly** — Use `schedule` for daily evals without blocking CI/CD
6. **Archive Results** — Download artifacts periodically to track evolution

## See Also

- [Workflows](./workflows/) — Full workflow definitions
- [Config](../README.md#configuration) — Configuration reference
- [Non-Interactive Mode](#non-interactive-mode) — Command reference
