#[path = "../../fixtures/signal_owner_services/independent_oracle/comparison.rs"]
// Independent model-sequence certification for the public Signal owner ports.
//
// This family is deliberately separate from the production owner modules and
// from the court-world builders. The model owns only small semantic values;
// production observations enter through the public facade and are normalized
// before comparison.
pub(crate) mod comparison;

#[path = "independent_oracle/model_sequences.rs"]
pub(crate) mod model_sequences;

#[path = "../../fixtures/signal_owner_services/independent_oracle/state.rs"]
pub(crate) mod state;

#[path = "../../fixtures/signal_owner_services/independent_oracle/transition.rs"]
pub(crate) mod transition;
