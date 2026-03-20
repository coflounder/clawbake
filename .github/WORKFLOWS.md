# GitHub Actions Workflows for Clawbake

Clawbake includes two professional-grade CI/CD workflows for continuous evaluation and identity optimization.

## Overview

| Workflow | Trigger | Purpose | Mode(s) | Schedule |
|----------|---------|---------|---------|----------|
| **soul-eval.yml** | Push, Manual | Single-mode focused eval | Any (default: soul) | On-demand |
| **eval-matrix.yml** | Push, Daily | Comprehensive multi-mode test | All 5 modes | Daily @ 2 AM UTC |

## Soul Eval Workflow

**File:** `.github/workflows/soul-eval.yml`

Single eval mode focused on identity optimization and PR feedback.

### Triggers

1. **Automatic (on push)**
   ```
   Push to main with code changes in:
   - src/**
   - .clawbake/**
   - .github/workflows/soul-eval.yml
   - Cargo.toml
   ```

2. **Manual (workflow_dispatch)**
   ```
   GitHub UI → Actions → Soul Eval Mode → Run workflow
   Inputs:
   - mode: soul|claude|agents|memory|skills (default: soul)
   - iterations: max iterations (default: 3)
   ```

### Features

- ✅ Builds clawbake release binary (cached)
- ✅ Authenticates with Claude via official action
- ✅ Runs specified eval mode in headless mode
- ✅ Exports results to structured format
- ✅ Posts PR comments with summary (if on PR)
- ✅ Uploads artifacts (30-day retention)
- ✅ Reports best score and mutations to summary

### Output

**PR Comment (if on pull request):**
```markdown
## ✨ Soul Eval Complete

### Best SOUL.md
```markdown
[First 50 lines of optimized identity]
```

### Scores
```json
{...}
```
```

**Artifacts:**
```
soul-eval-results/
├── eval-output/          # Exported identity files
│   ├── SOUL.md
│   └── scores.json
├── eval-output.log       # Full eval transcript
└── eval-summary.md       # Markdown summary
```

### Example Usage

```bash
# Trigger automatic eval on push
git push origin main

# Or trigger manually with custom settings
# GitHub UI → Actions → Soul Eval Mode
#   mode: claude
#   iterations: 5
```

---

## Eval Matrix Workflow

**File:** `.github/workflows/eval-matrix.yml`

Comprehensive multi-mode testing with matrix strategy.

### Triggers

1. **Automatic (on push)**
   ```
   Push to main or develop
   Same path filters as soul-eval.yml
   ```

2. **Daily schedule**
   ```
   Every day at 2 AM UTC
   Useful for overnight optimization
   ```

3. **Manual (workflow_dispatch)**
   ```
   GitHub UI → Actions → Eval Matrix
   (No inputs, uses config.toml)
   ```

### Matrix Strategy

Runs 5 eval modes in parallel with mode-specific configuration:

```yaml
soul:
  iterations: 5
  timeout: 30 min
  
claude:
  iterations: 3
  timeout: 20 min
  
agents:
  iterations: 3
  timeout: 20 min
  
memory:
  iterations: 2
  timeout: 15 min
  
skills:
  iterations: 2
  timeout: 15 min
```

**Total time:** ~30 minutes (parallel execution)

### Features

- ✅ Parallel matrix: all 5 modes run concurrently
- ✅ Mode-specific timeouts and iteration counts
- ✅ Separate build job (shared by all matrix jobs)
- ✅ Post summary comment to PR
- ✅ Separate artifacts per mode
- ✅ Continue-on-error: one mode failing doesn't block others
- ✅ Final summary job aggregates results

### Output

**PR Comment (if on pull request):**
```markdown
## 🧠 Eval Matrix Results

## soul Mode - Status
**Best Score:** 0.87
**Iterations:** 5
**Status:** ✅ Success

[30 lines of log...]

## claude Mode - Status
...

## agents Mode - Status
...
```

