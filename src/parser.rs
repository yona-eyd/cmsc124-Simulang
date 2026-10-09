use crate::tokens::{Literal, Token, TokenList};
use std::fmt;

//---------------AST------------------------
//expression node, printed in prefix form
#[derive(Debug)]
pub enum Expr {
    Number(f64),
    Str(String),
    Bool(bool),
    Nil,
    Variable(Token), //variable reference so errors can cite token
    Grouping(Box<Expr>),
    Unary { op: Token, right: Box<Expr>},

    //arithmetic, comparison, and equality operators
    Binary {
        left:   Box<Expr>,
        op:     Token,
        right:  Box<Expr>,
    },

    Logical {
        left:   Box<Expr>,
        op:     Token,
        right:  Box<Expr>
    },
    Assign{
        name:   Token,
        value: Box<Expr>
    }
}

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
type PResult<T> = Result<T, ParseError>;

//-------------------parser-------------------------

pub struct Parser {
    tokens: Vec<Token>,
    current: usize, //next token to consume
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
    let expr = match self.expression() {
        Ok(e) => e,
        Err(e) => { 
            self.errors.push(e); 
            return Err(self.errors); 
        }
    };

    //anything left over is an error
    if !self.at_end() {
        let e = self.error_at(self.peek(), "Expect end of expression.");
        self.errors.push(e);
    }
    if self.errors.is_empty() { Ok(expr) } else { Err(self.errors) }
    }

    //parses a whole program: ( expression ";" )* EOF
    //calls synchronize() after syntax errors to skip a statement
    pub fn parse(mut self) -> Result<Vec<Expr>, Vec<ParseError>> {
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
            Ok(exprs) 
        } else { 
            Err(self.errors) 
        }
    }

    
    // ** HELPER FUNCTIONS **
    
    // returns current (unconsumed) token
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    // returns token just consumed
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    // checks whether we're at EOF
    fn at_end(&self) -> bool {
        self.peek().token_type == TokenList::Engk
    }

    // consumes token and returns it to caller
    fn advance(&mut self) -> &Token {
        if !self.at_end() {
            self.current += 1;
        }
        self.previous()
    }

    // checks if cuttrnt token is of a certain type
    fn check(&self, token_type: TokenList) -> bool {
        if self.at_end() {
            return false;
        }
        self.peek().token_type == token_type
    }

    // ask for the token or throw error
    fn consume (&mut self, token_type: TokenList, message: &str) -> PResult<Token> {
        if self.check(token_type) {
            Ok(self.advance().clone())
        }
        else {
        Err(self.error_at(self.peek(), message))
        }
    }
    
    //consumes the current token if it is any of 'types' and return if it did
    fn matches(&mut self, types: &[TokenList]) -> bool {
        if types.iter().any(|t| self.check(t.clone())) {
            self.advance();
            true
        } else {
            false
        }
    }

    //-------------------error recovery---------------
    //builds a parser error
    fn error_at(&self, token: &Token, message: &str) -> ParseError{
        let location = if token.token_type == TokenList::Engk {
            "at end".to_string()
        } else {
            format!("at '{}'", token.lexeme)
        };
        ParseError {
            line: token.line, 
            location, 
            message: message.to_string()
        }
    }

    
    fn synchronize(&mut self) {
        self.advance(); // always make progress
        while !self.at_end() {
            if self.previous().token_type == TokenList::Semicolon {
                return;
            }
            match self.peek().token_type {
                TokenList::Var
                | TokenList::Print
                | TokenList::Return
                | TokenList::When
                | TokenList::Until
                | TokenList::Evolve
                | TokenList::Law
                | TokenList::Entity => return,
                _ => {}
            }
            self.advance();
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

//--------- '--parse' printer ------------------

//prefix form
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr:: Number(n) => write!(f, "{n:?}"),
            Expr:: Str(s) => write!(f, "\"{s}\""),
            Expr:: Bool(b) => write!(f, "{b}"),
            Expr:: Nil =>write!(f, "nil"),
            Expr:: Variable(name) => write!(f, "{}", name.lexeme),
            Expr::Unary {op, right} => write!(f, "({} {right})", op.lexeme),
            Expr::Binary {left,op, right} | Expr::Logical{left, op, right} => {write! (f, "({} {left} {right})", op.lexeme)}
            Expr:: Assign {name, value} => write!(f, "(= {} {value})", name.lexeme),
            Expr::Grouping(e) => write!(f, "(group {e})"),
        }
    }
}

