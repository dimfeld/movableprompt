use std::io::BufRead;

use error_stack::{Report, ResultExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use ureq::Response;

use crate::model::{map_model_response_err, ModelComms, ModelError, ModelOptions, OutputFormat};

pub const DEFAULT_HOST: &str = "http://localhost:11434";

#[derive(Debug, Serialize)]
pub struct OllamaRequest<'a> {
    pub model: &'a str,
    pub prompt: &'a str,
    pub system: Option<&'a str>,
    pub format: Option<OutputFormat>,
    pub stream: bool,
    pub options: OllamaModelOptions,
}

#[derive(Debug, Serialize)]
pub struct OllamaModelOptions {
    temperature: f32,
    top_p: Option<f32>,
    top_k: Option<u32>,
    repeat_penalty: Option<f32>,
    num_predict: Option<u32>,
    stop: Vec<String>,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
    done: bool,
    // TODO Add response stats
}

pub fn send_request(
    options: &ModelOptions,
    prompt: &str,
    system: Option<&str>,
    message_tx: flume::Sender<String>,
) -> Result<(), Report<ModelError>> {
    let ModelComms { host, .. } = options.api_host();
    let url = format!("{host}/api/generate");
    let response: Response = ureq::post(&url)
        .send_json(OllamaRequest {
            model: &options.full_model_name(),
            prompt,
            system,
            format: options.format,
            options: OllamaModelOptions {
                temperature: options.temperature,
                top_p: options.top_p,
                top_k: options.top_k,
                repeat_penalty: options.frequency_penalty,
                stop: options.stop.clone(),
                num_predict: options.max_tokens,
            },
            stream: true,
        })
        .map_err(map_model_response_err)?;

    let reader = std::io::BufReader::new(response.into_reader());
    for line in reader.lines() {
        let line = line.change_context(ModelError::Raw)?;
        let chunk = serde_json::from_str::<OllamaResponse>(&line)
            .change_context(ModelError::Deserialize)?;
        message_tx.send(chunk.response).ok();
    }

    Ok(())
}

#[derive(Deserialize, Debug)]
struct ModelInfo {
    modelfile: String,
    parameters: String,
    template: String,
}

pub fn model_context_limit(host: &str, model: &str) -> Result<usize, Report<ModelError>> {
    let response: ModelInfo = ureq::post(&format!("{host}/api/show"))
        .send_json(json!({
            "name": model
        }))
        .map_err(map_model_response_err)?
        .into_json()
        .change_context(ModelError::Deserialize)?;

    let context_param = response
        .parameters
        .split('\n')
        .find(|l| l.starts_with("num_ctx"));

    let Some(context_param) = context_param else {
        // The default if none is specified in the modelfile.
        return Ok(2048);
    };

    // There is at least one space after the param name, so just trim the rest to get the actual value.
    let context_size = context_param["num_ctx ".len()..]
        .trim()
        .parse::<usize>()
        .change_context(ModelError::Deserialize)?;

    Ok(context_size)
}

#[cfg(all(test, feature = "test-ollama"))]
mod test {
    // Note that for these tests to work, you must be running ollama and already have pulled the models
    // that it tries to use.

    use super::model_context_limit;

    #[test]
    /// Get the context size for a model that specifies it in the modelfile.
    fn model_context_with_info() {
        let limit = model_context_limit(super::DEFAULT_HOST, "yarn-mistral:7b-128k-q5_K_M")
            .expect("Fetching context");
        assert_eq!(limit, 131072);
    }

    #[test]
    /// Get the context size for a model that doesn't specify it in the modelfile.
    fn model_context_without_info() {
        let limit = model_context_limit(super::DEFAULT_HOST, "mistral:7b-instruct-q5_K_M")
            .expect("Fetching context");
        assert_eq!(limit, 2048);
    }
}
