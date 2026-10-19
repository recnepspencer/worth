//! The installed operation input supplies one external effect's payload exactly.
mod authoring;
mod recipe;
mod validation;
use super::ApplicationOperationBodyRequirementProgram;
use crate::application_schema::ApplicationExternalEffectProtocol;
use crate::portable_identity::WorthQueryPortableTypeIdentity;
pub use recipe::ApplicationOperationBodyInputEffectRecipe;
pub(crate) use validation::validate_emission;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyInputEmission {
    pub effect: String,
    pub input_binding: WorthQueryPortableTypeIdentity,
    pub payload_binding: WorthQueryPortableTypeIdentity,
    pub protocol: ApplicationExternalEffectProtocol,
    pub maximum_payload_bytes: u64,
    pub condition: Option<ApplicationOperationBodyRequirementProgram>,
}
