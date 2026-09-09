use crate::lexer::Span;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
}
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: Level,
    pub code: &'static str,
    pub message: String,
    pub span: Span,
}
impl Diagnostic {
    pub fn error(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self {
            level: Level::Error,
            code,
            message: message.into(),
            span,
        }
    }
    pub fn render(&self, source: &str, filename: &str) -> String {
        let line = source
            .lines()
            .nth(self.span.line.saturating_sub(1))
            .unwrap_or("");
        let width = self.span.column.saturating_sub(1);
        let length = self.span.end.saturating_sub(self.span.start).max(1);
        let mark = format!(
            "{}{}",
            " ".repeat(width),
            "^".repeat(length.min(line.len().saturating_sub(width).max(1)))
        );
        format!(
            "{}[{}]: {}\n\n --> {}:{}:{}\n  |\n{} | {}\n  | {}\n",
            match self.level {
                Level::Error => "error",
            },
            self.code,
            self.message,
            filename,
            self.span.line,
            self.span.column,
            self.span.line,
            line,
            mark
        )
    }
}
