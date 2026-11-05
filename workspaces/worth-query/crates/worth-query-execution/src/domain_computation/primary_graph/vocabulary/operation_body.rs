//! `operation_body`'s vocabulary, moved whole from `primary_graph::operation_body`: its root file's items here, one child per other source file.

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
pub(crate) mod assessment;
pub(crate) mod children;
pub(crate) mod field_expectation;
pub(crate) mod input;
pub(crate) mod operation_body_value;
pub(crate) mod path_links;
pub(crate) mod path_value;
pub(crate) mod reference_input;
pub(crate) mod requirement;
pub(crate) mod retirement;
pub(crate) mod reuse;
pub(crate) mod selection;
pub(crate) mod source;
pub(crate) mod value;
use serde::Deserialize;
use serde::Serialize;