**Artifacts:**
```
eval-soul-results/
eval-claude-results/
eval-agents-results/
eval-memory-results/
eval-skills-results/

Each containing:
├── eval-{mode}.log
├── eval-{mode}-output/
│   └── [exported files]
└── eval-{mode}-summary.md
```

### Example Usage

```bash
# Automatic on push
git push origin main

# Runs daily at 2 AM UTC automatically
# Or manually trigger in GitHub UI
```

---

## Configuration

Both workflows require:

1. **Clawbake Config** (`.clawbake/config.toml`)
   ```toml
   [persona]
   name = "MyAgent"
   role = "..."
   
   [eval]
   eval_count = 3
   max_iterations = 5
   max_budget_tokens = 500000
   
   [mode]
   target = "soul"
   ```

2. **API Secret** (Repository Settings → Secrets)
   ```
   ANTHROPIC_API_KEY = sk-ant-...
   ```

### Example Config

Copy and customize:

```bash
cp .clawbake/example-ci-config.toml .clawbake/config.toml
# Edit .clawbake/config.toml with your agent
git add .clawbake/config.toml
git commit -m "Configure clawbake for CI/CD"
git push origin main
```

---

## Non-Interactive Mode

Both workflows use non-interactive commands:

```bash
# Initialize from config file
clawbake init-config ./config.toml

# Run eval without TUI dashboard
clawbake run --no-wizard --mode soul --headless

# Export results
clawbake export -o ./output
```

This is key to CI/CD: no TTY required, all output goes to stdout for logging.

---

## Workflow Execution Details

### soul-eval.yml Execution Flow

```
1. Checkout code
2. Setup Rust toolchain
3. Cache dependencies (Swatinem/rust-cache)
4. Build release binary (1-2 min)
5. Authenticate with Claude (anthropics/claude-code-action)
6. Validate .clawbake/config.toml exists
7. Run: clawbake run --mode soul --headless --no-wizard
8. Export results: clawbake export -o ./eval-output
9. Create PR comment (if on PR)
10. Upload artifacts
11. Extract & report best score
```

**Typical duration:** 2-5 minutes (depending on eval_count/max_iterations)

### eval-matrix.yml Execution Flow

```
[Build Job]
1. Checkout
2. Setup Rust
3. Build clawbake
4. Upload binary as artifact

[Matrix Jobs] (parallel)
- soul (30 min max)
- claude (20 min max)
- agents (20 min max)
- memory (15 min max)
- skills (15 min max)

Each matrix job:
1. Download clawbake binary
2. Setup Claude API
3. Validate config
4. Run: clawbake run --mode {mode} --headless
5. Export results
6. Upload artifacts

[Summary Job]
1. Download all artifacts
2. Generate consolidated summary
3. Post PR comment (if on PR)
```

**Typical duration:** 30-40 minutes total

---

## Monitoring & Debugging

### View Workflow Runs

```
GitHub UI → Actions → Soul Eval Mode / Eval Matrix
```

Click any run to see:
- Build log
- Per-step output
- Artifacts
- Job summaries

### Check Logs

```bash
# Download artifacts locally
gh run download <run-id> -D ./eval-results

# View eval output
cat eval-results/eval-soul-results/eval-soul.log

# Check best score
cat eval-results/eval-soul-results/eval-soul-output/scores.json
```

### Troubleshooting

**Workflow fails immediately:**
- Check `.clawbake/config.toml` exists
- Validate TOML syntax: `toml-cli validate .clawbake/config.toml`

**Build fails:**
- Check Rust code compiles locally: `cargo build --release`
- Review build log in GitHub UI

**Eval fails:**
- Check `ANTHROPIC_API_KEY` secret is set
- Verify token budget isn't exhausted
- Check eval log for API errors

**Timeout:**
- Reduce `eval_count` or `max_iterations`
- Use cheaper models (haiku vs sonnet)
- Split into separate workflows for different modes

---

## Cost Optimization

To minimize API costs:

