use crate::tokens::Token;
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
    },
}

#[derive(Debug)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Block(Vec<Stmt>),
    Var { name: Token, initializer: Option<Expr> },
}

#[derive(Debug)]
pub struct Program(pub Vec<Stmt>);


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


impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stmt::Expression(e) => write!(f, "{e}"),
            Stmt::Print(e)      => write!(f, "(print {e})"),
            Stmt::Block(stmts) => {
                write!(f, "(block")?;
                for s in stmts {
                    write!(f, " {s}")?;
                }
                write!(f, ")")
            }
            Stmt::Var { name, initializer } => match initializer {
                Some(init)  => write!(f, "(var {} {init})", name.lexeme),
                None        => write! (f, "(var {})", name.lexeme),   
            },
        }
    }
    
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for stmt in &self.0 {
            writeln!(f, "{stmt}")?;
        }
        Ok(())
    }
}