use anyhow::*;
use aries_backtrack::{Backtrack, DecLvl};
use aries_core::*;
use aries_model::extensions::{AssignmentExt, Shaped};
use aries_model::lang::expr::leq;
use aries_model::lang::IVar;
use aries_scheduling::*;
use aries_solver::solver::search::activity::ActivityBrancher;
use aries_solver::solver::search::{Decision, SearchControl};
use aries_solver::solver::stats::Stats;
use aries_stn::theory::{StnConfig, StnTheory};
use std::fmt::Write;
use std::fs;
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(Clone, Debug)]
struct OpenShop {
    pub num_jobs: usize,
    pub num_machines: usize,
    times: Vec<i32>,
}

impl OpenShop {
    pub fn duration(&self, job: usize, op: usize) -> i32 {
        self.times[job * self.num_machines + op]
    }
    pub fn machines(&self) -> impl Iterator<Item = usize> {
        0..self.num_machines
    }
    pub fn jobs(&self) -> impl Iterator<Item = usize> {
        0..self.num_jobs
    }

    /// Computes a lower bound on the makespan as the maximum of the operation durations in each
    /// job and on each machine.
    pub fn makespan_lower_bound(&self) -> i32 {
        let max_by_jobs: i32 = (0..self.num_jobs)
            .map(|job| (0..self.num_machines).map(|task| self.duration(job, task)).sum::<i32>())
            .max()
            .unwrap();

        let max_by_machine: i32 = (0..self.num_machines)
            .map(|m| (0..self.num_jobs).map(|job| self.duration(job, m)).sum())
            .max()
            .unwrap();

        max_by_jobs.max(max_by_machine)
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "jobshop")]
struct Opt {
    /// File containing the jobshop instance to solve.
    file: String,
    /// Output file to write the solution
    #[structopt(long = "output", short = "o")]
    output: Option<String>,
    /// When set, the solver will fail if the found solution does not have this makespan.
    #[structopt(long = "expected-makespan")]
    expected_makespan: Option<u32>,
    #[structopt(long = "lower-bound", default_value = "0")]
    lower_bound: u32,
    #[structopt(long = "upper-bound", default_value = "100000")]
    upper_bound: u32,
    /// Search strategy to use: [activity, est, parallel]
    #[structopt(long = "search", default_value = "parallel")]
    search: SearchStrategy,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let file = &opt.file;
    if std::fs::metadata(file)?.is_file() {
        solve(&opt.file, &opt);
        Ok(())
    } else {
        for entry in WalkDir::new(file).follow_links(true).into_iter().filter_map(|e| e.ok()) {
            let f_name = entry.file_name().to_string_lossy();
            if f_name.ends_with(".txt") {
                println!("{}", f_name);
                solve(&entry.path().to_string_lossy(), &opt);
            }
        }
        Ok(())
    }
}

fn solve(filename: &str, opt: &Opt) {
    let start_time = std::time::Instant::now();
    let filecontent = fs::read_to_string(filename).expect("Cannot read file");

    let pb = parse(&filecontent);

    println!("{:?}", pb);

    let lower_bound = (opt.lower_bound).max(pb.makespan_lower_bound() as u32);
    println!("Initial lower bound: {}", lower_bound);

    let model = encode(&pb, lower_bound, opt.upper_bound);
    let makespan: IVar = IVar::new(model.shape.get_variable(&Var::Makespan).unwrap());

    let mut solver = Solver::new(model);
    solver.add_theory(|tok| StnTheory::new(tok, StnConfig::default()));

    let est_brancher = EstBrancher {
        pb: pb.clone(),
        saved: DecLvl::ROOT,
    };
    let mut solver = get_solver(solver, opt.search, est_brancher);

    let result = solver
        .minimize_with(makespan, |assignment| {
            println!("New solution with makespan: {}", assignment.var_domain(makespan).lb)
        })
        .unwrap();

    if let Some((optimum, solution)) = result {
        println!("Found optimal solution with makespan: {}", optimum);
        assert_eq!(solution.var_domain(makespan).lb, optimum);

        // Format the solution in resource order : each machine is given an ordered list of tasks to process.
        let mut formatted_solution = String::new();
        for m in pb.machines() {
            // all tasks on this machine
            let mut tasks = Vec::new();
            for j in 0..pb.num_jobs {
                let task = Var::Start(j, m);
                let start_var = solver.get_int_var(&task).unwrap();
                let start_time = solution.var_domain(start_var).lb;
                tasks.push(((j, m), start_time));
            }
            // sort task by their start time
            tasks.sort_by_key(|(_task, start_time)| *start_time);
            write!(formatted_solution, "Machine {}:\t", m).unwrap();
            for ((job, op), _) in tasks {
                write!(formatted_solution, "({}, {})\t", job, op).unwrap();
            }
            writeln!(formatted_solution).unwrap();
        }
        println!("\n=== Solution (resource order) ===");
        print!("{}", formatted_solution);
        println!("=================================\n");

        if let Some(output) = &opt.output {
            // write solution to file
            std::fs::write(output, formatted_solution).unwrap();
        }

        solver.print_stats();
        if let Some(expected) = opt.expected_makespan {
            assert_eq!(
                optimum as u32, expected,
                "The makespan found ({}) is not the expected one ({})",
                optimum, expected
            );
        }
        println!("XX\t{}\t{}\t{}", filename, optimum, start_time.elapsed().as_secs_f64());
    } else {
        eprintln!("NO SOLUTION");
        assert!(opt.expected_makespan.is_none(), "Expected a valid solution");
    }
    println!("TOTAL RUNTIME: {:.6}", start_time.elapsed().as_secs_f64());
}

