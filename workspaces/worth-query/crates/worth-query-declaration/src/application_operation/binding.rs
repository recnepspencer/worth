mod descriptor;
mod intent;
mod output;
mod portable_description;
mod principal_contract;
mod scope;
mod scope_contract;
mod scope_resolution;

pub use descriptor::{
    ApplicationMutationBindingDescriptor, ApplicationMutationHandlerMetadata,
    ApplicationMutationIdempotencyMetadata,
};
pub use intent::{ApplicationMutationBinding, ApplicationMutationIntent};
pub use output::{
    ApplicationMutationOutputContract, ApplicationMutationOutputPosture,
    ApplicationMutationOutputRoleDescriptor, NoApplicationMutationOutputs,
};
pub use portable_description::{
    ApplicationMutationDescription, ApplicationMutationDescriptionParts,
    ApplicationMutationOutputRoleDescription, ApplicationMutationScopeDescription,
};
pub use principal_contract::ApplicationMutationPrincipalBindingContract;
pub use scope::{
    ApplicationMutationFieldScope, ApplicationMutationPrincipalScope,
    ApplicationMutationScopeBinding,
};
pub use scope_contract::{
    ApplicationMutationScopeContract, ApplicationMutationScopeResolutionMode,
};
pub use scope_resolution::ApplicationMutationScopeResolution;

#[cfg(test)]
mod tests;
