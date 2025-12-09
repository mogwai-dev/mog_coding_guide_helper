pub mod span;
pub mod token;
pub mod lexer;
pub mod parser;
pub mod ast;
pub mod formatter;
pub mod diagnostics;

// 主要な型を re-export して使いやすくする
pub use span::Span;
pub use token::Token;
pub use lexer::Lexer;
pub use parser::Parser;
pub use ast::{TranslationUnit, Item};
pub use formatter::Formatter;
pub use diagnostics::{Diagnostic, DiagnosticSeverity, DiagnosticConfig, diagnose};

#[cfg(test)]
mod tests;
