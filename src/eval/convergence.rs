use crate::types::TokenBudget;

#[derive(Debug, Clone, PartialEq)]
pub enum StopReason {
    MaxIterations,
    BudgetExhausted,
    ScorePlateau,
    UserStopped,
    PerfectScore,
}

impl std::fmt::Display for StopReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaxIterations => write!(f, "Maximum iterations reached"),
            Self::BudgetExhausted => write!(f, "Token budget exhausted"),
            Self::ScorePlateau => write!(f, "Score plateau detected"),
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
    score_history: Vec<f64>,
}

impl ConvergenceChecker {
    pub fn new(max_iterations: usize) -> Self {
        Self {
            max_iterations,
            plateau_window: 3,
            plateau_epsilon: 0.01,
            perfect_threshold: 0.98,
            score_history: Vec::new(),
        }
    }

    pub fn record_score(&mut self, score: f64) {
        self.score_history.push(score);
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
