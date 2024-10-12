use crate::backtrack::{Backtrack, DecLvl, DecisionLevelTracker};
use crate::model::extensions::AssignmentExt;
use crate::model::Model;
use crate::solver::search::{Decision, SearchControl};
use crate::solver::stats::Stats;

#[derive(Copy, Clone)]
pub enum PreferredValue {
    Min,
    Max,
}

/// Assigns all values in lexical order to their minimal or maximal value.
/// Essentially intended to finish the search once all high-priority variables have been set.
#[derive(Copy, Clone)]
pub struct Lexical {
    pref: PreferredValue,
    lvl: DecisionLevelTracker,
}

impl Lexical {
    pub fn new(preferred_value: PreferredValue) -> Self {
        Lexical {
            pref: preferred_value,
            lvl: Default::default(),
        }
    }

    /// A variant that always assign the minimal value
    pub fn with_min() -> Self {
        Lexical::new(PreferredValue::Min)
    }

    /// A variant that always assigns the maximal value
    pub fn with_max() -> Self {
        Lexical::new(PreferredValue::Max)
    }
}

impl Backtrack for Lexical {
    fn save_state(&mut self) -> DecLvl {
        self.lvl.save_state()
    }

    fn num_saved(&self) -> u32 {
        self.lvl.num_saved()
    }

    fn restore_last(&mut self) {
        self.lvl.restore_last()
    }
}

impl<L> SearchControl<L> for Lexical {
    fn next_decision(&mut self, _stats: &Stats, model: &Model<L>) -> Option<Decision> {
        // set the first domain value of the first unset variable
        model
            .state
            .variables()
            .filter_map(|v| {
                if model.state.present(v) == Some(true) {
                    let dom = model.var_domain(v);
                    if dom.is_bound() {
                        None
                    } else {
                        match self.pref {
                            PreferredValue::Min => Some(Decision::SetLiteral(v.leq(dom.lb))),
                            PreferredValue::Max => Some(Decision::SetLiteral(v.geq(dom.ub))),
                        }
                    }
                } else {
                    None
                }
            })
            .next()
    }

    fn clone_to_box(&self) -> Box<dyn SearchControl<L> + Send> {
        Box::new(*self)
    }
}
