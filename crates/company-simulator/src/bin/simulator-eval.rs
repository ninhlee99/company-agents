use company_simulator::{evaluate, EvaluationConfig, SimConfig};
use std::{env, error::Error};

fn parse_env<T>(name: &str, default: T) -> Result<T, Box<dyn Error>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
{
    match env::var(name) {
        Ok(value) => Ok(value.parse::<T>()?),
        Err(_) => Ok(default),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let runs = parse_env("SIM_EVAL_RUNS", 30_u32)?;
    let days = parse_env("SIM_EVAL_DAYS", 180_u32)?;
    let seed = parse_env("SIM_EVAL_SEED", 42_u64)?;
    let seed_stride = parse_env("SIM_EVAL_SEED_STRIDE", 1_u64)?;

    let config = EvaluationConfig {
        runs,
        seed,
        seed_stride,
        simulation: SimConfig {
            days,
            ..SimConfig::default()
        },
    };

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let report = runtime.block_on(evaluate(config))?;

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
