use anyhow::{anyhow, bail, Context, Error, Ok};
use aries_core::{Lit, INT_CST_MAX};
use aries_grpc_api::{Expression, ExpressionKind, Problem};
use aries_model::extensions::Shaped;
use aries_model::lang::*;
use aries_model::symbols::SymbolTable;
use aries_model::types::TypeHierarchy;
use aries_planning::chronicles::*;
use aries_planning::parsing::pddl::TypedSymbol;
use aries_utils::input::Sym;
use std::collections::HashSet;
use std::sync::Arc;

/// Names for built in types. They contain UTF-8 symbols for sexiness
/// (and to avoid collision with user defined symbols)
static TASK_TYPE: &str = "★task★";
static ABSTRACT_TASK_TYPE: &str = "★abstract_task★";
static ACTION_TYPE: &str = "★action★";
static DURATIVE_ACTION_TYPE: &str = "★durative-action★";
static METHOD_TYPE: &str = "★method★";
static FLUENT_TYPE: &str = "★fluent★";
static OBJECT_TYPE: &str = "★object★";

pub fn problem_to_chronicles(problem: &Problem) -> Result<aries_planning::chronicles::Problem, Error> {
    // Construct the type hierarchy
    let types = {
        // Static types present in any problem
        let mut types: Vec<(Sym, Option<Sym>)> = vec![
            (TASK_TYPE.into(), None),
            (ABSTRACT_TASK_TYPE.into(), Some(TASK_TYPE.into())),
            (ACTION_TYPE.into(), Some(TASK_TYPE.into())),
            (DURATIVE_ACTION_TYPE.into(), Some(TASK_TYPE.into())),
            (METHOD_TYPE.into(), None),
            (FLUENT_TYPE.into(), None),
            (OBJECT_TYPE.into(), None),
        ];

        // Object types are currently not explicitly declared in the model.
        // Extract all types used in objects declared and add them.
        for obj in &problem.objects {
            let object_type = Sym::from(obj.r#type.clone());

            //check if type is already in types
            if !types.iter().any(|(t, _)| t == &object_type) {
                types.push((object_type.clone(), Some(OBJECT_TYPE.into())));
            }
        }
        // we have all the types, build the hierarchy
        TypeHierarchy::new(types)?
    };

    // determine the top types in the user-defined hierarchy.
    // this is typically "object" by convention but might something else (e.g. "obj" in some hddl problems).
    let mut symbols: Vec<TypedSymbol> = vec![];
    {
        // Types are currently not explicitly declared
        for obj in &problem.objects {
            let object_symbol = Sym::from(obj.name.clone());
            let object_type = Sym::from(obj.r#type.clone());

            // declare the object as a new symbol with the given type
            symbols.push(TypedSymbol {
                symbol: object_symbol.clone(),
                tpe: Some(object_type),
            });
        }

        // record all symbols representing fluents
        for fluent in &problem.fluents {
            symbols.push(TypedSymbol {
                symbol: Sym::from(fluent.name.clone()),
                tpe: Some(FLUENT_TYPE.into()),
            });
        }

        // actions are symbols as well, add them to the table
        for action in &problem.actions {
            symbols.push(TypedSymbol {
                symbol: Sym::from(action.name.clone()),
                tpe: Some(ACTION_TYPE.into()),
            });
        }
    }

    let symbols = symbols
        .drain(..)
        .map(|ts| (ts.symbol, ts.tpe.unwrap_or_else(|| OBJECT_TYPE.into())))
        .collect();
    let symbol_table = SymbolTable::new(types.clone(), symbols)?;

    let from_upf_type = |name: &str| {
        // TODO: Add in the upper and lower bound for int types using regex
        if name == "bool" {
            Ok(Type::Bool)
        } else if name == "int" {
            Ok(Type::Int)
        } else if let Some(tpe) = types.id_of(name) {
            Ok(Type::Sym(tpe))
        } else {
            Err(anyhow!("Unsupported type `{}`", name))
        }
    };

    let mut state_variables = vec![];
    {
        for fluent in &problem.fluents {
            let sym = symbol_table
                .id(&Sym::from(fluent.name.clone()))
                .with_context(|| format!("Fluent `{}` not found in symbol table", fluent.name))?;
            let mut args = Vec::with_capacity(1 + fluent.parameters.len());

            for arg in &fluent.parameters {
                args.push(from_upf_type(arg.r#type.as_str()).with_context(|| {
                    format!("Invalid parameter type `{}` for fluent `{}`", arg.r#type, fluent.name)
                })?);
            }

            args.push(from_upf_type(&fluent.value_type).with_context(|| {
                format!(
                    "Invalid return type `{}` for fluent `{}`",
                    fluent.value_type, fluent.name
                )
            })?);

            state_variables.push(StateFun { sym, tpe: args });
        }
    }

    let mut context = Ctx::new(Arc::new(symbol_table), state_variables);
    println!("===== Symbol Table =====");
    println!("{:?}", context.model.get_symbol_table());

    // Initial chronicle construction
    let mut init_ch = Chronicle {
        kind: ChronicleKind::Problem,
        presence: Lit::TRUE,
        start: context.origin(),
        end: context.horizon(),
        name: vec![],
        task: None,
        conditions: vec![],
        effects: vec![],
        constraints: vec![],
        subtasks: vec![],
    };

    // Initial state translates as effect at the global start time
    println!("===== Initial state =====");
    for init_state in &problem.initial_state {
        let expr = init_state
            .fluent
            .as_ref()
            .context("Initial state assignment has no valid fluent")?;
        let value = init_state
            .value
            .as_ref()
            .context("Initial state assignment has no valid value")?;

        let expr = read_expression(expr, &context)?;
        let value = read_value(value, &context)?;
        println!("{:?} := {:?}", expr, value);

        init_ch.effects.push(Effect {
            transition_start: init_ch.start,
            persistence_start: init_ch.start,
            state_var: expr,
            value,
        })
    }

    // goals translate as condition at the global end time
    println!("===== Goals =====");
    for goal in &problem.goals {
        // a goal is simply a condition where only constant atom can appear
        // TODO: Add temporal behaviour
        let goal_expr = goal.goal.as_ref().context("Goal has no valid expression")?;
        let state_var = read_expression(goal_expr, &context)?;
        let value = read_value(goal_expr, &context)?;
        println!("{:?} == {:?}", state_var, value);

        init_ch.conditions.push(Condition {
            start: init_ch.end,
            end: init_ch.end,
            state_var,
            value,
        })
    }

    // TODO: Task networks?
    let init_ch = ChronicleInstance {
        parameters: vec![],
        origin: ChronicleOrigin::Original,
        chronicle: init_ch,
    };

    let mut templates = Vec::new();
    for a in &problem.actions {
        let cont = Container::Template(templates.len());
        let template = read_chronicle_template(cont, a, &mut context)?;
        templates.push(template);
    }

    let problem = aries_planning::chronicles::Problem {
        context,
        templates,
        chronicles: vec![init_ch],
    };

    Ok(problem)
}

fn str_to_symbol(name: &str, symbol_table: &SymbolTable) -> anyhow::Result<SAtom> {
    let sym = symbol_table
        .id(name)
        .with_context(|| format!("Unknown symbol / operator `{}`", name))?;
    let tpe = symbol_table.type_of(sym);
    Ok(SAtom::new_constant(sym, tpe))
}

// read_atom` functions can return `Atom` or `SAtom`
enum AtomOrSAtom<S, T> {
    Atom(S),  // Atom
    SAtom(T), // SAtom
}

impl From<AtomOrSAtom<Atom, SAtom>> for SAtom {
    fn from(atom: AtomOrSAtom<Atom, SAtom>) -> SAtom {
        match atom {
            AtomOrSAtom::Atom(atom) => panic!("Expected SAtom, got Atom {:?}", atom),
            AtomOrSAtom::SAtom(satom) => satom,
        }
    }
}

impl From<AtomOrSAtom<Atom, SAtom>> for Atom {
    fn from(atom: AtomOrSAtom<Atom, SAtom>) -> Atom {
        match atom {
            AtomOrSAtom::Atom(atom) => atom,
            AtomOrSAtom::SAtom(satom) => Atom::from(satom),
        }
    }
}

fn read_atom(
    atom: &aries_grpc_api::Atom,
    symbol_table: &SymbolTable,
) -> Result<AtomOrSAtom<aries_model::lang::Atom, aries_model::lang::SAtom>, Error> {
    if let Some(atom_content) = atom.content.clone() {
        match atom_content {
            aries_grpc_api::atom::Content::Symbol(s) => {
                let atom = str_to_symbol(s.as_str(), symbol_table)?;
                Ok(AtomOrSAtom::SAtom(atom)) // Handles SAtom
            }
            aries_grpc_api::atom::Content::Int(i) => Ok(AtomOrSAtom::Atom(Atom::from(i))),
            aries_grpc_api::atom::Content::Float(_f) => {
                bail!("`Float` type not supported yet")
            }
            aries_grpc_api::atom::Content::Boolean(b) => Ok(AtomOrSAtom::Atom(Atom::Bool(b.into()))),
        }
    } else {
        Err(anyhow!("Unsupported atom"))
    }
}

/// Transform a condition into an pair (state_variable, value) representing an equality condition between the two.
///
/// The function expect a `read_atom` function that is used to transform an Expression into an atom.
/// This is necessary because this translation is context-dependent:
///  - in the initial facts or goals, an atom is simply a constant (symbol, symbol)
///  - inside an action, a string might refer to an action parameter.
///    In this case `read_atom` should return the corresponding variable that was created to represent the parameter (wrapped into an `Atom`)
fn read_expression(expr: &Expression, context: &Ctx) -> Result<Sv, Error> {
    let mut sv = Vec::new();
    let expr_kind = ExpressionKind::from_i32(expr.kind).unwrap();

    if expr_kind == ExpressionKind::Constant || expr_kind == ExpressionKind::Parameter {
        Ok(vec![read_atom(
            expr.atom.as_ref().unwrap(),
            context.model.get_symbol_table(),
        )?
        .into()])
    } else if expr_kind == ExpressionKind::FunctionApplication {
        assert_eq!(expr.atom, None, "Function application should not have an atom");

        let mut sub_list = expr.list.clone();

        while let Some(sub_expr) = sub_list.pop() {
            let sub_expr_kind = ExpressionKind::from_i32(sub_expr.kind).unwrap();
            if sub_expr_kind == ExpressionKind::Constant {
                continue;
            } else if sub_expr_kind == ExpressionKind::FunctionSymbol {
                assert!(sub_expr.atom.is_some(), "Function symbol should have an atom");
                // TODO: Complete the funciton symbol support
                let operator = sub_expr.atom.as_ref().unwrap().content.as_ref().unwrap();
                if let aries_grpc_api::atom::Content::Symbol(operator) = operator.clone() {
                    match operator.as_str() {
                        "==" => {
                            todo!("`==` operator not supported yet");
                        }
                        "and" => {
                            todo!("`and` operator not supported yet")
                        }
                        "not" => {
                            todo!("`not` operator not supported yet")
                        }
                        _ => {
                            bail!("Unsupported operator `{}`", operator)
                        }
                    }
                } else {
                    bail!("Operator {:?} should be a symbol", operator);
                }
            } else {
                let state_var = read_expression(&sub_expr, context)?;
                sv.extend(state_var);
            }
        }
        Ok(sv)
    } else if expr_kind == ExpressionKind::StateVariable {
        assert_eq!(expr.atom, None, "StateVariable should not have an atom");

        let mut sub_list = expr.list.clone();

        while let Some(sub_expr) = sub_list.pop() {
            if sub_expr.kind == ExpressionKind::FluentSymbol as i32 {
                match read_atom(sub_expr.atom.as_ref().unwrap(), context.model.get_symbol_table())? {
                    AtomOrSAtom::SAtom(fluent) => sv.push(fluent),
                    _ => bail!("Expected a valid fluent symbol as atom in expression"),
                }
            } else {
                let state_var = read_expression(&sub_expr, context)?;
                sv.extend(state_var);
            }
        }
        // FIXME: this is a hack to make sure that the state variables are sorted
        sv.reverse();
        Ok(sv)
    } else {
        bail!(anyhow!("Unsupported expression kind: {:?}", expr_kind))
    }
}

/// Read the constant term from an expression
///  - The function expect a `read_atom` function that is used to transform an Expression into an atom.
///  - It basically expects the `Constant` expression to have a single atom.
///  - If the expression is not a constant, the function looks for `Constant` expressions inside the expression.
///  - If none is found, the function returns an error.
fn read_value(expr: &aries_grpc_api::Expression, context: &Ctx) -> Result<Atom, Error> {
    let expr_kind = ExpressionKind::from_i32(expr.kind).unwrap();
    if expr_kind == ExpressionKind::Constant {
        return Ok(read_atom(expr.atom.as_ref().unwrap(), context.model.get_symbol_table())?.into());
    } else {
        // Fetch the constant expression
        let sub_list = expr.list.clone();
        let constant_expr = sub_list
            .iter()
            .find(|e| ExpressionKind::from_i32(e.kind).unwrap() == ExpressionKind::Constant);
        if constant_expr.is_none() {
            bail!("Expected a constant expression");
        } else {
            return Ok(read_atom(
                constant_expr.unwrap().atom.as_ref().unwrap(),
                context.model.get_symbol_table(),
            )?
            .into());
        }
    }
}

fn read_condition(cond: &aries_grpc_api::Condition, context: &Ctx) -> Result<(Sv, Atom), Error> {
    if let Some(_span) = &cond.span {
        // TODO: Implement the durative condition
        unimplemented!()
    } else {
        let cond = cond.cond.as_ref().context("Condition has no valid expression")?;
        let sv = read_expression(cond, context)?;
        let value = read_value(cond, context)?;
        Ok((sv, value))
    }
}

fn read_effect(eff: &aries_grpc_api::Effect, context: &Ctx) -> Result<(Sv, Atom), Error> {
    if let Some(_occurence_time) = &eff.occurence_time {
        // TODO: Implement the durative effect
        unimplemented!()
    } else {
        let effect = eff.effect.as_ref().unwrap();
        let expr = effect
            .fluent
            .as_ref()
            .with_context(|| "Expected a valid fluent expression".to_string())?;
        let value = effect
            .value
            .as_ref()
            .with_context(|| "Expected a valid value expression".to_string())?;

        let sv = read_expression(expr, context)?;
        let value = read_value(value, context)?;

        Ok((sv, value))
    }
}

fn read_chronicle_template(
    c: Container,
    action: &aries_grpc_api::Action,
    context: &mut Ctx,
) -> Result<ChronicleTemplate, Error> {
    let action_kind = {
        if action.duration.is_some() {
            ChronicleKind::DurativeAction
        } else {
            ChronicleKind::Action
        }
    };
    let mut params: Vec<Variable> = Vec::new();
    let prez_var = context.model.new_bvar(c / VarType::Presence);
    params.push(prez_var.into());
    let prez = prez_var.true_lit();

    let start = context
        .model
        .new_optional_fvar(0, INT_CST_MAX, TIME_SCALE, prez, c / VarType::ChronicleStart);
    params.push(start.into());
    let start = FAtom::from(start);

    let end: FAtom = match action_kind {
        ChronicleKind::Problem => bail!("Problem type not supported"),
        ChronicleKind::Method | ChronicleKind::DurativeAction => {
            // TODO: Add duration
            let end = context
                .model
                .new_optional_fvar(0, INT_CST_MAX, TIME_SCALE, prez, c / VarType::ChronicleEnd);
            params.push(end.into());
            end.into()
        }
        ChronicleKind::Action => start + FAtom::EPSILON,
    };

    let mut name: Vec<SAtom> = Vec::with_capacity(1 + action.parameters.len());
    let base_name = &Sym::from(action.name.clone());
    name.push(
        context
            .typed_sym(
                context
                    .model
                    .get_symbol_table()
                    .id(base_name)
                    .ok_or_else(|| base_name.invalid("Unknown action"))?,
            )
            .into(),
    );

    // Process, the arguments of the action, adding them to the parameters of the chronicle and to the name of the action
    for param in &action.parameters {
        let arg = Sym::from(param.name.clone());
        let arg_type = Sym::from(param.r#type.clone());
        let tpe = context
            .model
            .get_symbol_table()
            .types
            .id_of(&arg_type)
            .ok_or_else(|| arg.invalid("Unknown argument"))?;
        let arg = context.model.new_optional_sym_var(tpe, prez, c / VarType::Parameter); // arg.symbol
        params.push(arg.into());
        name.push(arg.into());
    }

    let mut ch = Chronicle {
        kind: action_kind,
        presence: prez,
        start,
        end,
        name: name.clone(),
        task: None,
        conditions: vec![],
        effects: vec![],
        constraints: vec![],
        subtasks: vec![],
    };

    // Process the effects of the action
    for eff in &action.effects {
        let result = read_effect(eff, context);
        match result {
            Result::Ok(eff) => {
                ch.effects.push(Effect {
                    transition_start: start,
                    persistence_start: end,
                    state_var: eff.0,
                    value: eff.1,
                });
            }
            Result::Err(e) => {
                return Err(anyhow!(
                    "Action {} has an invalid effect: {}",
                    action.name,
                    e.to_string()
                ))
            }
        }
    }

    let positive_effects: HashSet<Sv> = ch
        .effects
        .iter()
        .filter(|e| e.value == Atom::from(true))
        .map(|e| e.state_var.clone())
        .collect();
    ch.effects
        .retain(|e| e.value != Atom::from(false) || !positive_effects.contains(&e.state_var));

    for condition in &action.conditions {
        let result = read_condition(condition, context);
        match result {
            Result::Ok(condition) => ch.conditions.push(Condition {
                start,
                end,
                state_var: condition.0,
                value: condition.1,
            }),
            Result::Err(e) => {
                return Err(anyhow!(
                    "Action {} has an invalid condition: {}",
                    action.name,
                    e.to_string()
                ))
            }
        }
    }

    println!("===");
    dbg!(&ch);
    println!("===");

    Ok(ChronicleTemplate {
        label: Some(action.name.clone()),
        parameters: params,
        chronicle: ch,
    })
}
