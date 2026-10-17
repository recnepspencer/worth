//! Trusted native live-object broker for confined source-built macro guests.
//! No filesystem/environment implementation is linked into the guest runtime.
#![feature(proc_macro_span)]
extern crate proc_macro;
mod effect_imports;
mod execution;
mod hash_seed;
mod literals;
mod objects;
mod operations;
mod span;
mod structure;

/// An expansion aborts on invalid objects, unsupported effects, or exhausted work.
#[derive(Clone, Debug)]
pub enum Failure {
    InvalidModule(String),
    Protocol(String),
    ForbiddenEffect(String),
    WorkLimit,
    Runtime(String),
}
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidModule(s) | Self::Protocol(s) | Self::Runtime(s) => f.write_str(s),
            Self::ForbiddenEffect(s) => write!(f, "forbidden ambient guest effect: {s}"),
            Self::WorkLimit => f.write_str("macro expansion work limit exceeded"),
        }
    }
}
impl std::error::Error for Failure {}
impl wasmi::errors::HostError for Failure {}
impl From<String> for Failure {
    fn from(reason: String) -> Self {
        Self::Protocol(reason)
    }
}
impl From<&str> for Failure {
    fn from(reason: &str) -> Self {
        Self::Protocol(reason.into())
    }
}

/// Actual values read from the current compiler's live span objects.
#[derive(Debug)]
pub enum SpanRead {
    SourceText(Option<String>),
    Line(usize),
    Column(usize),
    File(String),
    LocalFile(Option<std::path::PathBuf>),
    Debug(String),
}
pub struct Expansion {
    output: proc_macro::TokenStream,
    observations: Vec<SpanRead>,
}
impl Expansion {
    pub fn into_tokens(self) -> proc_macro::TokenStream {
        self.output
    }
    pub fn observations(&self) -> &[SpanRead] {
        &self.observations
    }
}

/// Low-level execution used by the generated-shim qualification fixture.
/// Source provenance and native shim admission are separate, unfinished owners;
/// the ordinary observer still refuses macro units. This engine's byte input
/// cannot construct an implementation edition or native runtime authority.
pub fn expand(
    module: &[u8],
    entry: &str,
    input: proc_macro::TokenStream,
) -> Result<Expansion, Failure> {
    execution::expand(module, entry, input)
}
