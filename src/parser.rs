//! Parser for MicroPerl

use crate::ast::{BinOp, Expr, Program, Stmt, UnaryOp};
use crate::token::{Token, TokenWithSpan};

pub struct Parser {
    tokens: Vec<TokenWithSpan>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TokenWithSpan>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).map(|t| &t.token).unwrap_or(&Token::Eof)
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos + 1).map(|t| &t.token).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if self.current() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", expected, self.current()))
        }
    }

    fn at(&self, token: &Token) -> bool {
        self.current() == token
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut program = Program::new();
        while !self.at(&Token::Eof) {
            let stmt = self.parse_statement()?;
            program.statements.push(stmt);
        }
        Ok(program)
    }

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        match self.current().clone() {
            Token::My => self.parse_my(),
            Token::Our => self.parse_our(),
            Token::Sub => self.parse_sub(),
            Token::If => self.parse_if(),
            Token::Unless => self.parse_unless(),
            Token::While => self.parse_while(),
            Token::Until => self.parse_until(),
            Token::For => self.parse_for(),
            Token::Foreach => self.parse_foreach(),
            Token::Last => {
                self.advance();
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Last)
            }
            Token::Next => {
                self.advance();
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Next)
            }
            Token::Return => self.parse_return(),
            Token::Print => self.parse_print(),
            Token::Say => self.parse_say(),
            Token::Use => self.parse_use(),
            Token::Package => self.parse_package(),
            Token::LBrace => self.parse_block(),
            _ => {
                let expr = self.parse_expr()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    fn parse_my(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'my'
        let vars = self.parse_var_list()?;
        let init = if self.at(&Token::Assign) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(Token::Semicolon)?;
        Ok(Stmt::My(vars, init))
    }

    fn parse_our(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'our'
        let vars = self.parse_var_list()?;
        let init = if self.at(&Token::Assign) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(Token::Semicolon)?;
        Ok(Stmt::Our(vars, init))
    }

    fn parse_var_list(&mut self) -> Result<Vec<String>, String> {
        let mut vars = Vec::new();

        if self.at(&Token::LParen) {
            self.advance();
            loop {
                match self.current().clone() {
                    Token::ScalarVar(name) | Token::ArrayVar(name) | Token::HashVar(name) => {
                        vars.push(name);
                        self.advance();
                    }
                    _ => return Err(format!("Expected variable, got {:?}", self.current())),
                }
                if self.at(&Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(Token::RParen)?;
        } else {
            match self.current().clone() {
                Token::ScalarVar(name) | Token::ArrayVar(name) | Token::HashVar(name) => {
                    vars.push(name);
                    self.advance();
                }
                _ => return Err(format!("Expected variable, got {:?}", self.current())),
            }
        }

        Ok(vars)
    }

    fn parse_sub(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'sub'
        let name = match self.current().clone() {
            Token::Ident(n) => {
                self.advance();
                n
            }
            _ => return Err(format!("Expected subroutine name, got {:?}", self.current())),
        };

        // Optional parameter list
        let params = if self.at(&Token::LParen) {
            self.advance();
            let mut params = Vec::new();
            while !self.at(&Token::RParen) {
                match self.current().clone() {
                    Token::ScalarVar(name) => {
                        params.push(name);
                        self.advance();
                    }
                    _ => return Err(format!("Expected parameter, got {:?}", self.current())),
                }
                if self.at(&Token::Comma) {
                    self.advance();
                }
            }
            self.expect(Token::RParen)?;
            params
        } else {
            Vec::new()
        };

        self.expect(Token::LBrace)?;
        let body = self.parse_stmt_list()?;
        self.expect(Token::RBrace)?;

        Ok(Stmt::Sub { name, params, body })
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'if'
        self.expect(Token::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;
        let then_block = self.parse_stmt_list()?;
        self.expect(Token::RBrace)?;

        let mut elsif_blocks = Vec::new();
        while self.at(&Token::Elsif) {
            self.advance();
            self.expect(Token::LParen)?;
            let elsif_cond = self.parse_expr()?;
            self.expect(Token::RParen)?;
            self.expect(Token::LBrace)?;
            let elsif_body = self.parse_stmt_list()?;
            self.expect(Token::RBrace)?;
            elsif_blocks.push((elsif_cond, elsif_body));
        }

        let else_block = if self.at(&Token::Else) {
            self.advance();
            self.expect(Token::LBrace)?;
            let body = self.parse_stmt_list()?;
            self.expect(Token::RBrace)?;
            Some(body)
        } else {
            None
        };

        Ok(Stmt::If {
            cond,
            then_block,
            elsif_blocks,
            else_block,
        })
    }

    fn parse_unless(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'unless'
        self.expect(Token::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;
        let then_block = self.parse_stmt_list()?;
        self.expect(Token::RBrace)?;

        let else_block = if self.at(&Token::Else) {
            self.advance();
            self.expect(Token::LBrace)?;
            let body = self.parse_stmt_list()?;
            self.expect(Token::RBrace)?;
            Some(body)
        } else {
            None
        };

        Ok(Stmt::Unless {
            cond,
            then_block,
            else_block,
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'while'
        self.expect(Token::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;
        let body = self.parse_stmt_list()?;
        self.expect(Token::RBrace)?;

        Ok(Stmt::While { cond, body })
    }

    fn parse_until(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'until'
        self.expect(Token::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;
        let body = self.parse_stmt_list()?;
        self.expect(Token::RBrace)?;

        Ok(Stmt::Until { cond, body })
    }

    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'for'

        // Check if it's a C-style for or foreach-style
        if self.at(&Token::My) || matches!(self.current(), Token::ScalarVar(_)) {
            // Could be foreach-style: for my $x (...)
            return self.parse_foreach_style();
        }

        self.expect(Token::LParen)?;

        let init = if !self.at(&Token::Semicolon) {
            Some(Box::new(self.parse_statement()?))
        } else {
            self.advance();
            None
        };

        let cond = if !self.at(&Token::Semicolon) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(Token::Semicolon)?;

        let step = if !self.at(&Token::RParen) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(Token::RParen)?;

        self.expect(Token::LBrace)?;
        let body = self.parse_stmt_list()?;
        self.expect(Token::RBrace)?;

        Ok(Stmt::For { init, cond, step, body })
    }

    fn parse_foreach(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'foreach'
        self.parse_foreach_style()
    }

    fn parse_foreach_style(&mut self) -> Result<Stmt, String> {
        // Optional 'my'
        if self.at(&Token::My) {
            self.advance();
        }

        let var = match self.current().clone() {
            Token::ScalarVar(name) => {
                self.advance();
                name
            }
            _ => return Err(format!("Expected variable, got {:?}", self.current())),
        };

        self.expect(Token::LParen)?;
        let list = self.parse_expr()?;
        self.expect(Token::RParen)?;

        self.expect(Token::LBrace)?;
        let body = self.parse_stmt_list()?;
        self.expect(Token::RBrace)?;

        Ok(Stmt::Foreach { var, list, body })
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'return'
        let value = if !self.at(&Token::Semicolon) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(Token::Semicolon)?;
        Ok(Stmt::Return(value))
    }

    fn parse_print(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'print'
        let args = self.parse_expr_list()?;
        self.expect(Token::Semicolon)?;
        Ok(Stmt::Print(args))
    }

    fn parse_say(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'say'
        let args = self.parse_expr_list()?;
        self.expect(Token::Semicolon)?;
        Ok(Stmt::Say(args))
    }

    fn parse_use(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'use'
        let name = match self.current().clone() {
            Token::Ident(n) => {
                self.advance();
                n
            }
            _ => return Err(format!("Expected module name, got {:?}", self.current())),
        };
        self.expect(Token::Semicolon)?;
        Ok(Stmt::Use(name))
    }

    fn parse_package(&mut self) -> Result<Stmt, String> {
        self.advance(); // consume 'package'
        let name = match self.current().clone() {
            Token::Ident(n) => {
                self.advance();
                n
            }
            _ => return Err(format!("Expected package name, got {:?}", self.current())),
        };
        self.expect(Token::Semicolon)?;
        Ok(Stmt::Package(name))
    }

    fn parse_block(&mut self) -> Result<Stmt, String> {
        self.expect(Token::LBrace)?;
        let stmts = self.parse_stmt_list()?;
        self.expect(Token::RBrace)?;
        Ok(Stmt::Block(stmts))
    }

    fn parse_stmt_list(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        while !self.at(&Token::RBrace) && !self.at(&Token::Eof) {
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    fn parse_expr_list(&mut self) -> Result<Vec<Expr>, String> {
        let mut exprs = Vec::new();
        if !self.at(&Token::Semicolon) && !self.at(&Token::RParen) {
            exprs.push(self.parse_expr()?);
            while self.at(&Token::Comma) {
                self.advance();
                if !self.at(&Token::Semicolon) && !self.at(&Token::RParen) {
                    exprs.push(self.parse_expr()?);
                }
            }
        }
        Ok(exprs)
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr, String> {
        let left = self.parse_ternary()?;

        match self.current() {
            Token::Assign => {
                self.advance();
                let right = self.parse_assignment()?;
                Ok(Expr::Assign(Box::new(left), Box::new(right)))
            }
            Token::PlusEquals => {
                self.advance();
                let right = self.parse_assignment()?;
                Ok(Expr::OpAssign(Box::new(left), BinOp::Add, Box::new(right)))
            }
            Token::MinusEquals => {
                self.advance();
                let right = self.parse_assignment()?;
                Ok(Expr::OpAssign(Box::new(left), BinOp::Sub, Box::new(right)))
            }
            Token::StarEquals => {
                self.advance();
                let right = self.parse_assignment()?;
                Ok(Expr::OpAssign(Box::new(left), BinOp::Mul, Box::new(right)))
            }
            Token::SlashEquals => {
                self.advance();
                let right = self.parse_assignment()?;
                Ok(Expr::OpAssign(Box::new(left), BinOp::Div, Box::new(right)))
            }
            Token::DotEquals => {
                self.advance();
                let right = self.parse_assignment()?;
                Ok(Expr::OpAssign(Box::new(left), BinOp::Concat, Box::new(right)))
            }
            _ => Ok(left),
        }
    }

    fn parse_ternary(&mut self) -> Result<Expr, String> {
        let cond = self.parse_or()?;

        if self.at(&Token::Question) {
            self.advance();
            let then_expr = self.parse_expr()?;
            self.expect(Token::Colon)?;
            let else_expr = self.parse_ternary()?;
            Ok(Expr::Ternary(
                Box::new(cond),
                Box::new(then_expr),
                Box::new(else_expr),
            ))
        } else {
            Ok(cond)
        }
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;

        while matches!(self.current(), Token::Or | Token::OrWord) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinOp(Box::new(left), BinOp::Or, Box::new(right));
        }

        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;

        while matches!(self.current(), Token::And | Token::AndWord) {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinOp(Box::new(left), BinOp::And, Box::new(right));
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_additive()?;

        loop {
            // Check for regex match operators first
            if matches!(self.current(), Token::Match | Token::NotMatch) {
                let is_negated = matches!(self.current(), Token::NotMatch);
                self.advance();

                // Expect a regex pattern
                if let Token::Regex(pattern, flags) = self.current().clone() {
                    self.advance();
                    if is_negated {
                        left = Expr::NotMatch(Box::new(left), pattern, flags);
                    } else {
                        left = Expr::Match(Box::new(left), pattern, flags);
                    }
                    continue;
                } else {
                    return Err("Expected regex pattern after =~ or !~".to_string());
                }
            }

            let op = match self.current() {
                Token::Eq => BinOp::Eq,
                Token::Ne => BinOp::Ne,
                Token::Lt => BinOp::Lt,
                Token::Gt => BinOp::Gt,
                Token::Le => BinOp::Le,
                Token::Ge => BinOp::Ge,
                Token::Cmp => BinOp::Cmp,
                Token::StrEq => BinOp::StrEq,
                Token::StrNe => BinOp::StrNe,
                Token::StrLt => BinOp::StrLt,
                Token::StrGt => BinOp::StrGt,
                Token::StrLe => BinOp::StrLe,
                Token::StrGe => BinOp::StrGe,
                Token::StrCmp => BinOp::StrCmp,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::BinOp(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let op = match self.current() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                Token::Dot => BinOp::Concat,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinOp(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.current() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinOp(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.current() {
            Token::Not | Token::NotWord => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOp::Not, Box::new(expr)))
            }
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOp::Neg, Box::new(expr)))
            }
            Token::BitNot => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOp::BitNot, Box::new(expr)))
            }
            Token::Backslash => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Ref(Box::new(expr)))
            }
            Token::Increment => {
                self.advance();
                let expr = self.parse_postfix()?;
                Ok(Expr::PreIncrement(Box::new(expr)))
            }
            Token::Decrement => {
                self.advance();
                let expr = self.parse_postfix()?;
                Ok(Expr::PreDecrement(Box::new(expr)))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.current() {
                Token::Increment => {
                    self.advance();
                    expr = Expr::PostIncrement(Box::new(expr));
                }
                Token::Decrement => {
                    self.advance();
                    expr = Expr::PostDecrement(Box::new(expr));
                }
                Token::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(Token::RBracket)?;
                    expr = Expr::ArrayIndex(Box::new(expr), Box::new(index));
                }
                Token::LBrace => {
                    self.advance();
                    let key = self.parse_expr()?;
                    self.expect(Token::RBrace)?;
                    expr = Expr::HashIndex(Box::new(expr), Box::new(key));
                }
                Token::Arrow => {
                    self.advance();
                    match self.current() {
                        Token::LBracket => {
                            self.advance();
                            let index = self.parse_expr()?;
                            self.expect(Token::RBracket)?;
                            expr = Expr::ArrayIndex(Box::new(Expr::Deref(Box::new(expr))), Box::new(index));
                        }
                        Token::LBrace => {
                            self.advance();
                            let key = self.parse_expr()?;
                            self.expect(Token::RBrace)?;
                            expr = Expr::HashIndex(Box::new(Expr::Deref(Box::new(expr))), Box::new(key));
                        }
                        Token::Ident(name) => {
                            let name = name.clone();
                            self.advance();
                            let args = if self.at(&Token::LParen) {
                                self.advance();
                                let args = self.parse_expr_list()?;
                                self.expect(Token::RParen)?;
                                args
                            } else {
                                Vec::new()
                            };
                            expr = Expr::MethodCall(Box::new(expr), name, args);
                        }
                        _ => return Err(format!("Expected method or subscript after ->, got {:?}", self.current())),
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.current().clone() {
            Token::Integer(n) => {
                self.advance();
                Ok(Expr::Integer(n))
            }
            Token::Float(f) => {
                self.advance();
                Ok(Expr::Float(f))
            }
            Token::String(s) => {
                self.advance();
                Ok(Expr::String(s))
            }
            Token::ScalarVar(name) => {
                self.advance();
                Ok(Expr::ScalarVar(name))
            }
            Token::ArrayVar(name) => {
                self.advance();
                Ok(Expr::ArrayVar(name))
            }
            Token::HashVar(name) => {
                self.advance();
                Ok(Expr::HashVar(name))
            }
            Token::Ident(name) => {
                self.advance();
                if self.at(&Token::LParen) {
                    self.advance();
                    let args = self.parse_expr_list()?;
                    self.expect(Token::RParen)?;
                    Ok(Expr::Call(name, args))
                } else {
                    Ok(Expr::Call(name, Vec::new()))
                }
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Token::LBracket => {
                self.advance();
                let items = self.parse_expr_list()?;
                self.expect(Token::RBracket)?;
                Ok(Expr::List(items))
            }
            Token::LBrace => {
                self.advance();
                let mut pairs = Vec::new();
                while !self.at(&Token::RBrace) {
                    let key = self.parse_expr()?;
                    self.expect(Token::FatArrow)?;
                    let value = self.parse_expr()?;
                    pairs.push((key, value));
                    if self.at(&Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::RBrace)?;
                Ok(Expr::Hash(pairs))
            }
            _ => Err(format!("Unexpected token in expression: {:?}", self.current())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_expr(code: &str) -> Result<Expr, String> {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse_expr()
    }

    fn parse_program(code: &str) -> Result<Program, String> {
        let mut lexer = Lexer::new(code);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    // === Match expression tests ===

    #[test]
    fn test_parse_match_simple() {
        let expr = parse_expr("$x =~ /hello/").unwrap();
        match expr {
            Expr::Match(subject, pattern, flags) => {
                assert!(matches!(*subject, Expr::ScalarVar(s) if s == "x"));
                assert_eq!(pattern, "hello");
                assert!(flags.is_empty());
            }
            _ => panic!("Expected Match expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_not_match_simple() {
        let expr = parse_expr("$x !~ /world/").unwrap();
        match expr {
            Expr::NotMatch(subject, pattern, flags) => {
                assert!(matches!(*subject, Expr::ScalarVar(s) if s == "x"));
                assert_eq!(pattern, "world");
                assert!(flags.is_empty());
            }
            _ => panic!("Expected NotMatch expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_match_with_flags() {
        let expr = parse_expr("$s =~ /pattern/gi").unwrap();
        match expr {
            Expr::Match(_, pattern, flags) => {
                assert_eq!(pattern, "pattern");
                assert_eq!(flags, "gi");
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_parse_match_empty_pattern() {
        let expr = parse_expr("$x =~ //").unwrap();
        match expr {
            Expr::Match(_, pattern, _) => {
                assert!(pattern.is_empty());
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_parse_match_with_wildcard() {
        let expr = parse_expr("$x =~ /h.llo/").unwrap();
        match expr {
            Expr::Match(_, pattern, _) => {
                assert_eq!(pattern, "h.llo");
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_parse_match_complex_pattern() {
        let expr = parse_expr("$x =~ /^[a-z]+$/").unwrap();
        match expr {
            Expr::Match(_, pattern, _) => {
                assert_eq!(pattern, "^[a-z]+$");
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_parse_match_in_if_condition() {
        let program = parse_program("if ($x =~ /test/) { print 1; }").unwrap();
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::If { cond, .. } => {
                assert!(matches!(cond, Expr::Match(_, _, _)));
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_parse_not_match_in_if_condition() {
        let program = parse_program("if ($x !~ /bad/) { print 1; }").unwrap();
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::If { cond, .. } => {
                assert!(matches!(cond, Expr::NotMatch(_, _, _)));
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_parse_match_with_and() {
        let expr = parse_expr("$a =~ /one/ && $b =~ /two/").unwrap();
        match expr {
            Expr::BinOp(left, BinOp::And, right) => {
                assert!(matches!(*left, Expr::Match(_, _, _)));
                assert!(matches!(*right, Expr::Match(_, _, _)));
            }
            _ => panic!("Expected And expression"),
        }
    }

    #[test]
    fn test_parse_match_with_or() {
        let expr = parse_expr("$a =~ /one/ || $b =~ /two/").unwrap();
        match expr {
            Expr::BinOp(left, BinOp::Or, right) => {
                assert!(matches!(*left, Expr::Match(_, _, _)));
                assert!(matches!(*right, Expr::Match(_, _, _)));
            }
            _ => panic!("Expected Or expression"),
        }
    }

    #[test]
    fn test_parse_match_mixed_not() {
        let expr = parse_expr("$a =~ /yes/ && $b !~ /no/").unwrap();
        match expr {
            Expr::BinOp(left, BinOp::And, right) => {
                assert!(matches!(*left, Expr::Match(_, _, _)));
                assert!(matches!(*right, Expr::NotMatch(_, _, _)));
            }
            _ => panic!("Expected And expression"),
        }
    }

    #[test]
    fn test_parse_match_string_literal() {
        let expr = parse_expr("\"hello\" =~ /ell/").unwrap();
        match expr {
            Expr::Match(subject, pattern, _) => {
                assert!(matches!(*subject, Expr::String(s) if s == "hello"));
                assert_eq!(pattern, "ell");
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_parse_match_preserves_escapes() {
        let expr = parse_expr(r#"$x =~ /\d+/"#).unwrap();
        match expr {
            Expr::Match(_, pattern, _) => {
                assert_eq!(pattern, r"\d+");
            }
            _ => panic!("Expected Match expression"),
        }
    }

    // === Edge cases ===

    #[test]
    fn test_parse_division_not_regex() {
        // This should parse as division, not regex
        let expr = parse_expr("$x / 2").unwrap();
        match expr {
            Expr::BinOp(_, BinOp::Div, _) => {}
            _ => panic!("Expected division, got {:?}", expr),
        }
    }

    #[test]
    fn test_parse_match_followed_by_semicolon() {
        let program = parse_program("$x =~ /test/;").unwrap();
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::Expr(expr) => {
                assert!(matches!(expr, Expr::Match(_, _, _)));
            }
            _ => panic!("Expected Expr statement"),
        }
    }

    #[test]
    fn test_parse_my_with_match_init() {
        // Matching on initialization value
        let program = parse_program("my $x = \"hello\"; $x =~ /ell/;").unwrap();
        assert_eq!(program.statements.len(), 2);
    }

    #[test]
    fn test_parse_while_with_match() {
        let program = parse_program("while ($line =~ /data/) { print $line; }").unwrap();
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::While { cond, .. } => {
                assert!(matches!(cond, Expr::Match(_, _, _)));
            }
            _ => panic!("Expected While statement"),
        }
    }
}
