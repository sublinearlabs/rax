//! Common CLI utilities and types

use colored::*;
use riscv_aot::compiler::AotCompileError;
use std::fmt;

/// Result type for CLI operations
pub type CliResult<T> = Result<T, CliError>;

/// CLI-specific error type
#[derive(Debug)]
pub struct CliError {
    pub message: String,
    pub context: Option<String>,
}

impl CliError {
    /// Create a new CLI error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            context: None,
        }
    }

    /// Add context to the error
    #[allow(dead_code)]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {}", self.message)?;
        if let Some(ctx) = &self.context {
            write!(f, "\n  Context: {}", ctx)?;
        }
        Ok(())
    }
}

impl std::error::Error for CliError {}

/// Convert anyhow errors to CliError
impl From<anyhow::Error> for CliError {
    fn from(err: anyhow::Error) -> Self {
        CliError::new(err.to_string())
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        CliError::new(format!("IO error: {}", err))
    }
}

impl From<AotCompileError> for CliError {
    fn from(err: AotCompileError) -> Self {
        CliError::new(err.to_string())
    }
}

/// Output format for CLI results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum OutputFormat {
    /// Plain text format (default)
    Text,
    /// JSON format
    Json,
    /// CSV format
    Csv,
}

impl OutputFormat {
    /// Parse output format from string
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> CliResult<Self> {
        match s.to_lowercase().as_str() {
            "text" | "t" | "plain" | "p" => Ok(OutputFormat::Text),
            "json" | "j" => Ok(OutputFormat::Json),
            "csv" | "c" => Ok(OutputFormat::Csv),
            _ => Err(CliError::new(format!(
                "Unknown output format: {}. Use: text, json, csv",
                s
            ))),
        }
    }
}

/// Print a colored header
pub fn print_header(text: &str) {
    println!("\n{}", text.bold().cyan());
    println!("{}", "=".repeat(text.len()).cyan());
}

/// Print an info message
pub fn print_info(text: &str) {
    println!("{}", text.cyan());
}

/// Print a success message
pub fn print_success(text: &str) {
    println!("{}", text.green());
}

/// Print an error message
pub fn print_error(text: &str) {
    eprintln!("{}", text.red());
}

/// Print a warning message
#[allow(dead_code)]
pub fn print_warning(text: &str) {
    println!("{}", text.yellow());
}

/// Check if a file exists
pub fn check_file_exists(path: &str) -> CliResult<()> {
    if !std::path::Path::new(path).exists() {
        return Err(CliError::new(format!("File not found: {}", path)));
    }
    Ok(())
}
