# Eval Mode: Skills/Plugins

> Sub-issue of [00-tracking.md](./00-tracking.md) · Priority: **High**

## What Are Skills/Plugins?

Skills and plugins are executable capabilities bolted onto an agent's base toolset. They range from simple prompt templates (slash commands) to full MCP servers with custom tools. The key examples:

- **obra/superpowers**: Opinionated workflow skills (brainstorming, plan-writing, code review, TDD, etc.)
- **bmad-method**: Structured methodology for agent-driven development
- **Custom slash commands**: Project-specific workflows in `.claude/commands/`
- **MCP servers**: External tool integrations (Notion, Playwright, Linear, etc.)

## Why This Is the Most Undertested Layer

Skills are the agent equivalent of VSCode extensions — everyone installs a dozen, nobody measures their actual impact. The problems:

1. **Context window cost**: Every skill's instructions consume tokens. A skill that adds 2,000 tokens of system prompt but only helps 5% of the time is net-negative for the other 95%.
2. **Interaction effects**: Skill A works great alone. Skill B works great alone. A+B together produce conflicting instructions.
3. **No marginal value measurement**: Does obra/superpowers actually improve output quality vs. vanilla Claude Code? Nobody has data.
4. **Trigger accuracy**: Skills fire on patterns ("use this before any creative work"). False positives waste budget. False negatives miss opportunities.
5. **The kitchen sink problem**: Always-on agents tend to accumulate skills over time. At what point does the skill stack become a liability?

## Proposed Architecture

### CLI Integration

```
clawbake run --mode skills --skills-dir .claude/commands/ [--ab-test] [--ablation]
```

### What Gets Evaluated

The **marginal impact of each skill** on agent performance. This isn't about optimizing skill content (that's the skill author's job). It's about answering:
- Which skills help?
- Which skills hurt?
- What's the optimal skill loadout for this agent's workload?

### Eval Case Categories

| Category | What it tests | Example |
|----------|---------------|---------|
| `skill_vs_vanilla` | Does the skill improve output vs. no skill? | Run same task with and without brainstorming skill |
| `trigger_precision` | Does the skill fire when it should? | Present 20 tasks — skill should activate for 5 |
| `trigger_recall` | Does the skill fire when it should? | Present tasks where skill would help — does it trigger? |
| `context_cost` | Token overhead vs. quality improvement | Measure tokens consumed by skill instructions |
| `interaction_effects` | Skills playing well together | Enable pairs/groups of skills, measure combined impact |
| `workload_match` | Skills match the agent's actual task distribution | Profile real task mix, measure skill relevance |

### Scoring Weights

| Dimension | Weight | Rationale |
|-----------|--------|-----------|
| Task Quality Delta | **0.40** | Quality with skill minus quality without |
| Efficiency Delta | **0.30** | Token/turn cost with skill vs. without |
| Trigger Accuracy | 0.20 | Precision and recall of skill activation |
| Context Cost | 0.10 | Raw token footprint of skill instructions |

Note: All scores are **relative** (delta from baseline). A skill that adds 0.0 task quality at 500 token cost is net-negative.

### Evaluation Strategy

#### A/B Testing (Primary)

For each skill, run the same eval cases twice:
1. **Control**: Agent without the skill
2. **Treatment**: Agent with the skill

Compare scores. The delta is the skill's marginal value.

```
Skill: superpowers:brainstorming
  Control score:   0.72 (task quality)
  Treatment score: 0.81 (task quality)
  Delta:           +0.09
  Token overhead:  +1,200 tokens/invocation
  Verdict:         ✅ KEEP — meaningful quality improvement
```

```
Skill: obscure-formatting-plugin
  Control score:   0.75 (task quality)
  Treatment score: 0.74 (task quality)
  Delta:           -0.01
  Token overhead:  +800 tokens/invocation
  Verdict:         ❌ REMOVE — no benefit, wasting context
```

#### Ablation Testing (Secondary)

Start with all skills enabled. Remove one at a time. Measure impact.

This catches interaction effects that A/B testing misses: a skill that's useless alone but synergizes with another skill.

#### Loadout Optimization (Advanced)

Given N skills and a task distribution, find the optimal subset. This is combinatorial, so we use:
1. Individual A/B scores to rank skills
2. Greedy addition: start with best skill, add next-best if it improves combined score
3. Prune: remove any skill that doesn't improve the ensemble

### Sandbox Requirements

- Must be able to **inject and remove skills** between runs
- Need to measure token usage precisely (skills cost = total tokens with skill - total tokens without)
- Must support both slash command skills (`.claude/commands/`) and MCP-style skills
- Need a representative **task workload** that reflects the agent's real usage patterns

