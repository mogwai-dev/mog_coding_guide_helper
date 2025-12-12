use crate::expression::{Expression, BinaryOperator, UnaryOperator};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::token::*;
use crate::span::Span;
use crate::type_table::TypeTable;

/// 式をパースするパーサー
pub struct ExpressionParser<'a> {
    lexer: &'a mut Lexer,
    current_token: Option<Token>,
    type_table: Option<&'a TypeTable>,  // 型テーブルへの参照（オプション）
}

impl<'a> ExpressionParser<'a> {
    pub fn new(lexer: &'a mut Lexer) -> Self {
        let current_token = lexer.next_token();
        ExpressionParser {
            lexer,
            current_token,
            type_table: None,
        }
    }
    
    /// 型テーブルを設定
    pub fn with_type_table(mut self, type_table: &'a TypeTable) -> Self {
        self.type_table = Some(type_table);
        self
    }
    
    /// 名前が型名かどうかをチェック
    fn is_type_name(&self, name: &str) -> bool {
        // 型キーワードのチェック
        matches!(
            name,
            "void" | "char" | "short" | "int" | "long" | "float" | "double" 
            | "signed" | "unsigned" | "_Bool" | "const" | "volatile" | "restrict" | "_Atomic"
        ) || self.type_table.map_or(false, |table| table.is_type_name(name))
    }
    
