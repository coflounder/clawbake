use crate::tui::wizard::WizardState;
use crate::tui::dashboard::DashboardState;
use crate::eval::loop_runner::EvalEvent;

#[derive(Debug)]
pub enum AppMode {
    Wizard(WizardState),
    Dashboard(DashboardState),
    Done { reason: String, best_score: f64 },
}

pub struct App {
    pub mode: AppMode,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: AppMode::Wizard(WizardState::new()),
            should_quit: false,
        }
    }

    pub fn new_dashboard(max_iterations: usize, budget_limit: u64) -> Self {
        Self {
            mode: AppMode::Dashboard(DashboardState::new(max_iterations, budget_limit)),
            should_quit: false,
        }
    }

    pub fn handle_eval_event(&mut self, event: EvalEvent) {
        match &mut self.mode {
            AppMode::Dashboard(dashboard) => {
                match event {
                    EvalEvent::IterationStarted { iteration } => {
                        dashboard.current_iteration = iteration;
                        dashboard.phase = if iteration == 0 {
                            "Planning eval cases...".to_string()
                        } else {
                            "Running".to_string()
                        };
                    }
                    EvalEvent::PlanningComplete { case_count } => {
                        dashboard.total_cases = case_count;
                        dashboard.phase = "Planned".to_string();
                    }
                    EvalEvent::CaseStarted { case_id, title } => {
                        dashboard.active_case = Some(format!("{}: {}", case_id, title));
                        dashboard.completed_cases_in_iter += 0; // just mark active
                    }
                    EvalEvent::CaseComplete { case_id: _, score } => {
                        dashboard.completed_cases_in_iter += 1;
                        dashboard.last_scores.push(score);
                    }
                    EvalEvent::EvaluationComplete { iteration: _, average_score } => {
                        dashboard.score_history.push(average_score);
                        dashboard.phase = "Evaluated".to_string();
                        dashboard.active_case = None;
                    }
                    EvalEvent::OptimizationComplete { iteration: _, mutation } => {
                        dashboard.mutations.push(mutation);
                        dashboard.phase = "Optimized".to_string();
                        dashboard.completed_cases_in_iter = 0;
                        dashboard.last_scores.clear();
                    }
                    EvalEvent::BudgetUpdate { consumed, limit } => {
                        dashboard.budget_consumed = consumed;
                        dashboard.budget_limit = limit;
                    }
                    EvalEvent::ConvergenceCheck { stop_reason } => {
                        if let Some(reason) = stop_reason {
                            dashboard.phase = format!("Converging: {}", reason);
                        }
                    }
                    EvalEvent::LoopComplete { reason, best_score } => {
                        self.mode = AppMode::Done { reason, best_score };
                    }
                    EvalEvent::Error { message } => {
                        dashboard.logs.push(format!("[ERR] {}", &message));
                        dashboard.errors.push(message);
                    }
                    EvalEvent::Log { message } => {
                        dashboard.logs.push(message);
                    }
                }
            }
            _ => {}
        }
    }
}
