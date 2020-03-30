use std::fs;

use clap::{App, Arg};
use policy::Policy;

fn main() -> Result<(), anyhow::Error> {
    let matches = App::new("policy")
        .arg(
            Arg::with_name("policy")
                .short("p")
                .long("policy")
                .value_name("FILE")
                .help("Sets the location of the rego policy file")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("query")
                .short("q")
                .long("query")
                .value_name("QUERY")
                .help("Sets the rego query to evaluate")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .help("Sets the input file path")
                .takes_value(true),
        )
        .get_matches();

    let policy_path = matches.value_of("policy").expect("required policy");
    let query = matches.value_of("query").expect("required query");
    let input = matches
        .value_of_os("input")
        .map(fs::read_to_string)
        .unwrap_or_else(|| Ok("{}".to_string()))?;

    let mut policy = Policy::from_rego(&policy_path, query)?;
    let result = policy.evaluate(&input)?;
    println!("result: {}", result);
    Ok(())
}