fn is_comment(line: &str) -> bool {
    line.chars().any(|c| c == '#')
    // !line.chars().all(|c| c.is_whitespace() || c.is_numeric())
}

fn ints(input_line: &str) -> impl Iterator<Item = usize> + '_ {
    input_line.split_whitespace().map(|n| n.parse().unwrap())
}

fn lines(input: &str) -> impl Iterator<Item = &str> + '_ {
    input.lines().filter(|l| !is_comment(*l))
}

fn parse(input: &str) -> OpenShop {
    let mut lines = lines(input);
    //input.lines().peekable();
    // if is_comment(lines.peek().unwrap()) {
    //     lines.next(); // drop "Times" line
    // }
    lines.next();
    let mut x = ints(lines.next().unwrap());
    let num_jobs = x.next().unwrap();
    let num_machines = x.next().unwrap();

    // if is_comment(lines.peek().unwrap()) {
    //     lines.next(); // drop "Times" line
    // }
    let mut times = Vec::with_capacity(num_machines * num_jobs);
    for _ in 0..num_jobs {
        for t in ints(lines.next().unwrap()) {
            times.push(t as i32)
        }
    }

    OpenShop {
        num_jobs,
        num_machines,
        times,
    }
}

fn encode(pb: &OpenShop, lower_bound: u32, upper_bound: u32) -> Model {
    let start = |model: &Model, j: usize, t: usize| IVar::new(model.shape.get_variable(&Var::Start(j, t)).unwrap());
    let end = |model: &Model, j: usize, t: usize| start(model, j, t) + pb.duration(j, t);

    let lower_bound = lower_bound as i32;
    let upper_bound = upper_bound as i32;
    let mut m = Model::new();

    let makespan_variable = m.new_ivar(lower_bound, upper_bound, Var::Makespan);
    for j in 0..pb.num_jobs {
        for m1 in 0..pb.num_machines {
            let task_start = m.new_ivar(0, upper_bound, Var::Start(j, m1));
            m.enforce(leq(task_start + pb.duration(j, m1), makespan_variable));
        }
    }
    for j in 0..pb.num_jobs {
        for m1 in 0..pb.num_machines {
            for m2 in (m1 + 1)..pb.num_machines {
                let prec = m.new_bvar(Var::Prec(j, m1, j, m2));
                m.bind(leq(end(&m, j, m1), start(&m, j, m2)), prec.true_lit());
                m.bind(leq(end(&m, j, m2), start(&m, j, m1)), prec.false_lit());
            }
        }
    }
    for machine in 0..(pb.num_machines) {
        for j1 in 0..pb.num_jobs {
            for j2 in (j1 + 1)..pb.num_jobs {
                // variable that is true if (j1, i1) comes first and false otherwise.
                // in any case, setting a value to it enforces that the two tasks do not overlap
                let prec = m.new_bvar(Var::Prec(j1, machine, j2, machine));
                m.bind(leq(end(&m, j1, machine), start(&m, j2, machine)), prec.true_lit());
                m.bind(leq(end(&m, j2, machine), start(&m, j1, machine)), prec.false_lit());
            }
        }
    }
    m
}

/// Builds a solver for the given strategy.
pub fn get_solver(base: Solver, strategy: SearchStrategy, est_brancher: EstBrancher) -> ParSolver {
    let base_solver = Box::new(base);
    let make_act = |s: &mut Solver| s.set_brancher(ActivityBrancher::new_with_heuristic(ResourceOrderingFirst));
    let make_est = |s: &mut Solver| s.set_brancher(est_brancher.clone());
    match strategy {
        SearchStrategy::Activity => ParSolver::new(base_solver, 1, |_, s| make_act(s)),
        SearchStrategy::Est => ParSolver::new(base_solver, 1, |_, s| make_est(s)),
        SearchStrategy::Parallel => ParSolver::new(base_solver, 2, |id, s| match id {
            0 => make_act(s),
            1 => make_est(s),
            _ => unreachable!(),
        }),
    }
}

#[derive(Clone)]
pub struct EstBrancher {
    pb: OpenShop,
    saved: DecLvl,
}

impl SearchControl<Var> for EstBrancher {
    fn next_decision(&mut self, _stats: &Stats, model: &Model) -> Option<Decision> {
        let active_in_job = |j: usize| {
            for t in 0..self.pb.num_machines {
                let v = model.shape.get_variable(&Var::Start(j, t)).unwrap();
                let (lb, ub) = model.domain_of(v);
                if lb < ub {
                    return Some((v, lb, ub));
                }
            }
            None
        };
        // for each job selects the first task whose start time is not fixed yet
        let active_tasks = self.pb.jobs().filter_map(active_in_job);
        // among the task with the smallest "earliest starting time (est)" pick the one that has the least slack
        let best = active_tasks.min_by_key(|(_var, est, lst)| (*est, *lst));

        // decision is to set the start time to the selected task to the smallest possible value.
        // if no task was selected, it means that they are all instantiated and we have a complete schedule
        best.map(|(var, est, _)| Decision::SetLiteral(Lit::leq(var, est)))
    }

    fn clone_to_box(&self) -> Box<dyn SearchControl<Var> + Send> {
        Box::new(self.clone())
    }
}

impl Backtrack for EstBrancher {
    fn save_state(&mut self) -> DecLvl {
        self.saved += 1;
        self.saved
    }

    fn num_saved(&self) -> u32 {
        self.saved.to_int()
    }

    fn restore_last(&mut self) {
        self.saved -= 1;
    }
}
