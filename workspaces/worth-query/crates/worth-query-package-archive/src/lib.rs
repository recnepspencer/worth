//! Store-neutral, authority-free archive protocol for portable Query packages.

#![forbid(unsafe_code)]

mod application_program;
mod binary_encoding;
mod binary_input;
mod binary_output;
mod compatibility;
mod decoding;
mod denial;
mod encoding;
mod envelope;
mod limits;
mod manifest;
mod protocol;
mod record;
mod repository;

pub mod facade;
