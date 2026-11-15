use crate :: tokens :: {Literal, Token, TokenList};

pub struct Scanner {
    source: Vec<char>,
    start: usize,
    current: usize,
    line: usize,
    emitted_eof: bool,
}

// scan error is returned as data instead of being printed on the spot
#[derive(Debug)]
pub struct ScanError {
    pub line: usize,
    pub message: String,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Scanner {
            source: source.chars().collect(),
            start: 0,
            current: 0,
            line: 1,
            emitted_eof: false,
        }
    }

    fn at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current +=1;
        c
    }

    //answers the next character without consuming it
    fn peek(&self) -> char {
        if self.at_end() {'\0'} else {self.source[self.current]}
    }

    //one char further than peek()
    //exists for numeric literals where it might be a float
    fn peek_next(&self) -> char {
        if self.current +1 >= self.source.len() {'\0'} else {self.source[self.current +1]}
    }

    //advance only if the next char matches
    //helps form two-char operators without misreading single chars
    fn peek_match(&mut self, expected: char) -> bool {
        if self.at_end() || self.source[self.current] != expected {
            return false
        }
        self.current += 1;
        true
    }

    //shared helper for deciding whether it's two-char or one-char 
    //avoids repeating if-else statements
    fn one_or_two(&mut self, second:char, one: TokenList, two:TokenList) -> TokenList {
        if self.peek_match(second){two} else {one}
    }

    // slice the source once to build the lexeme
    // keeps scan arms simple and avoisds per‑char accumulation.

    fn make_token(&self, token_type: TokenList) -> Token {
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        Token {
            token_type,
            lexeme,
            literal: Literal::None,
            line: self.line,
        }
    }

    // variant of make_token for tokens that carry a real value
    // so that operators can use the simpler make_token without writing Literal::None everywhere
    fn make_token_literal(&self, token_type:TokenList, literal: Literal) -> Token {
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        Token {
            token_type, 
            lexeme, 
            literal, 
            line: self.line}
    }
    //for number literals
    //consumes digits
    fn number (&mut self) -> Token {
        while self.peek().is_ascii_digit(){
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit(){
            self.advance();
            while self.peek().is_ascii_digit(){
                self.advance();
            }
        }
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        let value: f64 = lexeme.parse().expect("scanned number must be valid");
        self.make_token_literal(TokenList::Number, Literal::Num(value))
    }

    //for string literals
    fn string (&mut self) -> Result<Token, ScanError> {
        let start_line = self.line;
        while self.peek() != '"' && !self.at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.at_end() {
            return Err(ScanError {
                line: start_line,
                message: "Unterminated string.".to_string(),
            });
        }
        self.advance(); //consume closing quote
        let value: String = self.source[self.start + 1..self.current - 1].iter().collect(); //source between quotes
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        Ok(Token {
            token_type: TokenList::StringLit,
            lexeme,
            literal: Literal::Str(value),
            line: start_line,
        })
    }

    //for identifiers, consume all characters then check keyword table
    //checking as it goes would misidentify identifiers that starts with keywords
    fn identifier(&mut self) -> Token {
        while is_identifier_continue(self.peek()) {
            self.advance();
        }
        let text:String = self.source[self.start..self.current].iter().collect();
        let token_type = keyword_type(&text).unwrap_or(TokenList::Identifier);
        self.make_token(token_type)
    }


    fn scan_token(&mut self) -> Result<Option<Token>, ScanError>{
        let c = self.advance();
        
        let token = match c {
            '(' => Some(self.make_token(TokenList::LeftParen)),
            ')' => Some(self.make_token(TokenList::RightParen)),
            '{' => Some(self.make_token(TokenList::LeftBrace)),
            '}' => Some(self.make_token(TokenList::RightBrace)),
            ',' => Some(self.make_token(TokenList::Comma)),
            '.' => Some(self.make_token(TokenList::Dot)),
            '+' => Some(self.make_token(TokenList::Plus)),
            '*' => Some(self.make_token(TokenList::Star)),
            '%' => Some(self.make_token(TokenList::Modulo)),
            ';' => Some(self.make_token(TokenList::Semicolon)),
            ':' => Some(self.make_token(TokenList::Colon)),


            '-' => {let kind = self.one_or_two('>',TokenList::Minus, TokenList::Arrow); 
                    Some(self.make_token(kind))}
            '!' => {let kind = self.one_or_two('=',TokenList::Not, TokenList::NotEqual); 
                    Some(self.make_token(kind))}
            '=' => {let kind = self.one_or_two('=',TokenList::Assign, TokenList::EqualTo); 
                    Some(self.make_token(kind))}
            '>' => {let kind = self.one_or_two('=',TokenList::Greater, TokenList::GreaterEql); 
                    Some(self.make_token(kind))}
            '<' => {let kind = self.one_or_two('=',TokenList::Less, TokenList::LessEql); 
                    Some(self.make_token(kind))}
            


            '/' => {
                if self.peek_match('/') {
                    while self.peek() != '\n' && !self.at_end() {
                        self.advance();
                    }
                    None // if comment, discard
                } else {
                    Some(self.make_token(TokenList::Slash))
                }
            }
            
            ' ' | '\r' | '\t' => None,
            '\n' =>{self.line += 1;
                    None }

            '"' => {
                    let token = self.string()?;
                    Some(token)}

            c if c.is_ascii_digit() => Some(self.number()),
            c if is_identifier_start(c) => Some(self.identifier()),


            other => {
                return Err(ScanError {
                    line: self.line,
                    message: format!("Unexpected character '{other}'"),
                })
            }

        };
        Ok(token)
    }
}

fn is_identifier_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_identifier_continue(c:char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn keyword_type(text: &str) -> Option<TokenList> {
    Some(match text {
        "and" => TokenList::And,
        "false" => TokenList::False,
        "true" => TokenList::True,
        "evolve" => TokenList::Evolve,
        "until" => TokenList::Until,
        "when" => TokenList::When,
        "otherwise" => TokenList::Otherwise,
        "nil" => TokenList::Nil,
        "or" => TokenList::Or,
        "print" => TokenList::Print,
        "return" => TokenList::Return,
        "var" => TokenList::Var,
        "law" => TokenList::Law,
        "entity" => TokenList::Entity,
        _ => return None,
    })
}

impl Iterator for Scanner {
    type Item = Result<Token, ScanError>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.at_end() {
                if !self.emitted_eof {
                    self.emitted_eof = true;
                    self.start = self.current;
                    return Some(Ok(self.make_token(TokenList::Engk)));
                }
                return None;
            }
            self.start = self.current;
            match self.scan_token() {
                Ok(Some(token)) => return Some(Ok(token)),
                Ok(None)        => continue,
                Err(e)          => return Some(Err(e)),
            }
        }
    }
}

impl Scanner {
    pub fn scan_tokens(self) -> Result<Vec<Token>, Vec<ScanError>> {
        let mut tokens  = Vec::new();
        let mut errors  = Vec::new();
        for result in self {
            match result {
                Ok(token)   => tokens.push(token),
                Err(e)      => errors.push(e),
            }
        }
        if errors.is_empty() { Ok(tokens) } else { Err(errors) }
    }
}