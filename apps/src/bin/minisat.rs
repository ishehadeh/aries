#![allow(clippy::map_entry)]

use aries_model::lang::{BAtom, BVar, Bound};
use aries_model::Model;
use aries_smt::solver::SMTSolver;
use std::collections::HashMap;
use std::fs;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "minisat")]
struct Opt {
    file: String,
    /// Sets the initial polarity of the variables to True/False to serve as the preferred value for variables.
    /// If not set, the solver will use an arbitrary value.
    #[structopt(long)]
    polarity: Option<bool>,
    #[structopt(long = "sat")]
    expected_satisfiability: Option<bool>,
}

fn main() {
    let opt = Opt::from_args();

    let file_content = fs::read_to_string(opt.file).expect("Cannot read file");

    let mut model = Model::new();
    let constraints = parse(&file_content, &mut model).unwrap();
    let mut solver = SMTSolver::new(model);
    solver.enforce_all(&constraints);
    // solver.solve();
    // solver.model.discrete.print();
    //
    // let mut solver = aries_sat::solver::Solver::with_clauses(clauses, SearchParams::default());
    // match opt.polarity {
    //     Some(true) => solver.variables().for_each(|v| solver.set_polarity(v, true)),
    //     Some(false) => solver.variables().for_each(|v| solver.set_polarity(v, false)),
    //     None => (),
    // };
    if solver.solve() {
        println!("SAT");
        if opt.expected_satisfiability == Some(false) {
            eprintln!("Error: expected UNSAT but got SAT");
            std::process::exit(1);
        }
    } else {
        println!("UNSAT");
        if opt.expected_satisfiability == Some(true) {
            eprintln!("Error: expected SAT but got UNSAT");
            std::process::exit(1);
        }
    }
    println!("{}", solver.stats);
}

/// Parses a set of clauses in CNF format (see `problems/cnf` for example)
pub fn parse(input: &str, model: &mut Model) -> Result<Vec<BAtom>, String> {
    let mut vars: HashMap<u32, BVar> = Default::default();
    let mut clauses = Vec::new();

    let mut lines_iter = input.lines().filter(|l| !l.starts_with('c'));
    let header = lines_iter.next();
    if header.and_then(|h| h.chars().next()) != Some('p') {
        return Err("No header line starting with 'p'".to_string());
    }
    let mut lits = Vec::with_capacity(32);
    for l in lines_iter {
        lits.clear();
        for lit in l.split_whitespace() {
            match lit.parse::<i32>() {
                Ok(0) => break,
                Ok(i) => {
                    let var_id = i.abs() as u32;
                    if !vars.contains_key(&var_id) {
                        vars.insert(var_id, model.new_bvar(format!("b{}", var_id)));
                    }
                    let var = vars[&var_id];
                    let lit: Bound = if i > 0 { var.into() } else { !var };
                    lits.push(lit.into());
                }
                Err(_) => return Err(format!("Invalid literal: {}", lit)),
            }
        }
        clauses.push(model.or(&lits));
    }
    Ok(clauses)
}

#[cfg(test)]
mod tests {
    use crate::parse;
    use aries_model::Model;

    const CNF_TEST: &str = "c This Formular is generated by mcnf
c
p cnf 3 4
-1 0
-2 0
1 2 3 0
1 2 -3 0
";

    #[test]
    fn test_parsing() {
        let mut model = Model::new();
        let constraints = parse(CNF_TEST, &mut model).unwrap();
        assert_eq!(constraints.len(), 4);
    }
}
