use crate::parser::{PResult, ParseError, Parser};
use crate::tokens::{Token, TokenList};

impl Parser {
        // ** HELPER FUNCTIONS **
    
    // returns current (unconsumed) token
    pub(crate) fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    // returns token just consumed
    pub(crate) fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    // checks whether we're at EOF
    pub(crate) fn at_end(&self) -> bool {
        self.peek().token_type == TokenList::Engk
    }

    // consumes token and returns it to caller
    pub(crate) fn advance(&mut self) -> &Token {
        if !self.at_end() {
            self.current += 1;
        }
        self.previous()
    }

    // checks if cuttrnt token is of a certain type
    pub(crate) fn check(&self, token_type: TokenList) -> bool {
        if self.at_end() {
            return false;
        }
        self.peek().token_type == token_type
    }

    // ask for the token or throw error
    pub(crate) fn consume (&mut self, token_type: TokenList, message: &str) -> PResult<Token> {
        if self.check(token_type) {
            Ok(self.advance().clone())
        }
        else {
        Err(self.error_at(self.peek(), message))
        }
    }
    
    //consumes the current token if it is any of 'types' and return if it did
    pub(crate) fn matches(&mut self, types: &[TokenList]) -> bool {
        if types.iter().any(|t| self.check(t.clone())) {
            self.advance();
            true
        } else {
            false
        }
    }

    

    //-------------------error recovery---------------
    //builds a parser error
    pub(crate) fn error_at(&self, token: &Token, message: &str) -> ParseError{
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

    
    pub(crate) fn synchronize(&mut self) {
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
}