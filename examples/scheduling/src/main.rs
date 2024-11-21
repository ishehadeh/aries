mod parser;
mod problem;
mod search;

use crate::problem::{Encoding, OperationId, Problem, ProblemKind};
use crate::search::{SearchStrategy, Solver, Var};
use anyhow::*;
use aries::model::extensions::AssignmentExt;
use aries::model::lang::IVar;
use aries::solver::parallel::{Solution, SolverResult};
use std::fmt::Write;
use std::time::{Duration, Instant};
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(Debug, StructOpt)]
#[structopt(name = "aries-scheduler")]
pub struct Opt {
    /// Kind of the problem to be solved in {jobshop, openshop}
    kind: ProblemKind,
    /// File containing the instance to solve.
    file: String,
    /// Output file to write the solution
    #[structopt(long = "output", short = "o")]
    output: Option<String>,
    /// When set, the solver will fail with an exit code of 1 if the found solution does not have this makespan.
    #[structopt(long = "expected-makespan")]
    expected_makespan: Option<u32>,
    #[structopt(long = "lower-bound", default_value = "0")]
    lower_bound: u32,
    #[structopt(long = "upper-bound", default_value = "100000")]
    upper_bound: u32,
    /// Search strategy to use
    #[structopt(long = "search", default_value = "default")]
    search: SearchStrategy,
    /// maximum runtime, in seconds.
    #[structopt(long = "timeout", short = "t")]
    timeout: Option<u32>,
    /// Number of threads to allocate to search
    #[structopt(long, default_value = "1")]
    num_threads: u32,
}

fn main() -> Result<()> {
    // Terminate the process if a thread panics.
    // take_hook() returns the default hook in case when a custom one is not set
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // invoke the default handler and exit the process
        orig_hook(panic_info);
        std::process::exit(1);
    }));
    // read command line arguments
    let opt = Opt::from_args();

    let file = &opt.file;
    if std::fs::metadata(file)?.is_file() {
        solve(opt.kind, &opt.file, &opt);
        Ok(())
    } else {
        for entry in WalkDir::new(file).follow_links(true).into_iter().filter_map(|e| e.ok()) {
            let f_name = entry.file_name().to_string_lossy();
            if f_name.ends_with(".txt") {
                println!("{f_name}");
                solve(opt.kind, &entry.path().to_string_lossy(), &opt);
            }
        }
        Ok(())
    }
}

fn solve(kind: ProblemKind, instance: &str, opt: &Opt) {
    let deadline = opt.timeout.map(|dur| Instant::now() + Duration::from_secs(dur as u64));
    let start_time = std::time::Instant::now();
    let filecontent = std::fs::read_to_string(instance).expect("Cannot read file");
    let pb = match kind {
        ProblemKind::OpenShop => parser::openshop(&filecontent),
        ProblemKind::JobShop => parser::jobshop(&filecontent),
        ProblemKind::FlexibleShop => parser::flexshop(&filecontent),
    };
    assert_eq!(pb.kind, kind);
    // println!("{:?}", pb);

    let lower_bound = (opt.lower_bound).max(pb.makespan_lower_bound() as u32);
    println!("Initial lower bound: {lower_bound}");

    let (model, encoding) = problem::encode(&pb, lower_bound, opt.upper_bound);
    let makespan: IVar = IVar::new(model.shape.get_variable(&Var::Makespan).unwrap());

    let solver = Solver::new(model);
    let mut solver = search::get_solver(solver, &opt.search, &encoding, opt.num_threads as usize);

    let result = solver.minimize_with(
        makespan,
        |s| println!("New solution with makespan: {}", s.domain_of(makespan).0),
        deadline,
    );

    match result {
        SolverResult::Sol(solution) => {
            let optimum = solution.var_domain(makespan).lb;
            println!("> OPTIMAL (cost: {optimum})");

            // export the solution to file if specified
            export(&solution, &pb, &encoding, opt.output.as_ref());

            solver.print_stats();
            if let Some(expected) = opt.expected_makespan {
                assert_eq!(
                    optimum as u32, expected,
                    "The makespan found ({optimum}) is not the expected one ({expected})"
                );
            }
            println!("XX\t{}\t{}\t{}", instance, optimum, start_time.elapsed().as_secs_f64());
        }
        SolverResult::Unsat => {
            solver.print_stats();
            println!("> UNSATISFIABLE");
            assert!(opt.expected_makespan.is_none(), "Expected a valid solution");
        }
        SolverResult::Timeout(None) => {
            solver.print_stats();
            println!("> TIMEOUT (not solution found)");
        }
        SolverResult::Timeout(Some(solution)) => {
            let best_cost = solution.var_domain(makespan).lb;
            println!("> TIMEOUT (best solution cost {best_cost})");

            // export the solution to file if specified
            export(&solution, &pb, &encoding, opt.output.as_ref());

            solver.print_stats();
        }
    }
    println!("TOTAL RUNTIME: {:.6}", start_time.elapsed().as_secs_f64());
}

/// Write the solution to file if the file if the file is not None
fn export(solution: &Solution, pb: &Problem, encoding: &Encoding, file: Option<&String>) {
    if let Some(output_file) = file {
        let mut formatted_solution = String::new();
        for m in pb.machines() {
            // all tasks on this machine
            let mut tasks = Vec::new();
            for alt in encoding.alternatives_on_machine(m) {
                if solution.entails(alt.presence) {
                    let start_time = solution.var_domain(alt.start).lb;
                    tasks.push((alt.id, start_time));
                }
            }
            // sort task by their start time
            tasks.sort_by_key(|(_task, start_time)| *start_time);
            write!(formatted_solution, "Machine {m}:\t").unwrap();
            for (OperationId { job, op, alt }, _) in tasks {
                let alt = alt.unwrap();
                write!(formatted_solution, "({job}, {op}, {alt})\t").unwrap();
            }
            writeln!(formatted_solution).unwrap();
        }
        // println!("\n=== Solution (resource order) ===");
        // print!("{}", formatted_solution);
        // println!("=================================\n");

        // write solution to file
        std::fs::write(output_file, formatted_solution).unwrap();
    }
}
