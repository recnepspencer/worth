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
pub use retirement::{
    PreparedBodyRetirements, PreparedRetirementBatch,
};
pub use selection::{
    PreparedBodySelection, PreparedBodySelections,
};
pub use source::PreparedBodySource;

pub use assessment::{
    WorthQueryCurrentBodyRequirementAssessment, WorthQueryOperationBodyAssessmentDenial,
};
pub use field_expectation::BodyFieldExpectation;
pub use requirement::evaluate_requirements;
pub use requirement::{
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
    pub target: WorthQueryInvariantMutationTarget<Schema, Scope>,
    pub inputs: Option<PreparedBodyInputs>,
    pub sources: Vec<Option<PreparedBodySource>>,
    pub selections: PreparedBodySelections<Schema>,
    pub emission: Option<emission::BodyExternalEffectSelection>,
    pub values: Option<value::PreparedBodyValues>,
    pub retirements: PreparedBodyRetirements<Schema>,
    pub path_writes: PreparedBodyPathWrites<Schema>,
    pub path_links: PreparedBodyPathLinks<Schema>,
    pub principal_link: Option<WorthQueryInvariantMutationTarget<Schema, ()>>,
    pub references: reference_input::PreparedBodyReferences<Schema>,
    pub creation_disposition: reuse::CreationDisposition,
    pub children: Option<PreparedBodyChildren<Schema>>,
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
