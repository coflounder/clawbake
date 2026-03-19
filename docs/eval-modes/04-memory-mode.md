# Eval Mode: MEMORY.md

> Sub-issue of [00-tracking.md](./00-tracking.md) · Priority: **Critical**

## What is MEMORY.md?

MEMORY.md is persistent, accumulated state — learned user preferences, codebase facts, past decisions, session summaries. Unlike other context files which are authored once and tuned occasionally, MEMORY.md **grows**. It's the difference between a fresh agent and one that "knows" your project.

Used by: nanobot, hermes-agent, any always-on agent that persists between sessions.

## Why This Is the Most Undervalued Layer

**No tooling exists to evaluate memory quality.** Consider:

- **SOUL.md** has clear authoring patterns and a small surface area. You can read it and judge it.
- **CLAUDE.md** gets tested implicitly every time you use Claude Code. You notice bad instructions fast.
- **MEMORY.md** degrades silently.** Bad memories don't cause errors — they cause subtly worse decisions, unnecessary context window usage, and stale assumptions that compound over weeks.

The failure mode is insidious: an agent with 200 memory entries performs worse than one with 50 curated entries, but nobody has the tooling to identify which 150 to cut.

## The Unique Challenge: Memory Is Not Static

Every other eval mode optimizes a static document. Memory mode must evaluate a **dynamic, growing artifact**. This means:

1. **Authorship is shared**: The human adds some entries, the agent adds some, automated systems add some.
2. **Staleness is the enemy**: A memory entry that was true 3 months ago may be false now.
3. **Redundancy accumulates**: Multiple entries saying the same thing in different words.
4. **Ordering matters**: Recent entries should generally outweigh old ones, but not always.
5. **Relevance is contextual**: An entry about "the auth system" is critical for auth tasks and noise for CSS tasks.

## Proposed Architecture

### CLI Integration

```
clawbake run --mode memory --memory-file ./MEMORY.md [--task-sample 20]
```

### What Gets Optimized

The MEMORY.md file. But unlike other modes, optimization here means **pruning, restructuring, and deduplication** — not just rewriting.

### Eval Case Categories

| Category | What it tests | Example |
|----------|---------------|---------|
| `recall_accuracy` | Agent correctly uses stored facts | Memory says "DB is Postgres" — does agent use pg syntax? |
| `staleness_detection` | Agent doesn't rely on outdated info | Memory says "using React 17" but package.json says 18 |
| `preference_honoring` | Agent follows learned preferences | Memory says "user prefers verbose commit messages" |
| `noise_resilience` | Agent performs well despite irrelevant entries | Add 100 irrelevant memories — does quality drop? |
| `contradiction_handling` | Agent navigates conflicting memories | Old entry says X, new entry says Y — which wins? |
| `context_window_pressure` | Memory doesn't crowd out task context | Large MEMORY.md leaves room for actual work? |
| `absence_detection` | Agent recognizes when memory lacks needed info | Task requires knowledge not in memory — does agent ask or hallucinate? |

### Scoring Weights

| Dimension | Weight | Rationale |
|-----------|--------|-----------|
| Recall Accuracy | **0.30** | Core value proposition of memory |
| Task Quality | 0.25 | Memory should improve, not degrade, output |
| Efficiency | **0.25** | Bloated memory = wasted tokens = slower agent |
| Freshness | **0.20** | New dimension: are stale entries detected and handled? |

### Optimization Strategy

This is fundamentally different from other modes:

#### Phase 1: Diagnostic
- **Entry-level impact scoring**: For each memory entry, measure task quality with and without it.
- **Redundancy clustering**: Identify groups of entries that convey the same information.
- **Staleness audit**: Cross-reference memory claims against current codebase state.

#### Phase 2: Surgical Optimization
- **Prune**: Remove entries with zero or negative impact.
- **Merge**: Combine redundant entries into single, clearer entries.
- **Refresh**: Update stale entries with current facts.
- **Restructure**: Organize entries by topic/relevance for better retrieval.
- **Compress**: Rewrite verbose entries to convey the same info in fewer tokens.

#### Phase 3: Growth Policy
- **Suggest memory hygiene rules**: "Review entries older than 30 days", "Max 100 entries", etc.
- **Generate a .clawbake/memory-policy.toml** that always-on agents can use to self-prune.

### Sandbox Requirements

- Need a **realistic MEMORY.md** with a mix of fresh, stale, redundant, and irrelevant entries.
- Must be able to inject the memory as either system prompt context or as a file the agent reads.
- Need access to a codebase to validate factual claims in memory entries.

