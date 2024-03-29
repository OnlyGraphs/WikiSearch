#[macro_use]
extern crate lazy_static;

pub mod ast;
pub mod errors;
pub mod parser;

#[cfg(test)]
pub mod parser_tests;
pub use {crate::parser::*, ast::*, errors::*};
