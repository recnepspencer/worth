//! Execution of a complete installed body, never a caller-selected replacement.
mod assessment;
mod children;
pub(in crate::domain_computation::primary_graph) mod emission;
mod expression;
mod field_expectation;
mod handler;
mod input;
mod input_expectation;
mod installation;
mod output;
mod path_links;
mod path_value;
pub(in crate::domain_computation::primary_graph) use path_links::PreparedBodyPathLinks;
pub(in crate::domain_computation::primary_graph) mod reference_input;
mod requirement;
mod retirement;
mod reuse;
pub(in crate::domain_computation::primary_graph) use children::PreparedBodyChildren;
pub(in crate::domain_computation::primary_graph) use reuse::CreationDisposition;
mod selection;
mod source;
pub(in crate::domain_computation::primary_graph) mod value;
pub(in crate::domain_computation::primary_graph) use input::{prepare_inputs, PreparedBodyInputs};
pub(in crate::domain_computation::primary_graph) use path_value::PreparedBodyPathWrites;
pub(in crate::domain_computation::primary_graph) use retirement::{
    PreparedBodyRetirements, PreparedRetirementBatch,
};
pub(in crate::domain_computation::primary_graph) use selection::{
    PreparedBodySelection, PreparedBodySelections,
};
pub(in crate::domain_computation::primary_graph) use source::PreparedBodySource;

pub use assessment::{
    WorthQueryCurrentBodyRequirementAssessment, WorthQueryOperationBodyAssessmentDenial,
};
pub(in crate::domain_computation::primary_graph) use field_expectation::BodyFieldExpectation;
pub(in crate::domain_computation::primary_graph) use handler::OperationBodyHandler;
pub(in crate::domain_computation::primary_graph) use requirement::evaluate_requirements;
pub(in crate::domain_computation::primary_graph) use requirement::{
    field_value, PreparedRequirementOperand,
};
pub use requirement::{
    WorthQueryOperationBodyRequirementFinding, WorthQueryOperationBodyRequirementStatus,
    WorthQueryOperationBodyRequirementTarget,
};

use super::WorthQueryInvariantMutationTarget;
use serde::{Deserialize, Serialize};

/// A native body decision retains its exact scope and observed replacement targets.
pub struct WorthQueryOperationBodyDecision<Schema, Scope> {
    target: WorthQueryInvariantMutationTarget<Schema, Scope>,
    inputs: Option<PreparedBodyInputs>,
    sources: Vec<Option<PreparedBodySource>>,
    selections: PreparedBodySelections<Schema>,
    emission: Option<emission::BodyExternalEffectSelection>,
    values: Option<value::PreparedBodyValues>,
    retirements: PreparedBodyRetirements<Schema>,
    path_writes: PreparedBodyPathWrites<Schema>,
    path_links: PreparedBodyPathLinks<Schema>,
    references: reference_input::PreparedBodyReferences<Schema>,
    creation_disposition: reuse::CreationDisposition,
    children: Option<PreparedBodyChildren<Schema>>,
}

/// A declared source expectation failed before any candidate effect.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum WorthQueryOperationBodyDenial {
    SourceMismatch {
        entity: String,
        aspect: String,
        field: String,
    },
    InputMismatch {
        member: String,
    },
    RequirementUnmet {
        entity: String,
        aspect: String,
        field: String,
    },
    DeclaredRequirementUnmet {
        identity: String,
    },
}