## Implementation Sketch

### Config Changes

```toml
[mode]
target = "memory"

[mode.memory]
memory_file = "MEMORY.md"
project_dir = "/path/to/repo"        # For staleness validation
entry_delimiter = "## "               # How to split memory into entries
max_entries = 200                      # Warn if exceeded
staleness_window_days = 30
enable_ablation = true                 # Per-entry impact testing
enable_compression = true              # Rewrite verbose entries

[mode.memory.growth_policy]
max_entries = 100
max_tokens = 8000
review_interval_days = 14
auto_prune_zero_impact = true
```

### New Types

```rust
pub struct MemoryEntry {
    pub id: usize,
    pub content: String,
    pub tokens: usize,
    pub category: MemoryCategory,
    pub created: Option<NaiveDate>,
    pub impact_score: Option<f64>,      // Set after ablation
    pub staleness: Option<StalenessStatus>,
    pub redundancy_cluster: Option<usize>,
}

pub enum MemoryCategory {
    Fact,           // "The database is Postgres 15"
    Preference,     // "User prefers snake_case"
    Decision,       // "We chose X over Y because Z"
    Procedure,      // "To deploy, run X then Y"
    SessionSummary, // "On 2026-01-15, we refactored auth"
    Unknown,
}

pub enum StalenessStatus {
    Current,                    // Verified against codebase
    Stale { evidence: String }, // Contradicted by current state
    Unverifiable,               // Can't check (opinion, preference)
}

pub struct MemoryDiagnostic {
    pub total_entries: usize,
    pub total_tokens: usize,
    pub zero_impact_entries: Vec<usize>,
    pub negative_impact_entries: Vec<usize>,
    pub stale_entries: Vec<usize>,
    pub redundancy_clusters: Vec<Vec<usize>>,
    pub compression_candidates: Vec<(usize, usize)>, // (entry_id, potential_token_savings)
}

pub struct MemoryPolicy {
    pub max_entries: usize,
    pub max_tokens: usize,
    pub review_interval_days: u32,
    pub auto_prune: bool,
    pub category_quotas: HashMap<MemoryCategory, usize>,
}
```

### New Files

- `src/eval/memory_analyzer.rs` — Parse MEMORY.md into entries, classify, cluster
- `src/eval/memory_ablation.rs` — Per-entry impact testing
- `src/eval/memory_staleness.rs` — Cross-reference entries against codebase
- `src/eval/memory_compressor.rs` — Rewrite entries for token efficiency
- `src/io/memory_policy.rs` — Generate and read memory-policy.toml

### Key Files to Modify

- `src/cli.rs` — Add `--memory-file` flag
- `src/types.rs` — All new types above
- `src/eval/planner.rs` — Generate memory-specific eval cases
- `src/eval/evaluator.rs` — Add freshness scoring dimension
- `src/eval/optimizer.rs` — Prune/merge/compress instead of rewrite

## The Killer Feature: Memory Report Card

After a memory mode run, clawbake should produce a human-readable report:

```
📊 MEMORY.md Diagnostic Report
═══════════════════════════════════════════════

Entries: 147 (23,400 tokens)
  ✅ High-impact:    43  (29%)
  ⚡ Moderate-impact: 51  (35%)
  ⚠️  Zero-impact:    38  (26%)  ← safe to remove
  ❌ Negative-impact: 15  (10%)  ← actively hurting performance

Staleness:
  🟢 Current:       89  (61%)
  🟡 Unverifiable:  31  (21%)
  🔴 Stale:         27  (18%)  ← contradicted by codebase

Redundancy: 12 clusters found (34 entries → could be 12)

Recommended actions:
  1. Remove 15 negative-impact entries     → save 2,100 tokens
  2. Prune 38 zero-impact entries          → save 5,800 tokens
  3. Merge 12 redundancy clusters          → save 3,200 tokens
  4. Refresh 27 stale entries              → improve accuracy
  Total potential savings: 11,100 tokens (47% reduction)
```

## Open Questions

1. How do we handle memory files that aren't Markdown? Some agents use JSON, YAML, or custom formats.
2. Should clawbake manage memory growth over time (daemon mode) or just snapshot-evaluate?
3. Can we build a "memory importance predictor" that pre-scores entries without full ablation (which is expensive)?
4. How do we handle memories that are only valuable in combination (A alone = 0, B alone = 0, A+B = high)?
5. Should the memory policy be enforced by clawbake at write-time, or just recommended?