## Implementation Sketch

### Config Changes

```toml
[mode]
target = "skills"

[mode.skills]
skills_dir = ".claude/commands/"
mcp_configs = [".claude/mcp.json"]
ab_test = true
ablation = true
loadout_optimization = false   # Expensive — opt-in

# Define the agent's typical task distribution for workload matching
[[mode.skills.workload]]
category = "code_writing"
frequency = 0.40

[[mode.skills.workload]]
category = "code_review"
frequency = 0.20

[[mode.skills.workload]]
category = "debugging"
frequency = 0.25

[[mode.skills.workload]]
category = "documentation"
frequency = 0.15
```

### New Types

```rust
pub struct SkillSpec {
    pub name: String,
    pub source: SkillSource,
    pub trigger_pattern: Option<String>,  // When should this skill fire?
    pub token_footprint: usize,           // Tokens added to context
}

pub enum SkillSource {
    SlashCommand { path: PathBuf },
    McpServer { name: String, config: serde_json::Value },
    SystemPromptSnippet { content: String },
}

pub struct SkillEvalResult {
    pub skill: String,
    pub control_scores: Vec<EvalScore>,
    pub treatment_scores: Vec<EvalScore>,
    pub quality_delta: f64,
    pub efficiency_delta: f64,
    pub token_overhead: i64,
    pub trigger_precision: f64,
    pub trigger_recall: f64,
    pub verdict: SkillVerdict,
}

pub enum SkillVerdict {
    Keep { reason: String },
    Remove { reason: String },
    Conditional { condition: String },  // "Keep for code review tasks only"
}

pub struct LoadoutRecommendation {
    pub enabled_skills: Vec<String>,
    pub disabled_skills: Vec<String>,
    pub estimated_quality_improvement: f64,
    pub estimated_token_savings: i64,
}
```

### New Files

- `src/eval/skill_scanner.rs` — Discover and parse skills from directories and MCP configs
- `src/eval/skill_ab.rs` — A/B test runner for individual skills
- `src/eval/skill_ablation.rs` — Full-stack ablation testing
- `src/eval/skill_loadout.rs` — Combinatorial loadout optimization
- `src/eval/skill_trigger.rs` — Trigger precision/recall measurement

### Key Files to Modify

- `src/cli.rs` — Add `--skills-dir`, `--ab-test`, `--ablation` flags
- `src/types.rs` — All new types above
- `src/eval/planner.rs` — Generate workload-representative eval cases
- `src/eval/runner.rs` — Support control/treatment paired runs
- `src/sandbox/environment.rs` — Skill injection/removal between runs

## The Killer Feature: Skill Report Card

```
🔌 Skills Evaluation Report
═══════════════════════════════════════════════

Skills evaluated: 8
Task workload: 40% coding, 25% debugging, 20% review, 15% docs

Results (ranked by net impact):
  1. ✅ superpowers:brainstorming      +0.09 quality  +1,200 tokens  NET: +0.07
  2. ✅ superpowers:tdd                +0.07 quality    +900 tokens  NET: +0.06
  3. ✅ code-review:code-review        +0.05 quality    +600 tokens  NET: +0.04
  4. ⚡ superpowers:systematic-debug   +0.03 quality  +1,100 tokens  NET: +0.01
  5. ⚠️  superpowers:writing-plans      +0.02 quality  +1,500 tokens  NET: -0.01
  6. ❌ custom:format-checker          -0.01 quality    +800 tokens  NET: -0.02
  7. ❌ custom:verbose-logger          -0.03 quality  +2,000 tokens  NET: -0.05
  8. ❌ bmad-method:full-stack         +0.01 quality  +4,200 tokens  NET: -0.06

Recommended loadout: Enable #1-4, disable #5-8
  Estimated improvement: +0.18 quality, -8,500 tokens saved

Interaction effects detected:
  ⚠️  brainstorming + writing-plans overlap significantly (r=0.82)
     → Keep brainstorming (higher individual impact), drop writing-plans
```

## Open Questions

1. How do we handle skills that are only valuable for specific task types? Per-task-category scoring?
2. A/B testing doubles the eval cost. Can we do cheaper proxy measurements?
3. How do we evaluate MCP tools that have side effects (Notion writes, Playwright browser sessions)?
4. Should clawbake recommend skills the user *doesn't have* based on task distribution gaps?
5. How do we handle skill versioning? A skill that was net-negative v1 might be great in v2.
6. Can we build a community benchmark dataset of skill effectiveness across different agent profiles?
