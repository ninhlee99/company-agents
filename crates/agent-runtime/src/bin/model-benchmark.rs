use agent_runtime::model::{model_error_kind, provider_from_name, run_model_benchmark};
use serde::Serialize;
use std::env;

#[derive(Debug, Serialize)]
struct BenchmarkReport {
    providers: Vec<String>,
    initialization_errors: Vec<InitializationError>,
    observations: Vec<agent_runtime::model::ModelBenchmarkObservation>,
}

#[derive(Debug, Serialize)]
struct InitializationError {
    provider: String,
    error_kind: String,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let provider_names = env::var("LLM_BENCHMARK_PROVIDERS")
        .unwrap_or_else(|_| "mock".into())
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .take(16)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    if provider_names.is_empty() {
        return Err("LLM_BENCHMARK_PROVIDERS must name at least one provider".into());
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let report = runtime.block_on(async {
        let mut observations = Vec::new();
        let mut initialization_errors = Vec::new();

        for provider_name in &provider_names {
            match provider_from_name(provider_name) {
                Ok(model) => {
                    observations.extend(
                        run_model_benchmark(provider_name, model.as_ref()).await,
                    );
                }
                Err(error) => initialization_errors.push(InitializationError {
                    provider: provider_name.clone(),
                    error_kind: model_error_kind(&error).into(),
                }),
            }
        }

        BenchmarkReport {
            providers: provider_names.clone(),
            initialization_errors,
            observations,
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
