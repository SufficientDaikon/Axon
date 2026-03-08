// error.rs — Compiler error types with structured output (FR-043, FR-044, FR-045)

use serde::Serialize;
use crate::span::Span;

/// Severity level for compiler diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Note,
}

/// A structured compiler error (FR-043).
#[derive(Debug, Clone, Serialize)]
pub struct CompileError {
    pub error_code: String,
    pub message: String,
    pub severity: Severity,
    pub location: Option<Span>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl CompileError {
    pub fn new(code: &str, message: impl Into<String>, span: Span) -> Self {
        CompileError {
            error_code: code.to_string(),
            message: message.into(),
            severity: Severity::Error,
            location: Some(span),
            suggestion: None,
            notes: Vec::new(),
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Format for human-readable display.
    pub fn format_human(&self) -> String {
        let mut out = String::new();
        let sev = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        };

        if let Some(ref loc) = self.location {
            out.push_str(&format!(
                "{}[{}]: {}\n  --> {}:{}:{}\n",
                sev,
                self.error_code,
                self.message,
                loc.start.file,
                loc.start.line,
                loc.start.column,
            ));
        } else {
            out.push_str(&format!(
                "{}[{}]: {}\n",
                sev, self.error_code, self.message
            ));
        }

        if let Some(ref suggestion) = self.suggestion {
            out.push_str(&format!("  help: {}\n", suggestion));
        }

        for note in &self.notes {
            out.push_str(&format!("  note: {}\n", note));
        }

        out
    }

    /// Format for JSON output (FR-044).
    pub fn format_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| format!("{{\"error\": \"{}\"}}", self.message))
    }
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_human())
    }
}

/// Collects multiple errors (FR-045).
#[derive(Debug, Default)]
pub struct ErrorReporter {
    pub errors: Vec<CompileError>,
    pub json_mode: bool,
}

impl ErrorReporter {
    pub fn new(json_mode: bool) -> Self {
        ErrorReporter {
            errors: Vec::new(),
            json_mode,
        }
    }

    pub fn report(&mut self, error: CompileError) {
        self.errors.push(error);
    }

    pub fn has_errors(&self) -> bool {
        self.errors.iter().any(|e| e.severity == Severity::Error)
    }

    pub fn error_count(&self) -> usize {
        self.errors
            .iter()
            .filter(|e| e.severity == Severity::Error)
            .count()
    }

    /// Render all errors to a string.
    pub fn render(&self) -> String {
        if self.json_mode {
            self.render_json()
        } else {
            self.render_human()
        }
    }

    fn render_human(&self) -> String {
        let mut out = String::new();
        for err in &self.errors {
            out.push_str(&err.format_human());
            out.push('\n');
        }
        if self.has_errors() {
            let count = self.error_count();
            out.push_str(&format!(
                "aborting due to {} previous error{}\n",
                count,
                if count == 1 { "" } else { "s" }
            ));
        }
        out
    }

    fn render_json(&self) -> String {
        serde_json::to_string_pretty(&self.errors)
            .unwrap_or_else(|_| "[]".to_string())
    }
}
