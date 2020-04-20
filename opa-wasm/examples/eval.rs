use std::{fs, io};

use clap::{App, Arg};
use opa_wasm::{Policy, Value};
use tracing::Level;
use tracing_subscriber::{fmt, EnvFilter};

fn main() -> Result<(), anyhow::Error> {
    let subscriber = fmt::Subscriber::builder()
        .with_ansi(atty::is(atty::Stream::Stderr))
        .with_max_level(Level::TRACE)
        .with_writer(io::stderr)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

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
    let input_str = matches
        .value_of_os("input")
        .map(fs::read_to_string)
        .unwrap_or_else(|| Ok("{}".to_string()))?;
    let input = serde_json::from_str::<Value>(&input_str)?;

    let module = opa_go::wasm::compile(query, &policy_path)?;
    let mut policy = Policy::from_wasm(&module)?;
    let result = policy.evaluate(&input)?;
    println!("result: {}", result);
    Ok(())
}
