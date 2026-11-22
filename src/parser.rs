use crate::tokens::{Literal, Token, TokenList};
use std::fmt;
use crate::ast::{Expr, FunctionDecl, Program, Stmt};
use std::rc::Rc;

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

    //parses one declaration or statement with required semicolon ';'
    pub fn parse_line(mut self) -> Result<Stmt, Vec<ParseError>> {
    let expr = match self.declaration() {
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

    //parses a whole program: declaration* EOF
    //calls synchronize() after syntax errors to skip a statement
    pub fn parse(mut self) -> Result<Program, Vec<ParseError>> {
        let mut stmts = Vec::new();
        while !self.at_end() {
            match self.declaration() {
                Ok(stmt) => stmts.push(stmt),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize();
                }
            }
        }
        if self.errors.is_empty() { Ok(Program(stmts)) } else { Err(self.errors) }
    }

    // ** RECURSIVE DESCENT FUNCTIONS **
    
    // declaration -> varDecl | statement
    fn declaration(&mut self) -> PResult<Stmt> {
        if self.matches(&[TokenList::Var]) {
            return self.var_declaration();
        }
        // law is a declaration, not a statement
        // so it is legal only where declaration() is called: top level + inside blocks
        if self.matches(&[TokenList::Law]) {
            return self.function();
        }
        self.statement()
    }

    // varDecl -> "var" IDENTIFIER "=" expression ";"
    fn var_declaration(&mut self) -> PResult<Stmt> {
        let name = self.consume(TokenList::Identifier, "Expect variable name.")?;
        self.consume(TokenList::Assign, "Expect '=' after variable name.")?;
        let initializer = self.expression()?;
        self.consume(TokenList::Semicolon, "Expect ';' after variable declaration.")?;
        Ok(Stmt::Var { name, initializer })
    }

    // funDecl -> "law" IDENTIFIER "(" parameters? ")" block
    fn function(&mut self) -> PResult<Stmt> {
        let name = self.consume(TokenList::Identifier, "Expect function name.")?;
        self.consume(TokenList::LeftParen, "Expect '(' after function name.")?;
        let mut params = Vec::new();
        if !self.check(TokenList::RightParen) {
            loop {
                params.push(self.consume(TokenList::Identifier, "Expect parameter name.")?);
                if !self.matches(&[TokenList::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenList::RightParen, "Expect ')' after parameters.")?;
        // block() expects "{" to be consumed already
        // but it's the one who consumes "}"
        self.consume(TokenList::LeftBrace, "Expect '{' before function body.")?;
        let body = self.block()?;
        Ok(Stmt::Function(Rc::new(FunctionDecl { name, params, body })))
    }

    // block -> "{" statement* "}"
    fn block(&mut self) -> PResult<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while !self.check(TokenList::RightBrace) && !self.at_end() {
            match self.declaration() {         
                Ok(s) => stmts.push(s),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize();
                }
            }
        }
        self.consume(TokenList::RightBrace, "Expect '}' after block.")?;
        Ok(stmts)
    }

    // statement -> printStmt | exprStmt
    fn statement(&mut self) -> PResult<Stmt> {
        if self.matches(&[TokenList::Print]) {
            let value = self.expression()?;
            self.consume(TokenList::Semicolon, "Expect ';' after value.")?;
            return Ok(Stmt::Print(value));
        }
        if self.matches(&[TokenList::LeftBrace]) {
            return Ok(Stmt::Block(self.block()?));
        }
        if self.matches(&[TokenList::When]) {
            return self.when_statement();
        }
        if self.matches(&[TokenList::Evolve]) {
            return self.evolve_statement();
        }
        if self.matches(&[TokenList::Return]) {
            return self.return_statement();
        }
        let expr = self.expression()?;
        self.consume(TokenList::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(expr))
    }

    fn when_statement(&mut self) -> PResult<Stmt> {
        self.consume(TokenList::LeftParen, "Expect '(' after 'when'.")?;
        let condition = self.expression()?;
        self.consume(TokenList::RightParen, "Expect ')' after condition.")?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.matches(&[TokenList::Otherwise]) {
            Some(Box::new(self.statement()?))
        } else {
            None   
        };
        Ok(Stmt::When { condition, then_branch, else_branch })
    }

    fn evolve_statement(&mut self) -> PResult<Stmt> {
        // standard while
        if self.matches(&[TokenList::Until]) {
            self.consume(TokenList::LeftParen, "Expect '(' after 'until'.")?;
            let condition = self.expression()?;
            self.consume(TokenList::RightParen, "Expect ')' after condition.")?;
            let body = Box::new(self.statement()?);
            return Ok(Stmt::Evolve { body, condition, check_first: true });
        }

        // do-while
        let body = Box::new(self.statement()?);
        self.consume(TokenList::Until, "Expect 'until' after loop body.")?;
        self.consume(TokenList::LeftParen, "Expect '(' after 'until'.")?;
        let condition = self.expression()?;
        self.consume(TokenList::RightParen, "Expect ')' after condition.")?;
        self.consume(TokenList::Semicolon, "Expect ';' after expression.")?;
        return Ok(Stmt::Evolve { body, condition, check_first: false });
    }

    // returnStmt -> "return" expression? ";"
    fn return_statement(&mut self) -> PResult<Stmt> {
        // "return" token was jsut matched, so it is now previous()
        let keyword = self.previous().clone();
        // a bare 'return;' has no value; it will evaluate to nil at runtime
        let value = if !self.check(TokenList::Semicolon) {
            Some(self.expression()?)
        }
        else {
            None
        };
        self.consume(TokenList::Semicolon, "Expect ';' after return value.")?;
        Ok(Stmt::Return { keyword, value })
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
        self.call()
    }  
    
    // call -> primary ( "(" arguments? ")" )*
    fn call(&mut self) -> PResult<Expr> {
        let mut expr = self.primary()?;
        while self.matches(&[TokenList::LeftParen]) {
            expr = self.finish_call(expr)?;
        }
        Ok(expr)
    }

    // parses part after "(": arguments? ")"
    // split from above for readability
    fn finish_call(&mut self, callee: Expr) -> PResult<Expr> {
        let mut arguments = Vec::new();
        // guard, what allows zero arguments like f()
        if !self.check(TokenList::RightParen) {
            loop {
                arguments.push(self.expression()?);     // read one item
                if !self.matches(&[TokenList::Comma]) {
                    break;      // no comma = list is over
                }
            }
        }
        let paren = self.consume(TokenList::RightParen, "Expect ')' after arguments.")?;
        Ok(Expr::Call { callee: Box::new(callee), paren, arguments })
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