1. **Use Haiku for all tiers** (default in example config)
   ```toml
   [models]
   planner = "haiku"
   optimizer = "haiku"
   evaluator = "haiku"
   persona = "haiku"
   stub = "haiku"
   ```

2. **Reduce eval_count** (fewer test cases per iteration)
   ```toml
   [eval]
   eval_count = 2  # Instead of 5
   ```

3. **Reduce max_iterations** (fewer optimization rounds)
   ```toml
   [eval]
   max_iterations = 3  # Instead of 10
   ```

4. **Reduce session_count** (fewer multi-session runs)
   ```toml
   [mode.soul]
   session_count = 2  # Instead of 5
   ```

5. **Use schedule, not on-push** (daily instead of every commit)
   ```yaml
   on:
     schedule:
       - cron: '0 2 * * *'  # Daily, not on every push
   ```

**Estimated cost per full matrix run (haiku):** ~$1-2 USD

---

## Best Practices

### 1. Commit Config to Repo

```bash
git add .clawbake/config.toml
git commit -m "Configure clawbake for evals"
git push origin main
```

This ensures:
- Reproducible evals across CI/CD
- Easy config updates
- Audit trail for persona changes

### 2. Review PR Comments

When a workflow runs on a PR, review the generated comment:
- Check if identity improved
- Look for mutations that make sense
- Approve or request changes before merging

### 3. Archive Artifacts

Periodically download and store artifacts:
```bash
# Monthly archive
gh run list --limit 50 | \
  while read run_id; do
    gh run download $run_id -D ./archive/
  done
```

This preserves evolution history even after 30-day retention.

### 4. Schedule Nightly Evals

Use `schedule` trigger for overnight runs:
```yaml
on:
  schedule:
    - cron: '0 2 * * *'  # 2 AM UTC daily
```

This:
- Doesn't block PR reviews
- Provides fresh identity every morning
- Spreads API cost over time

### 5. Monitor Token Budget

Check logs for budget warnings:
```bash
grep "Budget:" eval-output.log
```

If consistently hitting limits, reduce eval_count or max_iterations.

---

## Advanced: Custom Workflows

Create custom workflows for specific needs:

**Example: Eval only main changes**

```yaml
name: Quick Eval on PR

on:
  pull_request:
    paths:
      - 'src/**'

jobs:
  eval:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      
      - uses: anthropics/claude-code-action@v1
        with:
          api_key: ${{ secrets.ANTHROPIC_API_KEY }}
      
      - name: Quick eval (1 iteration)
        run: |
          ./target/release/clawbake init-config .clawbake/config.toml
          ./target/release/clawbake run \
            --no-wizard \
            --mode soul \
            --headless
      
      - name: Comment results
        # ... comment script ...
```

---

## Integration with Other Tools

### Slack Notifications

Add to workflow:

```yaml
- name: Notify Slack
  if: always()
  uses: slackapi/slack-github-action@v1
  with:
    payload: |
      {
        "text": "Clawbake eval: ${{ job.status }}",
        "blocks": [
          {
            "type": "section",
            "text": {
              "type": "mrkdwn",
              "text": "Soul eval: ${{ env.BEST_SCORE }}"
            }
          }
        ]
      }
  env:
    SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK }}
```

### Discord Notifications

```yaml
- name: Notify Discord
  if: failure()
  run: |
    curl -X POST ${{ secrets.DISCORD_WEBHOOK }} \
      -H 'Content-Type: application/json' \
      -d '{"content":"Clawbake eval failed"}'
```

### Status Checks

Workflows automatically add status checks to PRs:
- Green ✓ if eval succeeds
- Red ✗ if eval fails
- Can be marked as required in branch protection rules

---

## Reference

- [soul-eval.yml]('./workflows/soul-eval.yml') — Full workflow
- [eval-matrix.yml]('./workflows/eval-matrix.yml') — Full workflow
- [CI-CD.md]('./CI-CD.md') — Setup guide
- [Clawbake README](../README.md) — General documentation
