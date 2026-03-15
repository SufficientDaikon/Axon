// error.rs — Compiler error types with structured output (FR-043, FR-044, FR-045)
//
// Enhanced with diagnostic categories, severity configuration, and error limits.

use serde::Serialize;
use std::collections::HashSet;

use crate::span::Span;

/// Severity level for compiler diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Note,
}

/// Diagnostic category for filtering and severity overrides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticCategory {
    ParseError,
    TypeError,
    BorrowError,
    ShapeError,
    LintWarning,
    UnusedVariable,
    UnusedImport,
    UnreachableCode,
    DeprecatedSyntax,
}

impl DiagnosticCategory {
    /// Parse a category from a CLI string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "parse-error" => Some(Self::ParseError),
            "type-error" => Some(Self::TypeError),
            "borrow-error" => Some(Self::BorrowError),
            "shape-error" => Some(Self::ShapeError),
            "lint-warning" => Some(Self::LintWarning),
            "unused-variable" => Some(Self::UnusedVariable),
            "unused-import" => Some(Self::UnusedImport),
            "unreachable-code" => Some(Self::UnreachableCode),
            "deprecated-syntax" => Some(Self::DeprecatedSyntax),
            _ => None,
        }
    }

    /// All known category names (for help text).
    pub fn all_names() -> &'static [&'static str] {
        &[
            "parse-error",
            "type-error",
            "borrow-error",
            "shape-error",
            "lint-warning",
            "unused-variable",
            "unused-import",
            "unreachable-code",
            "deprecated-syntax",
        ]
    }
}

/// Configuration for diagnostic severity overrides and error limits.
#[derive(Debug, Clone, Default)]
pub struct DiagnosticConfig {
    pub deny: HashSet<DiagnosticCategory>,
    pub allow: HashSet<DiagnosticCategory>,
    pub error_limit: Option<usize>,
}

impl DiagnosticConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from CLI flags. Returns an error message for unknown category names.
    pub fn from_cli(
        warn: &[String],
        deny: &[String],
        allow: &[String],
        error_limit: Option<usize>,
    ) -> Result<Self, String> {
        let mut config = DiagnosticConfig {
            error_limit,
            ..Default::default()
        };

        for name in deny {
            match DiagnosticCategory::from_str(name) {
                Some(cat) => { config.deny.insert(cat); }
                None => return Err(format!("unknown diagnostic category: '{}'. Valid: {:?}", name, DiagnosticCategory::all_names())),
            }
        }
        for name in allow {
            match DiagnosticCategory::from_str(name) {
                Some(cat) => { config.allow.insert(cat); }
                None => return Err(format!("unknown diagnostic category: '{}'. Valid: {:?}", name, DiagnosticCategory::all_names())),
            }
        }
        // --warn categories: remove from allow, remove from deny (just use default severity)
        for name in warn {
            if DiagnosticCategory::from_str(name).is_none() {
                return Err(format!("unknown diagnostic category: '{}'. Valid: {:?}", name, DiagnosticCategory::all_names()));
            }
        }

        Ok(config)
    }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<DiagnosticCategory>,
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
            category: None,
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

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_category(mut self, category: DiagnosticCategory) -> Self {
        self.category = Some(category);
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

/// Collects multiple errors with severity configuration (FR-045).
#[derive(Debug)]
pub struct ErrorReporter {
    pub errors: Vec<CompileError>,
    pub json_mode: bool,
    pub config: DiagnosticConfig,
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self {
            errors: Vec::new(),
            json_mode: false,
            config: DiagnosticConfig::default(),
        }
    }
}

impl ErrorReporter {
    pub fn new(json_mode: bool) -> Self {
        ErrorReporter {
            errors: Vec::new(),
            json_mode,
            config: DiagnosticConfig::default(),
        }
    }

    pub fn new_with_config(json_mode: bool, config: DiagnosticConfig) -> Self {
        ErrorReporter {
            errors: Vec::new(),
            json_mode,
            config,
        }
    }

    /// Report an error, applying severity overrides from config.
    pub fn report(&mut self, mut error: CompileError) {
        // Apply deny: promote matching categories to Error
        if let Some(cat) = error.category {
            if self.config.deny.contains(&cat) {
                error.severity = Severity::Error;
            }
        }
        self.errors.push(error);
    }

    /// Check if a particular error should be displayed (not allowed/suppressed).
    pub fn should_display(&self, error: &CompileError) -> bool {
        if let Some(cat) = error.category {
            if self.config.allow.contains(&cat) && error.severity != Severity::Error {
                return false;
            }
        }
        true
    }

    /// Check if the error limit has been reached (fatal checkpoint).
    pub fn check_fatal(&self) -> bool {
        if let Some(limit) = self.config.error_limit {
            self.error_count() >= limit
        } else {
            false
        }
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

    pub fn warning_count(&self) -> usize {
        self.errors
            .iter()
            .filter(|e| e.severity == Severity::Warning)
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
            if self.should_display(err) {
                out.push_str(&err.format_human());
                out.push('\n');
            }
        }

        // Summary line
        let errors = self.error_count();
        let warnings = self.warning_count();

        if errors > 0 || warnings > 0 {
            let mut parts = Vec::new();
            if errors > 0 {
                parts.push(format!("{} error{}", errors, if errors == 1 { "" } else { "s" }));
            }
            if warnings > 0 {
                parts.push(format!("{} warning{}", warnings, if warnings == 1 { "" } else { "s" }));
            }
            out.push_str(&format!("{} emitted\n", parts.join(", ")));
        }

        out
    }

