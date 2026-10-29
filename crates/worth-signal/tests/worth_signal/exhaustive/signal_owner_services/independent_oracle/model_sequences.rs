#[path = "model_sequences/adjacency_roster.rs"]
// Model-sequence assembly. Each child owns one semantic test family.
pub(crate) mod adjacency_roster;

#[path = "model_sequences/negative_trace.rs"]
pub(crate) mod negative_trace;

#[path = "model_sequences/public_trace.rs"]
pub(crate) mod public_trace;

#[path = "../../../fixtures/signal_owner_services/independent_oracle/model_sequences/public_trace_terminal.rs"]
pub(crate) mod public_trace_terminal;

#[path = "../../../fixtures/signal_owner_services/independent_oracle/model_sequences/trace_support.rs"]
pub(crate) mod trace_support;
