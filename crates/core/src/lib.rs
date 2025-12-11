pub mod span;
pub mod token;
pub mod lexer;
pub mod parser;
pub mod ast;
pub mod formatter;
pub mod diagnostics;
pub mod trivia;
pub mod type_system;

// 主要な型を re-export して使いやすくする
pub use span::Span;
pub use token::Token;
pub use lexer::Lexer;
pub use parser::Parser;
pub use ast::{TranslationUnit, Item};
pub use formatter::Formatter;
pub use diagnostics::{Diagnostic, DiagnosticSeverity, DiagnosticConfig, diagnose};
pub use trivia::{Comment, Trivia};

#[cfg(test)]
mod tests;
