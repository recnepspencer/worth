use worth_execution::ExecutionRequest;
pub(crate) use worth_relational_mutation::internal_port::mvcc::validation::proposal_touches::{
    collection, inverse_projection, preparation, project_applied_intents, work_observation,
};
pub use worth_relational_mutation::internal_port::mvcc::validation::proposal_touches::{
    ValidatedEntityStructTouch, ValidatedMutationTouch, ValidatedMutationTouchProjectionError,
    ValidatedMutationTouchProjectionWork, ValidatedMutationTouches,
};
#[path = "proposal_touches/authored_restoration.rs"]
mod authored_restoration;
#[path = "proposal_touches/proposal_projection.rs"]
mod proposal_projection;
