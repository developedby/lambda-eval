use crate::ast::{Book, Name, Term};

impl Term {
  pub fn reduce(
    &mut self,
    book: &Book,
    stop: impl Fn(&Term) -> bool,
    step: impl Fn(&mut Term, &Book) -> bool,
  ) {
    while !stop(self) {
      let reduced = step(self, book);
      if !reduced {
        eprintln!("no redex left");
        break;
      }
    }
  }

  pub fn reduce_stepped(
    &mut self,
    book: &Book,
    stop: impl Fn(&Term) -> bool,
    step: impl Fn(&mut Term, &Book) -> bool,
  ) -> Vec<Term> {
    let mut steps = vec![self.clone()];
    while !stop(self) {
      let reduced = step(self, book);
      if reduced {
        steps.push(self.clone());
      } else {
        break;
      }
    }
    steps
  }

  pub fn stop_whnf(&self) -> bool {
    match self {
      Term::Lam { .. } => true,
      Term::Var { .. } => true,
      Term::App { .. } => false,
      Term::Ref { .. } => true,
    }
  }

  pub fn stop_nf(&self) -> bool {
    false
  }

  pub fn normal_order_step(&mut self, book: &Book) -> bool {
    match self {
      Term::Lam { nam: _, bod } => bod.normal_order_step(book),
      Term::Var { nam: _ } => false,
      Term::App { fun, arg } => match fun.as_mut() {
        Term::Lam { nam, bod } => {
          if let Some(nam) = nam {
            bod.subst(nam, arg);
          }
          let bod = std::mem::take(bod.as_mut());
          *self = bod;
          true
        }
        Term::Var { .. } => false,
        Term::App { .. } => fun.normal_order_step(book) || arg.normal_order_step(book),
        Term::Ref { nam } => {
          *fun.as_mut() = book.0[nam].bod.clone();
          true
        }
      },
      Term::Ref { nam } => {
        *self = book.0[nam].bod.clone();
        true
      }
    }
  }

  pub fn subst(&mut self, nam: &Name, val: &Term) {
    match self {
      Term::Lam { nam: Some(lam_nam), bod: _ } if lam_nam == nam => (),
      Term::Lam { nam: _, bod } => bod.subst(nam, val),
      Term::Var { nam: var_nam } if var_nam == nam => {
        *self = val.clone();
      }
      Term::Var { nam: _ } => (),
      Term::App { fun, arg } => {
        fun.subst(nam, val);
        arg.subst(nam, val);
      }
      Term::Ref { nam: _ } => (),
    }
  }
}
