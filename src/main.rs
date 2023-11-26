use std::path::{Path, PathBuf};

use ast::{Book, Term};
use clap::{Parser, ValueEnum};
use itertools::Itertools;
use parser::parse_book;

mod ast;
mod parser;
mod passes;
mod reduce;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(value_enum)]
  pub mode: Mode,

  #[arg(short, long, default_value="nf")]
  pub form: Form,

  #[arg(short, long, default_value="normal")]
  pub order: ReductionOrder,

  #[arg(help = "Path to the input file")]
  pub path: PathBuf,
}

#[derive(ValueEnum, Clone, Debug)]
enum Mode {
  Run,
  RunStepped,
  Interactive,
}

#[derive(ValueEnum, Clone, Debug)]
enum Form {
  Nf,
  Wnf,
  Hnf,
  Whnf,
}

#[derive(ValueEnum, Clone, Debug)]
enum ReductionOrder {
  Normal,
  Applicative,
}

/// Reads a file and parses to a definition book.
pub fn load_book(path: &Path) -> Result<Book, String> {
  let code = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
  match parse_book(&code) {
    Ok(book) => Ok(book),
    Err(errs) => {
      let msg = errs.into_iter().map(|e| e.to_string()).join("\n");
      Err(msg)
    }
  }
}

fn stop_fn(form: &Form) -> impl Fn(&Term) -> bool {
  match form {
    Form::Nf => Term::stop_nf,
    Form::Wnf => todo!(),
    Form::Hnf => todo!(),
    Form::Whnf => Term::stop_whnf,
  }
}

fn step_fn(order: &ReductionOrder) -> impl Fn(&mut Term, &Book) -> bool {
  match order {
    ReductionOrder::Normal => Term::normal_order_step,
    ReductionOrder::Applicative => todo!(),
  }
}

fn main() -> Result<(), String> {
  let args = Args::parse();

  let mut book = load_book(&args.path)?;
  book.check_has_main()?;
  book.resolve_refs();
  book.check_unbounds()?;

  let mut main = book.0["main"].clone().bod;
  let stop = stop_fn(&args.form);
  let step = step_fn(&args.order);

  match args.mode {
    Mode::Run => {
      main.reduce(&book, stop, step);
      println!("{main}");
    }
    Mode::RunStepped => {
      let steps = main.reduce_stepped(&book, stop, step);
      for step in steps {
        println!("{step}");
      }
    }
    Mode::Interactive => todo!(),
  }

  Ok(())
}
