use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error reading configuration file")]
    ParseConfig,
    #[error("Error reading template")]
    ParseTemplate,
    #[error("Template not found")]
    TemplateNotFound,
    #[error("Template contents not found")]
    TemplateContentsNotFound,
    #[error("This template is missing template and template_path")]
    EmptyTemplate,
    #[error("Failed to parse arguments")]
    ArgParseFailure,
    #[error("Encountered an error preparinhg the prompt")]
    PreparePrompt,
    #[error("Encountered an error running the prompt")]
    RunPrompt,
    #[error("Failed to calculate context limit")]
    ContextLimit,
    #[error("Failed reading input")]
    Io,
    #[error(transparent)]
    CmdlineParseFailure(#[from] clap::Error),
    #[error("Failed to encode tokens")]
    Tokenizer(String),
}
