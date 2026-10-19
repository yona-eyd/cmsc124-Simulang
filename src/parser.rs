use crate::tokens::{Literal, Token, TokenList};
use std::fmt;
use crate::ast::Expr;

//----------------errors-----------------------

//syntax error
#[derive(Debug, Clone)]
pub struct ParseError {
    pub line: usize,
    pub location: String,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}] Error {}: {}", self.line, self.location, self.message)
    }
}

//result type used by every grammar rule
pub(crate) type PResult<T> = Result<T, ParseError>;

//-------------------parser-------------------------

pub struct Parser {
    pub(crate)tokens: Vec<Token>,
    pub(crate) current: usize, //next token to consume
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens, 
            current: 0, 
            errors: Vec::new() }
    }

    //parses one expression with required semicolon ';'
    pub fn parse_line(mut self) -> Result<Expr, Vec<ParseError>> {
    let expr = match self.statement() {
        Ok(e) => e,
        Err(e) => { 
            self.errors.push(e); 
            return Err(self.errors); 
        }
    };

    //anything left over is an error
    if !self.at_end() {
        let e = self.error_at(self.peek(), "Expect end of input after ';'");
        self.errors.push(e);
    }
    if self.errors.is_empty() { Ok(expr) } else { Err(self.errors) }
    }

    //parses a whole program: ( expression ";" )* EOF
    //calls synchronize() after syntax errors to skip a statement
    pub fn parse(mut self) -> Result<Expr, Vec<ParseError>> {
        let mut exprs = Vec::new();
        while !self.at_end() {
            match self.statement() {
                Ok(expr) => exprs.push(expr),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize();
                }
            }
        }
        if self.errors.is_empty() { 
            Ok(Expr::Program(exprs))    // refactored, now only returns a single Expr
        } else { 
            Err(self.errors) 
        }
    }

    // ** RECURSIVE DESCENT FUNCTIONS **
    
    //statement -> expression ";"
    fn statement(&mut self) -> PResult<Expr> {
        let expr = self.expression()?;
        self.consume(TokenList::Semicolon, "Expect ';' after expression.")?;
        Ok(expr)
    }
    // expression → assignment
    pub fn expression(&mut self) -> PResult<Expr> {
        self.assignment()
    }

    //assignment -> IDENTIFIER "=" assignment | logic_or (right associative)
    fn assignment(&mut self) -> PResult<Expr>{
        let expr = self.logic_or()?;
        if self.matches(&[TokenList::Assign]){
            let equals = self.previous().clone();
            let value = self.assignment()?;

            return match expr {
                Expr::Variable(name) => Ok(Expr::Assign {name, value:Box::new(value)}),
                _ => {
                    //report error but keep parsing
                    //target is just invalid
                    let err = self.error_at(&equals, "Invalid assignment target.");
                    self.errors.push(err);
                    Ok(expr)
                }
            };
        }
        Ok(expr)
    }

    //logic_or -> logic_and ( "or" logic_and)*
    fn logic_or(&mut self) -> PResult<Expr> {
        let mut expr = self.logic_and()?;
        while self.matches(&[TokenList::Or]){
            let op = self.previous().clone();
            let right = self.logic_and()?;
            expr = Expr::Logical {left: Box::new(expr), op, right: Box::new(right)};
        }
        Ok(expr)
    }

    //logic_and -> equality ("and" equality)*
    fn logic_and(&mut self) -> PResult<Expr> {
        let mut expr = self.equality()?;
        while self.matches(&[TokenList::And]){
            let op = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Logical {left: Box::new(expr), op, right: Box::new(right)};
        }
        Ok(expr)
    }

    //shared loop for every left-associative binary level
    //ops (operators at this level), next(parses the tighter level)
    fn binary_level(
        &mut self,
        ops: &[TokenList],
        next: fn(&mut Self) -> PResult<Expr>,
    ) -> PResult<Expr> {
        let mut expr = next(self)?;
        while self.matches(ops){
            let op = self.previous().clone();
            let right = next(self)?;
            expr = Expr::Binary {left: Box::new(expr), op, right:Box::new(right)};
        }
        Ok(expr)
    }

    //equality ->comparison
    fn equality(&mut self) -> PResult<Expr> {
        self.binary_level(&[TokenList::NotEqual, TokenList::EqualTo], Self::comparison)
    }

    //comparison -> term
    fn comparison(&mut self) -> PResult<Expr> {
        self.binary_level(&[TokenList::Greater, TokenList::GreaterEql, TokenList::Less, TokenList::LessEql], Self::term)
    }

    //term -> factor
    fn term(&mut self) -> PResult<Expr> {
        self.binary_level (&[TokenList::Minus, TokenList::Plus], Self::factor)
        }
    
    
    // factor → unary
    fn factor(&mut self) -> PResult<Expr> {
        self.binary_level(
            &[TokenList::Slash, TokenList::Star, TokenList::Modulo], Self::unary,
        )
    }

    //unary -> unary|primary
    fn unary(&mut self) -> PResult<Expr> {
        if self.matches(&[TokenList::Not, TokenList::Minus]) {
            let op = self.previous().clone();
            let right = self.unary()?; //right-associative
            return Ok(Expr::Unary {op, right:Box::new(right)});
        }
        self.primary()
    }

    //primary -> number | string |"true" |"false"|"nil" |identifier| ()
    fn primary(&mut self) -> PResult<Expr> {
        let tok = self.peek().clone();
        let expr = match tok.token_type {
            TokenList::False => Expr::Bool(false),
            TokenList::True => Expr::Bool(true),
            TokenList::Nil => Expr::Nil,
            TokenList::Number => match &tok.literal {
                Literal::Num(n) => Expr::Number(*n),
                _ => return Err(self.error_at(&tok, "Expect expression.")),
            },
            TokenList::StringLit => match &tok.literal {
                Literal::Str(s) => Expr::Str(s.clone()),
                _ => return Err(self.error_at (&tok, "Expect expression.")),
            },
            TokenList::Identifier => Expr::Variable(tok.clone()),
            TokenList::LeftParen => {
                //consumes "(" 
                self.advance();
                let inner = self.expression()?;
                self.consume(TokenList::RightParen, "Expect ')' after expression.")?;
                return Ok(Expr::Grouping(Box::new(inner)));
            }
            _ =>return Err(self.error_at(&tok, "Expect expression.")),
        };
        self.advance();
        Ok(expr)
    }
}


