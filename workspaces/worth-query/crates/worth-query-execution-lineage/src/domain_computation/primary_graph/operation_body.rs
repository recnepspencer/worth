//! Execution of a complete installed body, never a caller-selected replacement.
pub mod assessment;
pub mod children;
pub mod emission;
pub mod expression;
pub mod field_expectation;
pub mod input;
pub mod output;
pub mod path_links;
pub mod path_value;
pub use path_links::PreparedBodyPathLinks;
pub mod reference_input;
pub mod requirement;
pub mod retirement;
pub mod reuse;
pub use children::PreparedBodyChildren;
pub use reuse::CreationDisposition;
pub mod selection;
pub mod source;
pub mod value;
pub use input::{prepare_inputs, PreparedBodyInputs};
pub use path_value::PreparedBodyPathWrites;
pub use retirement::{PreparedBodyRetirements, PreparedRetirementBatch};
pub use selection::{PreparedBodySelection, PreparedBodySelections};
pub use source::PreparedBodySource;

pub use assessment::{
    WorthQueryCurrentBodyRequirementAssessment, WorthQueryOperationBodyAssessmentDenial,
};
pub use field_expectation::BodyFieldExpectation;
pub use requirement::evaluate_requirements;
pub use requirement::{field_value, PreparedRequirementOperand};
pub use requirement::{
    WorthQueryOperationBodyRequirementFinding, WorthQueryOperationBodyRequirementStatus,
    WorthQueryOperationBodyRequirementTarget,
};

use super::WorthQueryInvariantMutationTarget;
use serde::{Deserialize, Serialize};

/// A native body decision retains its exact scope and observed replacement targets.
pub struct WorthQueryOperationBodyDecision<Schema, Scope> {
    pub(crate) target: WorthQueryInvariantMutationTarget<Schema, Scope>,
    pub(crate) inputs: Option<PreparedBodyInputs>,
    pub(crate) sources: Vec<Option<PreparedBodySource>>,
    pub(crate) selections: PreparedBodySelections<Schema>,
    pub(crate) emission: Option<emission::BodyExternalEffectSelection>,
    pub(crate) values: Option<value::PreparedBodyValues>,
    pub(crate) retirements: PreparedBodyRetirements<Schema>,
    pub(crate) path_writes: PreparedBodyPathWrites<Schema>,
    pub(crate) path_links: PreparedBodyPathLinks<Schema>,
    pub(crate) principal_link: Option<WorthQueryInvariantMutationTarget<Schema, ()>>,
    pub(crate) references: reference_input::PreparedBodyReferences<Schema>,
    pub(crate) creation_disposition: reuse::CreationDisposition,
    pub(crate) children: Option<PreparedBodyChildren<Schema>>,
    pub(crate) admitted_at: Option<worth_foundational::facade::CanonicalTimestamp>,
}

/// A declared source expectation, input, requirement or handler rule refused
/// the mutation before any candidate effect.
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
    /// A hand-written handler refused with the application's own bounded
    /// reason; see `HandlerResult::domain_refusal`.
    DomainRefused(super::WorthQueryDomainRefusal),
}

impl From<super::WorthQueryDomainRefusal> for WorthQueryOperationBodyDenial {
    fn from(refusal: super::WorthQueryDomainRefusal) -> Self {
        Self::DomainRefused(refusal)
    }
}
