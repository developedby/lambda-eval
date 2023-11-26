use core::fmt;
use std::collections::HashMap;

pub type Name = String;

#[derive(Debug, Clone)]
pub struct Book(pub HashMap<Name, Definition>);

#[derive(Debug, Clone)]
pub struct Definition {
  pub nam: Name,
  pub bod: Term,
}

#[derive(Debug, Clone)]
pub enum Term {
  Lam { nam: Option<Name>, bod: Box<Term> },
  Var { nam: Name },
  App { fun: Box<Term>, arg: Box<Term> },
  Ref { nam: Name },
}

impl fmt::Display for Term {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Term::Lam { nam, bod } => {
        write!(f, "λ{} {}", nam.as_ref().map(String::as_str).unwrap_or("*"), bod)
      }
      Term::Var { nam } => write!(f, "{nam}"),
      Term::App { fun, arg } => write!(f, "({} {})", fun, arg),
      Term::Ref { nam } => write!(f, "{nam}"),
    }
  }
}

impl Default for Term {
  fn default() -> Self {
    Term::Var { nam: String::new() }
  }
}
