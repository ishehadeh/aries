pub mod brancher;
pub mod sat_solver;
pub mod stats;
pub mod theory_solver;

use crate::backtrack::Backtrack;
use crate::model::lang::{BAtom, IVar, IntCst};
use crate::model::{Model, ModelEvents, WriterId};
use crate::queues::Q;
use crate::{Theory, TheoryResult};
use aries_sat::all::Lit;

use crate::model::assignments::{Assignment, SavedAssignment};
use crate::solver::brancher::{Brancher, Decision};
use crate::solver::sat_solver::{SatPropagationResult, SatSolver};
use crate::solver::stats::Stats;
use crate::solver::theory_solver::TheorySolver;
use std::time::Instant;

pub struct SMTSolver {
    pub model: Model,
    brancher: Brancher,
    sat: SatSolver,
    theories: Vec<TheorySolver>,
    queues: Vec<ModelEvents>,
    num_saved_states: u32,
    pub stats: Stats,
}
impl SMTSolver {
    fn sat_token() -> WriterId {
        WriterId::new(1)
    }
    fn decision_token() -> WriterId {
        WriterId::new(0)
    }
    fn theory_token(theory_num: u8) -> WriterId {
        WriterId::new(2 + theory_num)
    }

    pub fn new(model: Model) -> SMTSolver {
        let sat = SatSolver::new(Self::sat_token(), model.bool_event_reader());
        SMTSolver {
            model,
            brancher: Brancher::new(),
            sat,
            theories: Vec::new(),
            queues: Vec::new(),
            num_saved_states: 0,
            stats: Default::default(),
        }
    }
    pub fn add_theory(&mut self, theory: Box<dyn Theory>) {
        let module = TheorySolver::new(theory);
        self.theories.push(module);
        self.queues.push(self.model.readers());
        self.stats.per_module_propagation_time.push(0.0);
        self.stats.per_module_conflicts.push(0);
        self.stats.per_module_propagation_loops.push(0);
    }

    pub fn enforce(&mut self, constraints: &[BAtom]) {
        let start = Instant::now();
        let mut queue = Q::new();
        let mut reader = queue.reader();

        for atom in constraints {
            match self.sat.enforce(*atom, &mut self.model, &mut queue) {
                EnforceResult::Enforced => (),
                EnforceResult::Reified(l) => queue.push(Binding::new(l, *atom)),
                EnforceResult::Refined => (),
            }
        }

        while let Some(binding) = reader.pop() {
            let var = binding.lit.variable();
            if !self.brancher.is_declared(var) {
                self.brancher.declare(var);
                self.brancher.enqueue(var);
            }
            let mut supported = false;
            let expr = self.model.expressions.expr_of(binding.atom);
            // if the BAtom has not a corresponding expr, then it is a free variable and we can stop.

            if let Some(expr) = expr {
                match self.sat.bind(binding.lit, expr, &mut queue, &mut self.model.bools) {
                    BindingResult::Enforced => supported = true,
                    BindingResult::Unsupported => {}
                    BindingResult::Refined => supported = true,
                }
                for theory in &mut self.theories {
                    match theory.bind(binding.lit, binding.atom, &mut self.model, &mut queue) {
                        BindingResult::Enforced => supported = true,
                        BindingResult::Unsupported => {}
                        BindingResult::Refined => supported = true,
                    }
                }
            }

            assert!(supported, "Unsupported binding")
        }
        self.stats.init_time += start.elapsed().as_secs_f64()
    }

    pub fn solve(&mut self) -> bool {
        let start = Instant::now();
        loop {
            if !self.propagate_and_backtrack_to_consistent() {
                // UNSAT
                self.stats.solve_time += start.elapsed().as_secs_f64();
                return false;
            }
            match self.brancher.next_decision(&self.stats, &self.model) {
                Some(Decision::SetLiteral(lit)) => {
                    self.decide(lit);
                }
                Some(Decision::Restart) => {
                    self.reset();
                    self.stats.num_restarts += 1;
                }
                None => {
                    // SAT: consistent + no choices left
                    self.stats.solve_time += start.elapsed().as_secs_f64();
                    return true;
                }
            }
        }
    }

    pub fn minimize(&mut self, objective: IVar) -> Option<(IntCst, SavedAssignment)> {
        self.minimize_with(objective, |_, _| ())
    }

