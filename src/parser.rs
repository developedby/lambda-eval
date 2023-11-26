use core::fmt;
use std::{collections::HashMap, iter::Map, ops::Range};

use chumsky::{
  input::{SpannedInput, Stream, ValueInput},
  prelude::*,
};
use logos::{FilterResult, Lexer, Logos, SpannedIter};

use crate::ast::{Book, Definition, Name, Term};

pub fn parse_book(code: &str) -> Result<Book, Vec<Rich<Token>>> {
  book().parse(token_stream(code)).into_result()
}

pub fn parse_term(code: &str) -> Result<Term, Vec<Rich<Token>>> {
  term().parse(token_stream(code)).into_result()
}

fn book<'a, I>() -> impl Parser<'a, I, Book, extra::Err<Rich<'a, Token>>>
where
  I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
  definition()
    .repeated()
    .collect::<Vec<_>>()
    .map(|defs| Book(HashMap::from_iter(defs.into_iter().map(|def| (def.nam.clone(), def)))))
}

fn definition<'a, I>() -> impl Parser<'a, I, Definition, extra::Err<Rich<'a, Token>>>
where
  I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
  name().then_ignore(just(Token::Equals)).then(term()).map(|(nam, bod)| Definition { nam, bod })
}

fn term<'a, I>() -> impl Parser<'a, I, Term, extra::Err<Rich<'a, Token>>>
where
  I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
  let var = name().map(|name| Term::Var { nam: name }).boxed();
  recursive(|term| {
    // λx body
    let lam = just(Token::Lambda)
      .ignore_then(name_or_era())
      .then(term.clone())
      .map(|(name, body)| Term::Lam { nam: name, bod: Box::new(body) })
      .boxed();

    // (f arg1 arg2 ...)
    let app = term
      .clone()
      .foldl(term.clone().repeated(), |fun, arg| Term::App { fun: Box::new(fun), arg: Box::new(arg) })
      .delimited_by(just(Token::LParen), just(Token::RParen))
      .boxed();

    choice((var, lam, app))
  })
}

fn name<'a, I>() -> impl Parser<'a, I, Name, extra::Err<Rich<'a, Token>>>
where
  I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
  select!(Token::Name(name) => name)
}

fn name_or_era<'a, I>() -> impl Parser<'a, I, Option<Name>, extra::Err<Rich<'a, Token>>>
where
  I: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
  choice((select!(Token::Asterisk => None), name().map(Some)))
}

fn token_stream(
  code: &str,
) -> SpannedInput<
  Token,
  SimpleSpan,
  Stream<
    Map<SpannedIter<Token>, impl FnMut((Result<Token, LexingError>, Range<usize>)) -> (Token, SimpleSpan)>,
  >,
> {
  let token_iter = Token::lexer(code).spanned().map(|(token, span)| match token {
    Ok(t) => (t, SimpleSpan::from(span)),
    Err(e) => (Token::Error(e), SimpleSpan::from(span)),
  });
  Stream::from_iter(token_iter).spanned(SimpleSpan::from(code.len() .. code.len()))
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(error=LexingError)]
pub enum Token {
  #[regex("[_.a-zA-Z][_.a-zA-Z0-9]*", |lex| lex.slice().parse().ok())]
  Name(String),

  #[regex("@|λ")]
  Lambda,

  #[token("*")]
  Asterisk,

  #[token("=")]
  Equals,

  #[token("(")]
  LParen,

  #[token(")")]
  RParen,

  #[regex("//.*", logos::skip)]
  SingleLineComment,

  #[token("/*", comment)]
  MultiLineComment,

  #[regex(r"[ \t\f\r\n]+", logos::skip)]
  Whitespace,

  Error(LexingError),
}

#[derive(Default, Debug, PartialEq, Clone)]
pub enum LexingError {
  UnclosedComment,

  #[default]
  InvalidCharacter,
}

// Lexer for nested multi-line comments
#[derive(Logos)]
pub enum MultiLineComment {
  #[token("/*")]
  Open,

  #[token("*/")]
  Close,

  #[regex("(?s).")]
  Other,
}

fn comment(lexer: &mut Lexer<'_, Token>) -> FilterResult<(), LexingError> {
  let start = lexer.remainder();
  let mut comment = MultiLineComment::lexer(start);
  let mut depth = 1; // Already matched an Open token, so count it
  loop {
    if let Some(token) = comment.next() {
      match token {
        Ok(MultiLineComment::Open) => depth += 1,
        Ok(MultiLineComment::Close) => depth -= 1,
        Ok(MultiLineComment::Other) => {}
        Err(()) => unreachable!(),
      }
    } else {
      // Unclosed comment
      return FilterResult::Error(LexingError::UnclosedComment);
    }
    if depth <= 0 {
      break;
    }
  }
  let end = comment.remainder();
  let span = (end as *const str as *const () as usize) - (start as *const str as *const () as usize);
  lexer.bump(span);
  FilterResult::Skip
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::Name(s) => write!(f, "{}", s),
      Self::Lambda => write!(f, "λ"),
      Self::Asterisk => write!(f, "*"),
      Self::Equals => write!(f, "="),
      Self::LParen => write!(f, "("),
      Self::RParen => write!(f, ")"),
      Self::SingleLineComment => write!(f, "<SingleLineComment>"),
      Self::MultiLineComment => write!(f, "<MultiLineComment>"),
      Self::Whitespace => write!(f, "<Whitespace>"),
      Self::Error(LexingError::InvalidCharacter) => write!(f, "<InvalidCharacter>"),
      Self::Error(LexingError::UnclosedComment) => write!(f, "<UnclosedComment>"),
    }
  }
}
