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
    Assign {
        name:   Token,
        value: Box<Expr>
    },
}

#[derive(Debug)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Block(Vec<Stmt>),
    Var { name: Token, initializer: Expr },
    When {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>
    },
    Evolve {
        body: Box<Stmt>,
        condition: Expr,
        check_first: bool
    },
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
            Stmt::Var { name, initializer } => write!(f, "(var {} {initializer})", name.lexeme),
            Stmt::When { condition,then_branch, else_branch } => match else_branch {
                Some(e) => write!(f, "when {condition} {then_branch} {e}"),
                None    => write!(f, "(when {condition} {then_branch})"),
            }
            Stmt::Evolve { body, condition, check_first } => {
                if *check_first {
                    write!(f, "(evolve-until {condition} {body})")      // while, might not execute at all
                } else {
                    write!(f, "(evolve {body} until {condition})")      // do-while, executes at least once
                }
            }
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