    pub fn minimize_with(
        &mut self,
        objective: IVar,
        mut on_new_solution: impl FnMut(IntCst, &SavedAssignment),
    ) -> Option<(IntCst, SavedAssignment)> {
        let mut result = None;
        while self.solve() {
            let lb = self.model.lower_bound(objective);

            let sol = SavedAssignment::from_model(&self.model);
            on_new_solution(lb, &sol);
            result = Some((lb, sol));
            self.stats.num_restarts += 1;
            self.reset();
            let improved = self.model.lt(objective, lb);
            self.enforce(&[improved]);
        }
        result
    }

    pub fn decide(&mut self, decision: Lit) {
        self.save_state();
        self.model.bools.set(decision, Self::decision_token());
        self.stats.num_decisions += 1;
    }

    pub fn propagate_and_backtrack_to_consistent(&mut self) -> bool {
        let global_start = Instant::now();
        loop {
            let sat_start = Instant::now();
            let bool_model = &mut self.model.bools;
            self.stats.per_module_propagation_loops[0] += 1;
            let brancher = &mut self.brancher;
            let on_learnt_clause = |clause: &[Lit]| {
                for l in clause {
                    brancher.bump_activity(l.variable());
                }
            };
            match self.sat.propagate(bool_model, on_learnt_clause) {
                SatPropagationResult::Backtracked(n) => {
                    let bt_point = self.num_saved_states - n.get();
                    self.restore(bt_point);
                    self.stats.num_conflicts += 1;
                    self.stats.per_module_conflicts[0] += 1;

                    // skip theory propagations to repeat sat propagation,
                    self.stats.per_module_propagation_time[0] += sat_start.elapsed().as_secs_f64();
                    continue;
                }
                SatPropagationResult::Inferred => (),
                SatPropagationResult::NoOp => (),
                SatPropagationResult::Unsat => {
                    self.stats.propagation_time += global_start.elapsed().as_secs_f64();
                    self.stats.per_module_propagation_time[0] += sat_start.elapsed().as_secs_f64();
                    return false;
                }
            }
            self.stats.per_module_propagation_time[0] += sat_start.elapsed().as_secs_f64();

            let mut contradiction_found = false;
            for i in 0..self.theories.len() {
                let theory_propagation_start = Instant::now();
                self.stats.per_module_propagation_loops[i + 1] += 1;
                debug_assert!(!contradiction_found);
                let th = &mut self.theories[i];
                let queue = &mut self.queues[i];
                match th.process(queue, &mut self.model.writer(Self::theory_token(i as u8))) {
                    TheoryResult::Consistent => {
                        // theory is consistent
                    }
                    TheoryResult::Contradiction(clause) => {
                        // theory contradiction.
                        // learnt a new clause, add it to sat
                        // and skip the rest of the propagation
                        self.sat.sat.add_forgettable_clause(&clause);
                        contradiction_found = true;

                        self.stats.per_module_conflicts[i + 1] += 1;
                        self.stats.per_module_propagation_time[i + 1] +=
                            theory_propagation_start.elapsed().as_secs_f64();
                        break;
                    }
                }
                self.stats.per_module_propagation_time[i + 1] += theory_propagation_start.elapsed().as_secs_f64();
            }
            if !contradiction_found {
                // if we reach this point, no contradiction has been found
                break;
            }
        }
        self.stats.propagation_time += global_start.elapsed().as_secs_f64();
        true
    }
}

impl Backtrack for SMTSolver {
    fn save_state(&mut self) -> u32 {
        self.num_saved_states += 1;
        let n = self.num_saved_states - 1;
        assert_eq!(self.model.save_state(), n);
        assert_eq!(self.brancher.save_state(), n);
        assert_eq!(self.sat.save_state(), n);
        for th in &mut self.theories {
            assert_eq!(th.save_state(), n);
        }
        n
    }

    fn num_saved(&self) -> u32 {
        self.num_saved_states
    }

    fn restore_last(&mut self) {
        assert!(self.num_saved() > 0, "No state to restore");
        let last = self.num_saved() - 1;
        self.restore(last);
        self.num_saved_states -= 1;
    }

    fn restore(&mut self, saved_id: u32) {
        self.num_saved_states = saved_id;
        self.model.restore(saved_id);
        self.brancher.restore(saved_id);
        self.sat.restore(saved_id);
        for th in &mut self.theories {
            th.restore(saved_id);
        }
    }
}

#[derive(Copy, Clone)]
pub struct Binding {
    lit: Lit,
    atom: BAtom,
}
impl Binding {
    pub fn new(lit: Lit, atom: BAtom) -> Binding {
        Binding { lit, atom }
    }
}

pub enum EnforceResult {
    Enforced,
    Reified(Lit),
    Refined,
}

pub enum BindingResult {
    Enforced,
    Unsupported,
    Refined,
}