    fn render_json(&self) -> String {
        let displayed: Vec<&CompileError> = self.errors.iter()
            .filter(|e| self.should_display(e))
            .collect();
        serde_json::to_string_pretty(&displayed)
            .unwrap_or_else(|_| "[]".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    fn dummy_span() -> Span {
        Span::dummy()
    }

    #[test]
    fn test_compile_error_with_category() {
        let err = CompileError::new("E1001", "undefined variable", dummy_span())
            .with_category(DiagnosticCategory::TypeError);
        assert_eq!(err.category, Some(DiagnosticCategory::TypeError));
    }

    #[test]
    fn test_diagnostic_category_from_str() {
        assert_eq!(DiagnosticCategory::from_str("parse-error"), Some(DiagnosticCategory::ParseError));
        assert_eq!(DiagnosticCategory::from_str("type-error"), Some(DiagnosticCategory::TypeError));
        assert_eq!(DiagnosticCategory::from_str("borrow-error"), Some(DiagnosticCategory::BorrowError));
        assert_eq!(DiagnosticCategory::from_str("unknown"), None);
    }

    #[test]
    fn test_diagnostic_config_from_cli() {
        let config = DiagnosticConfig::from_cli(
            &[],
            &["type-error".into()],
            &["lint-warning".into()],
            Some(5),
        ).unwrap();
        assert!(config.deny.contains(&DiagnosticCategory::TypeError));
        assert!(config.allow.contains(&DiagnosticCategory::LintWarning));
        assert_eq!(config.error_limit, Some(5));
    }

    #[test]
    fn test_diagnostic_config_unknown_category() {
        let result = DiagnosticConfig::from_cli(&[], &["bogus".into()], &[], None);
        assert!(result.is_err());
    }

    #[test]
    fn test_reporter_deny_promotes_to_error() {
        let mut config = DiagnosticConfig::new();
        config.deny.insert(DiagnosticCategory::LintWarning);

        let mut reporter = ErrorReporter::new_with_config(false, config);
        let warn = CompileError::new("W5001", "unused variable", dummy_span())
            .with_severity(Severity::Warning)
            .with_category(DiagnosticCategory::LintWarning);
        reporter.report(warn);

        assert_eq!(reporter.error_count(), 1);
        assert!(reporter.has_errors());
    }

    #[test]
    fn test_reporter_allow_suppresses_display() {
        let mut config = DiagnosticConfig::new();
        config.allow.insert(DiagnosticCategory::UnusedVariable);

        let reporter = ErrorReporter::new_with_config(false, config);
        let warn = CompileError::new("W5001", "unused variable", dummy_span())
            .with_severity(Severity::Warning)
            .with_category(DiagnosticCategory::UnusedVariable);

        assert!(!reporter.should_display(&warn));
    }

    #[test]
    fn test_reporter_allow_does_not_suppress_errors() {
        let mut config = DiagnosticConfig::new();
        config.allow.insert(DiagnosticCategory::TypeError);

        let reporter = ErrorReporter::new_with_config(false, config);
        let err = CompileError::new("E2001", "type mismatch", dummy_span())
            .with_category(DiagnosticCategory::TypeError);

        // Errors are never suppressed, even if category is in allow list
        assert!(reporter.should_display(&err));
    }

    #[test]
    fn test_reporter_error_limit() {
        let config = DiagnosticConfig {
            error_limit: Some(2),
            ..Default::default()
        };

        let mut reporter = ErrorReporter::new_with_config(false, config);
        reporter.report(CompileError::new("E0001", "error 1", dummy_span()));
        assert!(!reporter.check_fatal());
        reporter.report(CompileError::new("E0002", "error 2", dummy_span()));
        assert!(reporter.check_fatal());
    }

    #[test]
    fn test_reporter_summary_line() {
        let mut reporter = ErrorReporter::new(false);
        reporter.report(CompileError::new("E0001", "error 1", dummy_span()));
        reporter.report(
            CompileError::new("W5001", "warning 1", dummy_span())
                .with_severity(Severity::Warning)
        );
        let output = reporter.render();
        assert!(output.contains("1 error, 1 warning emitted"));
    }

    #[test]
    fn test_reporter_render_no_errors() {
        let reporter = ErrorReporter::new(false);
        let output = reporter.render();
        assert_eq!(output, "");
    }

    #[test]
    fn test_reporter_warning_count() {
        let mut reporter = ErrorReporter::new(false);
        reporter.report(
            CompileError::new("W5001", "warn a", dummy_span()).with_severity(Severity::Warning)
        );
        reporter.report(
            CompileError::new("W5002", "warn b", dummy_span()).with_severity(Severity::Warning)
        );
        reporter.report(CompileError::new("E0001", "error", dummy_span()));
        assert_eq!(reporter.warning_count(), 2);
        assert_eq!(reporter.error_count(), 1);
    }
}
