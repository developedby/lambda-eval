use std::collections::{HashMap, HashSet};

use crate::ast::{Book, Name, Term};

impl Book {
  pub fn resolve_refs(&mut self) {
    let def_names = HashSet::from_iter(self.0.keys().cloned());
    for def in self.0.values_mut() {
      def.bod.resolve_refs(&def_names)
    }
  }

  pub fn check_unbounds(&mut self) -> Result<(), String> {
    for def in self.0.values() {
      def.bod.check_unbounds()?;
    }
    Ok(())
  }

  pub fn check_has_main(&self) -> Result<(), String> {
    if self.0.contains_key("main") {
      Ok(())
    } else {
      Err("Program doesn't have 'main' definition".to_string())
    }
  }
}

impl Term {
  pub fn resolve_refs(&mut self, def_names: &HashSet<Name>) {
    fn go(term: &mut Term, def_names: &HashSet<Name>, scope: &mut VarScope) {
      match term {
        Term::Lam { nam, bod } => {
          scope.push(nam);
          bod.resolve_refs(def_names);
          scope.pop(nam);
        }
        Term::Var { nam } => {
          if !scope.has_var(nam) && def_names.contains(nam) {
            *term = Term::Ref { nam: nam.clone() }
          }
        }
        Term::App { fun, arg } => {
          fun.resolve_refs(def_names);
          arg.resolve_refs(def_names);
        }
        Term::Ref { .. } => (),
      }
    }
    go(self, def_names, &mut VarScope::default())
  }

  pub fn check_unbounds(&self) -> Result<(), String> {
    fn go(term: &Term, scope: &mut VarScope) -> Result<(), String> {
      match term {
        Term::Lam { nam, bod } => {
          scope.push(nam);
          go(bod, scope)?;
          scope.pop(nam);
        }
        Term::Var { nam } => {
          if !scope.has_var(nam) {
            return Err(format!("Unbound variable '{nam}'"));
          }
        }
        Term::App { fun, arg } => {
          go(fun, scope)?;
          go(arg, scope)?;
        }
        Term::Ref { nam: _ } => (),
      }
      Ok(())
    }
    go(self, &mut VarScope::default())
  }
}

#[derive(Default, Debug)]
struct VarScope(HashMap<Name, usize>);

impl VarScope {
  fn push(&mut self, nam: &Option<Name>) {
    if let Some(nam) = nam {
      *self.0.entry(nam.clone()).or_default() += 1;
    }
  }

  fn pop(&mut self, nam: &Option<Name>) {
    if let Some(nam) = nam {
      *self.0.get_mut(nam).unwrap() -= 1;
    }
  }

  fn has_var(&self, nam: &Name) -> bool {
    if let Some(times_declared) = self.0.get(nam) {
      *times_declared > 0
    } else {
      false
    }
  }
}