    /// 次のトークンを取得して current_token を更新
    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }
    
    /// 現在のトークンを参照
    fn peek(&self) -> Option<&Token> {
        self.current_token.as_ref()
    }
    
    /// 式全体をパース（エントリーポイント）
    pub fn parse_expression(&mut self) -> Option<Expression> {
        self.parse_logical_or()
    }
    
    /// 論理和 (||)
    fn parse_logical_or(&mut self) -> Option<Expression> {
        let mut left = self.parse_logical_and()?;
        
        loop {
            if matches!(self.peek(), Some(Token::PipePipe(_))) {
                self.advance(); // consume ||
                let right = self.parse_logical_and()?;
                let span = self.merge_spans(&left, &right);
                left = Expression::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOperator::LogicalOr,
                    right: Box::new(right),
                    span,
                };
            } else {
                break;
            }
        }
        
        Some(left)
    }
    
    /// 論理積 (&&)
    fn parse_logical_and(&mut self) -> Option<Expression> {
        let mut left = self.parse_bitwise_or()?;
        
        loop {
            if matches!(self.peek(), Some(Token::AmpersandAmpersand(_))) {
                self.advance(); // consume &&
                let right = self.parse_bitwise_or()?;
                let span = self.merge_spans(&left, &right);
                left = Expression::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOperator::LogicalAnd,
                    right: Box::new(right),
                    span,
                };
            } else {
                break;
            }
        }
        
        Some(left)
    }
    
    /// ビット和 (|)
    fn parse_bitwise_or(&mut self) -> Option<Expression> {
        let mut left = self.parse_bitwise_xor()?;
        
        loop {
            if matches!(self.peek(), Some(Token::Pipe(_))) {
                self.advance(); // consume |
                let right = self.parse_bitwise_xor()?;
                let span = self.merge_spans(&left, &right);
                left = Expression::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOperator::BitwiseOr,
                    right: Box::new(right),
                    span,
                };
            } else {
                break;
            }
        }
        
        Some(left)
    }
    
    /// ビット排他的論理和 (^)
    fn parse_bitwise_xor(&mut self) -> Option<Expression> {
        let mut left = self.parse_bitwise_and()?;
        
        loop {
            if matches!(self.peek(), Some(Token::Caret(_))) {
                self.advance(); // consume ^
                let right = self.parse_bitwise_and()?;
                let span = self.merge_spans(&left, &right);
                left = Expression::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOperator::BitwiseXor,
                    right: Box::new(right),
                    span,
                };
            } else {
                break;
            }
        }
        
        Some(left)
    }
    
    /// ビット積 (&)
    fn parse_bitwise_and(&mut self) -> Option<Expression> {
        let mut left = self.parse_equality()?;
        
        loop {
            if matches!(self.peek(), Some(Token::Ampersand(_))) {
                self.advance(); // consume &
                let right = self.parse_equality()?;
                let span = self.merge_spans(&left, &right);
                left = Expression::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOperator::BitwiseAnd,
                    right: Box::new(right),
                    span,
                };
            } else {
                break;
            }
        }
        
        Some(left)
    }
    
    /// 等価比較 (==, !=)
    fn parse_equality(&mut self) -> Option<Expression> {
        let mut left = self.parse_relational()?;
        
        loop {
            let op = match self.peek() {
                Some(Token::EqualEqual(_)) => BinaryOperator::Equal,
                Some(Token::NotEqual(_)) => BinaryOperator::NotEqual,
                _ => break,
            };
            
            self.advance(); // consume operator
            let right = self.parse_relational()?;
            let span = self.merge_spans(&left, &right);
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        
        Some(left)
    }
    
    /// 関係比較 (<, <=, >, >=)
    fn parse_relational(&mut self) -> Option<Expression> {
        let mut left = self.parse_shift()?;
        
        loop {
            let op = match self.peek() {
                Some(Token::LessThan(_)) => BinaryOperator::LessThan,
                Some(Token::LessThanOrEqual(_)) => BinaryOperator::LessThanOrEq,
                Some(Token::GreaterThan(_)) => BinaryOperator::GreaterThan,
                Some(Token::GreaterThanOrEqual(_)) => BinaryOperator::GreaterThanOrEq,
                _ => break,
            };
            
            self.advance(); // consume operator
            let right = self.parse_shift()?;
            let span = self.merge_spans(&left, &right);
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        
        Some(left)
    }
    
    /// シフト演算 (<<, >>)
    fn parse_shift(&mut self) -> Option<Expression> {
        let mut left = self.parse_additive()?;
        
        loop {
            let op = match self.peek() {
                Some(Token::LeftShift(_)) => BinaryOperator::LeftShift,
                Some(Token::RightShift(_)) => BinaryOperator::RightShift,
                _ => break,
            };
            
            self.advance(); // consume operator
            let right = self.parse_additive()?;
            let span = self.merge_spans(&left, &right);
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        
        Some(left)
    }
    
    /// 加減算 (+, -)
    fn parse_additive(&mut self) -> Option<Expression> {
        let mut left = self.parse_multiplicative()?;
        
        loop {
            let op = match self.peek() {
                Some(Token::Plus(_)) => BinaryOperator::Add,
                Some(Token::Minus(_)) => BinaryOperator::Subtract,
                _ => break,
            };
            
            self.advance(); // consume operator
            let right = self.parse_multiplicative()?;
            let span = self.merge_spans(&left, &right);
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        
        Some(left)
    }
    
    /// 乗算・除算・剰余 (*, /, %)
    fn parse_multiplicative(&mut self) -> Option<Expression> {
        let mut left = self.parse_unary()?;
        
        loop {
            let op = match self.peek() {
                Some(Token::Asterisk(_)) => BinaryOperator::Multiply,
                Some(Token::Slash(_)) => BinaryOperator::Divide,
                Some(Token::Percent(_)) => BinaryOperator::Modulo,
                _ => break,
            };
            
            self.advance(); // consume operator
            let right = self.parse_unary()?;
            let span = self.merge_spans(&left, &right);
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        
        Some(left)
    }
    
    /// 単項演算子 (-, !, ~, &, *, ++, --)
    fn parse_unary(&mut self) -> Option<Expression> {
        // 前置単項演算子をチェック
        let (op, op_span) = match self.peek() {
            Some(Token::Minus(_)) => {
                let span = self.get_current_span();
                self.advance();
                (Some(UnaryOperator::Negate), span)
            },
            Some(Token::Exclamation(_)) => {
                let span = self.get_current_span();
                self.advance();
                (Some(UnaryOperator::LogicalNot), span)
            },
            Some(Token::Tilde(_)) => {
                let span = self.get_current_span();
                self.advance();
                (Some(UnaryOperator::BitwiseNot), span)
            },
            Some(Token::Ampersand(_)) => {
                let span = self.get_current_span();
                self.advance();
                (Some(UnaryOperator::AddressOf), span)
            },
            Some(Token::Asterisk(_)) => {
                let span = self.get_current_span();
                self.advance();
                (Some(UnaryOperator::Dereference), span)
            },
            Some(Token::PlusPlus(_)) => {
                let span = self.get_current_span();
                self.advance();
                (Some(UnaryOperator::PreIncrement), span)
            },
            Some(Token::MinusMinus(_)) => {
                let span = self.get_current_span();
                self.advance();
                (Some(UnaryOperator::PreDecrement), span)
            },
            _ => (None, Span {
                start_line: 0,
                start_column: 0,
                end_line: 0,
                end_column: 0,
                byte_start_idx: 0,
                byte_end_idx: 0,
            }),
        };
        
        if let Some(operator) = op {
            // 単項演算子がある場合、再帰的にparse_unary（複数の単項演算子に対応）
            let operand = Box::new(self.parse_unary()?);
            let operand_span = self.get_expression_span(&operand);
            
            let span = Span {
                start_line: op_span.start_line,
                start_column: op_span.start_column,
                end_line: operand_span.end_line,
                end_column: operand_span.end_column,
                byte_start_idx: op_span.byte_start_idx,
                byte_end_idx: operand_span.byte_end_idx,
            };
            
            Some(Expression::UnaryOp {
                op: operator,
                operand,
                span,
            })
        } else {
            // 単項演算子がない場合は postfix または primary
            self.parse_postfix()
        }
    }
    
    /// 後置演算子 (++, --)
    fn parse_postfix(&mut self) -> Option<Expression> {
        let mut expr = self.parse_primary()?;
        
        // 後置インクリメント/デクリメント
        loop {
            match self.peek() {
                Some(Token::PlusPlus(_)) => {
                    let op_span = self.get_current_span();
                    self.advance();
                    let expr_span = self.get_expression_span(&expr);
                    
                    let span = Span {
                        start_line: expr_span.start_line,
                        start_column: expr_span.start_column,
                        end_line: op_span.end_line,
                        end_column: op_span.end_column,
                        byte_start_idx: expr_span.byte_start_idx,
                        byte_end_idx: op_span.byte_end_idx,
                    };
                    
                    expr = Expression::UnaryOp {
                        op: UnaryOperator::PostIncrement,
                        operand: Box::new(expr),
                        span,
                    };
                },
                Some(Token::MinusMinus(_)) => {
                    let op_span = self.get_current_span();
                    self.advance();
                    let expr_span = self.get_expression_span(&expr);
                    
                    let span = Span {
                        start_line: expr_span.start_line,
                        start_column: expr_span.start_column,
                        end_line: op_span.end_line,
                        end_column: op_span.end_column,
                        byte_start_idx: expr_span.byte_start_idx,
                        byte_end_idx: op_span.byte_end_idx,
                    };
                    
                    expr = Expression::UnaryOp {
                        op: UnaryOperator::PostDecrement,
                        operand: Box::new(expr),
                        span,
                    };
                },
                _ => break,
            }
        }
        
        Some(expr)
    }
    
    /// 基本式（リテラル、識別子、括弧式）
    fn parse_primary(&mut self) -> Option<Expression> {
        let token = self.current_token.clone()?;
        
        match token {
            Token::NumberLiteral(NumberLiteralToken { value, span }) => {
                self.advance();
                // Remove suffix before parsing
                let value_without_suffix = value.trim_end_matches(|c: char| {
                    c == 'u' || c == 'U' || c == 'l' || c == 'L'
                });
                
                let int_value = if value_without_suffix.starts_with("0x") || value_without_suffix.starts_with("0X") {
                    // 16進数
                    i64::from_str_radix(&value_without_suffix[2..], 16).ok()?
                } else if value_without_suffix.starts_with('0') && value_without_suffix.len() > 1 {
                    // 8進数
                    i64::from_str_radix(value_without_suffix, 8).ok()?
                } else {
                    // 10進数
                    value_without_suffix.parse::<i64>().ok()?
                };
                
                Some(Expression::IntLiteral { 
                    value: int_value,
                    span,
                })
            }
            Token::FloatLiteral(FloatLiteralToken { value, span }) => {
                self.advance();
                // Remove suffix before parsing
                let value_without_suffix = value.trim_end_matches(|c: char| {
                    c == 'f' || c == 'F' || c == 'l' || c == 'L'
                });
                
                let float_value = value_without_suffix.parse::<f64>().ok()?;
                Some(Expression::FloatLiteral { 
                    value: float_value,
                    span,
                })
            }
            Token::Ident(IdentToken { name, span }) => {
                self.advance();
                Some(Expression::Identifier { 
                    name,
                    span,
                })
            }
            Token::LeftParen(_) => {
                // ( で始まる場合、キャスト式か括弧式かを判定
                let start_span = self.get_current_span();
                self.advance(); // consume '('
                
                // 次のトークンが型名キーワードかtypedef名かチェック
                let is_type = match self.peek() {
                    Some(Token::Void(_)) | Some(Token::Char(_)) | Some(Token::Short(_)) 
                    | Some(Token::Int(_)) | Some(Token::Long(_)) | Some(Token::Float(_)) 
                    | Some(Token::Double(_)) | Some(Token::Signed(_)) | Some(Token::Unsigned(_))
                    | Some(Token::Struct(_)) | Some(Token::Union(_)) | Some(Token::Enum(_))
                    | Some(Token::Const(_)) | Some(Token::Volatile(_)) => true,
                    Some(Token::Ident(IdentToken { name, .. })) => {
                        // typedef名かどうかをチェック
                        self.is_type_name(name)
                    },
                    _ => false,
                };
                
                if is_type {
                    // キャスト式 (Type) expr
                    let mut type_tokens = Vec::new();
                    loop {
                        match self.peek() {
                            Some(Token::RightParen(_)) => break,
                            None => return None,
                            _ => {
                                if let Some(token) = self.current_token.clone() {
                                    type_tokens.push(token);
                                }
                                self.advance();
                            }
                        }
                    }
                    
                    if !matches!(self.peek(), Some(Token::RightParen(_))) {
                        return None;
                    }
                    self.advance(); // ')' をスキップ
                    
                    // 型名をパース
                    let target_type = self.parse_type_from_tokens(&type_tokens);
                    
                    // キャスト対象の式をパース（unaryレベルから再開）
                    let operand = self.parse_unary()?;
                    let operand_span = self.get_expression_span(&operand);
                    
                    let span = Span {
                        start_line: start_span.start_line,
                        start_column: start_span.start_column,
                        end_line: operand_span.end_line,
                        end_column: operand_span.end_column,
                        byte_start_idx: start_span.byte_start_idx,
                        byte_end_idx: operand_span.byte_end_idx,
                    };
                    
                    return Some(Expression::Cast {
                        target_type,
                        operand: Box::new(operand),
                        span,
                    });
                } else {
                    // 括弧式 ( expression )
                    let inner_expr = self.parse_expression()?;
                    
                    // ')' を期待
                    if !matches!(self.peek(), Some(Token::RightParen(_))) {
                        return None; // エラー: 閉じ括弧がない
                    }
                    
                    let end_span = self.get_current_span();
                    self.advance(); // consume ')'
                    
                    // 括弧式のSpanは括弧全体を含む
                    let span = Span {
                        start_line: start_span.start_line,
                        start_column: start_span.start_column,
                        end_line: end_span.end_line,
                        end_column: end_span.end_column,
                        byte_start_idx: start_span.byte_start_idx,
                        byte_end_idx: end_span.byte_end_idx,
                    };
                    
                    // 括弧式は内部の式をそのまま返すが、Spanは更新
                    // 注: 将来的にParenthesized式を追加してもよい
                    Some(match inner_expr {
                        Expression::IntLiteral { value, .. } => Expression::IntLiteral { value, span },
                        Expression::FloatLiteral { value, .. } => Expression::FloatLiteral { value, span },
                        Expression::Identifier { name, .. } => Expression::Identifier { name, span },
                        Expression::BinaryOp { left, op, right, .. } => Expression::BinaryOp { left, op, right, span },
                        Expression::UnaryOp { op, operand, .. } => Expression::UnaryOp { op, operand, span },
                        Expression::Cast { target_type, operand, .. } => Expression::Cast { target_type, operand, span },
                        other => other, // その他の式はそのまま
                    })
                }
            }
            _ => None
        }
    }
    
    /// 2つの式のSpanをマージして新しいSpanを作成
    fn merge_spans(&self, left: &Expression, right: &Expression) -> Span {
        let left_span = self.get_expression_span(left);
        let right_span = self.get_expression_span(right);
        
        Span {
            start_line: left_span.start_line,
            start_column: left_span.start_column,
            end_line: right_span.end_line,
            end_column: right_span.end_column,
            byte_start_idx: left_span.byte_start_idx,
            byte_end_idx: right_span.byte_end_idx,
        }
    }
    
    /// 現在のトークンのSpanを取得
    fn get_current_span(&self) -> Span {
        match &self.current_token {
            Some(Token::NumberLiteral(t)) => t.span.clone(),
            Some(Token::FloatLiteral(t)) => t.span.clone(),
            Some(Token::Ident(t)) => t.span.clone(),
            Some(Token::Plus(t)) => t.span.clone(),
            Some(Token::Minus(t)) => t.span.clone(),
            Some(Token::Asterisk(t)) => t.span.clone(),
            Some(Token::Slash(t)) => t.span.clone(),
            Some(Token::Percent(t)) => t.span.clone(),
            Some(Token::EqualEqual(t)) => t.span.clone(),
            Some(Token::NotEqual(t)) => t.span.clone(),
            Some(Token::LessThan(t)) => t.span.clone(),
            Some(Token::LessThanOrEqual(t)) => t.span.clone(),
            Some(Token::GreaterThan(t)) => t.span.clone(),
            Some(Token::GreaterThanOrEqual(t)) => t.span.clone(),
            Some(Token::Ampersand(t)) => t.span.clone(),
            Some(Token::AmpersandAmpersand(t)) => t.span.clone(),
            Some(Token::Pipe(t)) => t.span.clone(),
            Some(Token::PipePipe(t)) => t.span.clone(),
            Some(Token::Caret(t)) => t.span.clone(),
            Some(Token::Tilde(t)) => t.span.clone(),
            Some(Token::Exclamation(t)) => t.span.clone(),
            Some(Token::LeftShift(t)) => t.span.clone(),
            Some(Token::RightShift(t)) => t.span.clone(),
            Some(Token::LeftParen(t)) => t.span.clone(),
            Some(Token::RightParen(t)) => t.span.clone(),
            Some(Token::PlusPlus(t)) => t.span.clone(),
            Some(Token::MinusMinus(t)) => t.span.clone(),
            _ => Span {
                start_line: 0,
                start_column: 0,
                end_line: 0,
                end_column: 0,
                byte_start_idx: 0,
                byte_end_idx: 0,
            }
        }
    }
    
    /// トークンからSpanを取得するヘルパー
    fn get_current_span_from_token(&self, token: &Token) -> Span {
        match token {
            Token::NumberLiteral(t) => t.span.clone(),
            Token::FloatLiteral(t) => t.span.clone(),
            Token::Ident(t) => t.span.clone(),
            Token::LeftParen(t) => t.span.clone(),
            Token::RightParen(t) => t.span.clone(),
            _ => Span {
                start_line: 0,
                start_column: 0,
                end_line: 0,
                end_column: 0,
                byte_start_idx: 0,
                byte_end_idx: 0,
            }
        }
    }
    
    /// 式からSpanを取得
    fn get_expression_span(&self, expr: &Expression) -> Span {
        match expr {
            Expression::IntLiteral { span, .. } => span.clone(),
            Expression::FloatLiteral { span, .. } => span.clone(),
            Expression::Identifier { span, .. } => span.clone(),
            Expression::BinaryOp { span, .. } => span.clone(),
            Expression::UnaryOp { span, .. } => span.clone(),
            Expression::Cast { span, .. } => span.clone(),
            Expression::FunctionCall { span, .. } => span.clone(),
            Expression::ArrayAccess { span, .. } => span.clone(),
            Expression::MemberAccess { span, .. } => span.clone(),
            Expression::PointerMemberAccess { span, .. } => span.clone(),
            Expression::Conditional { span, .. } => span.clone(),
            Expression::Assignment { span, .. } => span.clone(),
        }
    }
    
    /// トークン列から型をパース
    fn parse_type_from_tokens(&self, tokens: &[Token]) -> crate::type_system::Type {
        // トークン列から元のテキストを再構成
        let mut type_text = String::new();
        let mut has_ident = false;
        let mut ident_name = String::new();
        
        for token in tokens {
            match token {
                Token::Void(_) => type_text.push_str("void "),
                Token::Char(_) => type_text.push_str("char "),
                Token::Short(_) => type_text.push_str("short "),
                Token::Int(_) => type_text.push_str("int "),
                Token::Long(_) => type_text.push_str("long "),
                Token::Float(_) => type_text.push_str("float "),
                Token::Double(_) => type_text.push_str("double "),
                Token::Signed(_) => type_text.push_str("signed "),
                Token::Unsigned(_) => type_text.push_str("unsigned "),
                Token::Const(_) => type_text.push_str("const "),
                Token::Volatile(_) => type_text.push_str("volatile "),
                Token::Restrict(_) => type_text.push_str("restrict "),
                Token::Atomic(_) => type_text.push_str("_Atomic "),
                Token::Asterisk(_) => type_text.push_str("* "),
                Token::Ident(IdentToken { name, .. }) => {
                    // typedef名の可能性
                    has_ident = true;
                    ident_name = name.clone();
                },
                _ => {},
            }
        }
        
        // 識別子が1つだけで、それが型名の場合は型テーブルから取得
        if has_ident && type_text.trim().is_empty() {
            if let Some(type_table) = self.type_table {
                if let Some(type_info) = type_table.get_type_info(&ident_name) {
                    return type_info.clone();
                }
            }
            // typedef名が見つからない場合はintとして扱う
            return crate::type_system::Type::new(
                crate::type_system::BaseType::Int,
                Span {
                    start_line: 0,
                    start_column: 0,
                    end_line: 0,
                    end_column: 0,
                    byte_start_idx: 0,
                    byte_end_idx: 0,
                }
            );
        }
        
        // 型キーワードがある場合はParserを使って型をパース
        let type_lexer = Lexer::new(&type_text);
        let mut type_parser = Parser::new(type_lexer);
        type_parser.parse_type().unwrap_or_else(|| {
            // デフォルトとして int を返す
            crate::type_system::Type::new(
                crate::type_system::BaseType::Int,
                Span {
                    start_line: 0,
                    start_column: 0,
                    end_line: 0,
                    end_column: 0,
                    byte_start_idx: 0,
                    byte_end_idx: 0,
                }
            )
        })
    }
}
