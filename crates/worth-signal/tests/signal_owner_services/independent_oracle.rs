//! Independent model-sequence certification for the public Signal owner ports.
//!
//! This family is deliberately separate from the production owner modules and
//! from the court-world builders. The model owns only small semantic values;
//! production observations enter through the public facade and are normalized
//! before comparison.

#[path = "independent_oracle/comparison.rs"]
mod comparison;
#[path = "independent_oracle/model_sequences.rs"]
mod model_sequences;
#[path = "independent_oracle/state.rs"]
mod state;
#[path = "independent_oracle/transition.rs"]
mod transition;
