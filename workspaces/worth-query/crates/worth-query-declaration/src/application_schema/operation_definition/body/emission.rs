//! One external slot, with an explicit native payload source and optional guard.
mod authoring;
mod evaluation_authoring;
mod recipe;
mod validation;
use super::ApplicationOperationBodyRequirementProgram;
use crate::application_schema::ApplicationExternalEffectProtocol;
use crate::portable_identity::WorthQueryPortableTypeIdentity;
pub use recipe::ApplicationOperationBodyExternalEffectRecipe;
pub(crate) use validation::validate_emission;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyExternalEmission {
    pub source: ApplicationOperationBodyExternalPayloadSource,
    pub effect: String,
    pub input_binding: WorthQueryPortableTypeIdentity,
    pub payload_binding: WorthQueryPortableTypeIdentity,
    pub protocol: ApplicationExternalEffectProtocol,
    pub maximum_payload_bytes: u64,
    pub condition: Option<ApplicationOperationBodyRequirementProgram>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationOperationBodyExternalPayloadSource {
    OperationInput,
    QueryContext {
        mutation_binding: String,
        query: Box<crate::application_query::ErasedApplicationQueryDefinition>,
    },
    Evaluation {
        mutation_binding: String,
        query: Box<crate::application_query::ErasedApplicationQueryDefinition>,
    },
}
