use act_logically::engine::datalog::MicroRuntime;
use datalog_rule_macro::program;
use datalog_syntax::*;

fn main() {
    // Skolem Functions + Infinite relations seem to be all that it is necessary.
    let f: SkolemFunctionCall = |args| {
        let x_variable_value = args.get("x").unwrap();
        let y_variable_value = args.get("y").unwrap();

        return format!("{}={}", x_variable_value, y_variable_value).into()
    };

    let program = program! {
        ENV(?x, ?y)     <- [INPUTS("env", ?x, ?y)],
        FENV(f(?x, ?y)) <- [ENV(?x, ?y)]
    };

    let mut micro_runtime = MicroRuntime::new(program);
    micro_runtime.insert("INPUTS", vec!["env".into(), "a".into(), "b".into()]);

    micro_runtime.poll();
    let q = build_query!(FENV(_));
    let answer: Vec<_> = micro_runtime.query(&q).unwrap().into_iter().collect();
    answer
        .into_iter()
        .for_each(|formatted_env| {
            println!("{}({})", "FENV", formatted_env[0])
        })
}
