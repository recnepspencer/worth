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
use crate::domain_computation::primary_graph::vocabulary::operation_body::path_links::PreparedBodyPathLinks;
pub(in crate::domain_computation::primary_graph) mod reference_input;
mod requirement;
mod retirement;
mod reuse;
use crate::domain_computation::primary_graph::vocabulary::operation_body::children::PreparedBodyChildren;
use crate::domain_computation::primary_graph::vocabulary::operation_body::reuse::CreationDisposition;
mod selection;
mod source;
pub(in crate::domain_computation::primary_graph) mod value;
pub(in crate::domain_computation::primary_graph) use input::prepare_inputs;
use crate::domain_computation::primary_graph::vocabulary::operation_body::input::PreparedBodyInputs;
use crate::domain_computation::primary_graph::vocabulary::operation_body::path_value::PreparedBodyPathWrites;
use crate::domain_computation::primary_graph::vocabulary::operation_body::retirement::PreparedBodyRetirements;
use crate::domain_computation::primary_graph::vocabulary::operation_body::selection::PreparedBodySelections;
use crate::domain_computation::primary_graph::vocabulary::operation_body::source::PreparedBodySource;

pub use assessment::WorthQueryOperationBodyAssessmentDenial;

pub(in crate::domain_computation::primary_graph) use handler::OperationBodyHandler;
pub(in crate::domain_computation::primary_graph) use requirement::evaluate_requirements;
pub(in crate::domain_computation::primary_graph) use requirement::field_value;


use crate::domain_computation::primary_graph::vocabulary::invariant_projection::WorthQueryInvariantMutationTarget;
use serde::{Deserialize, Serialize};

/// A native body decision retains its exact scope and observed replacement targets.
pub struct WorthQueryOperationBodyDecision<Schema, Scope> {
    target: WorthQueryInvariantMutationTarget<Schema, Scope>,
    inputs: Option<PreparedBodyInputs>,
    sources: Vec<Option<PreparedBodySource>>,
    selections: PreparedBodySelections<Schema>,
    emission: Option<emission::BodyExternalEffectSelection>,
    values: Option<crate::domain_computation::primary_graph::vocabulary::operation_body::operation_body_value::PreparedBodyValues>,
    retirements: PreparedBodyRetirements<Schema>,
    path_writes: PreparedBodyPathWrites<Schema>,
    path_links: PreparedBodyPathLinks<Schema>,
    principal_link: Option<WorthQueryInvariantMutationTarget<Schema, ()>>,
    references: crate::domain_computation::primary_graph::vocabulary::operation_body::reference_input::PreparedBodyReferences<Schema>,
    creation_disposition: crate::domain_computation::primary_graph::vocabulary::operation_body::reuse::CreationDisposition,
    children: Option<PreparedBodyChildren<Schema>>,
}

pub use assessment::AssessmentOps;
pub(crate) mod host;
