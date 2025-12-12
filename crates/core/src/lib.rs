pub mod span;
pub mod token;
pub mod lexer;
pub mod parser;
pub mod ast;
pub mod expression;
pub mod expression_parser;
pub mod statement;
pub mod formatter;
pub mod diagnostics;
pub mod trivia;
pub mod type_system;
pub mod type_table;

// 主要な型を re-export して使いやすくする
pub use span::Span;
pub use token::Token;
pub use lexer::Lexer;
pub use parser::Parser;
pub use ast::{TranslationUnit, Item};
pub use expression::{Expression, BinaryOperator, UnaryOperator};
pub use statement::{Statement, SwitchCase};
pub use formatter::Formatter;
pub use diagnostics::{Diagnostic, DiagnosticSeverity, DiagnosticConfig, diagnose};
pub use trivia::{Comment, Trivia};
pub use type_table::TypeTable;

#[cfg(test)]
mod tests;
