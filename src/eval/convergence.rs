use crate::types::TokenBudget;

#[derive(Debug, Clone, PartialEq)]
pub enum StopReason {
    MaxIterations,
    BudgetExhausted,
    ScorePlateau,
    ScoreRegression,
    UserStopped,
    PerfectScore,
}

impl std::fmt::Display for StopReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaxIterations => write!(f, "Maximum iterations reached"),
            Self::BudgetExhausted => write!(f, "Token budget exhausted"),
            Self::ScorePlateau => write!(f, "Score plateau detected"),
            Self::ScoreRegression => write!(f, "Score regression detected"),
            Self::UserStopped => write!(f, "Stopped by user"),
            Self::PerfectScore => write!(f, "Perfect score achieved"),
        }
    }
}

pub struct ConvergenceChecker {
    max_iterations: usize,
    plateau_window: usize,
    plateau_epsilon: f64,
    perfect_threshold: f64,
    regression_threshold: f64,
    best_score: f64,
    score_history: Vec<f64>,
    consecutive_regressions: usize,
    regression_exempt_until: usize,
}

impl ConvergenceChecker {
    pub fn new(max_iterations: usize) -> Self {
        Self {
            max_iterations,
            plateau_window: 3,
            plateau_epsilon: 0.01,
            perfect_threshold: 0.98,
            regression_threshold: 0.10,
            best_score: 0.0,
            score_history: Vec::new(),
            consecutive_regressions: 0,
            regression_exempt_until: 0,
        }
    }

    pub fn record_score(&mut self, score: f64) {
        if !self.score_history.is_empty() && self.best_score - score > self.regression_threshold {
            self.consecutive_regressions += 1;
        } else {
            self.consecutive_regressions = 0;
        }
        self.score_history.push(score);
        if score > self.best_score {
            self.best_score = score;
        }
    }

    /// Exempt an iteration from regression checks (e.g., after case regeneration).
    pub fn exempt_from_regression(&mut self, iteration: usize) {
        self.regression_exempt_until = iteration;
    }

    pub fn check(
        &self,
        iteration: usize,
        budget: &TokenBudget,
        user_stopped: bool,
    ) -> Option<StopReason> {
        if user_stopped {
            return Some(StopReason::UserStopped);
        }

        if iteration >= self.max_iterations {
            return Some(StopReason::MaxIterations);
        }

        if budget.exhausted() {
            return Some(StopReason::BudgetExhausted);
        }

        // Check for perfect score
        if let Some(&last) = self.score_history.last() {
            if last >= self.perfect_threshold {
                return Some(StopReason::PerfectScore);
            }
        }

        // Check for plateau
        if self.score_history.len() >= self.plateau_window {
            let recent = &self.score_history[self.score_history.len() - self.plateau_window..];
            let min = recent.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = recent.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            if (max - min) < self.plateau_epsilon {
                return Some(StopReason::ScorePlateau);
            }
        }

        // Check for regression: require 2 consecutive, skip if exempt
        if self.score_history.len() >= 2
            && iteration > self.regression_exempt_until
            && self.consecutive_regressions >= 2
        {
            return Some(StopReason::ScoreRegression);
        }

        // Preemptive budget check
        if self.score_history.len() >= 2 {
            let avg_tokens_per_iter = budget.consumed / self.score_history.len() as u64;
            if budget.remaining() < avg_tokens_per_iter {
                return Some(StopReason::BudgetExhausted);
            }
        }

        None
    }
}
