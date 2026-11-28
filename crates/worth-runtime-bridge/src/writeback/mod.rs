mod aspect_intent;
mod contracts;
mod declaration;
mod effect;
mod idempotence;
mod loop_prevention;
mod outcome;
mod validation;

pub use aspect_intent::{BridgeWritebackEffectIntent, BridgeWritebackEffectIntentError};
pub use contracts::{
    AdmittedBridgeWritebackContract, BridgeWritebackAuthorityInputs,
    BridgeWritebackContractIdentity,
};
pub use declaration::{
    BridgeWritebackDeclaration, BridgeWritebackDeclarationIdentity, BridgeWritebackEffectClass,
    BridgeWritebackFamilyKind, BridgeWritebackRequestMode, BridgeWritebackStrategyClass,
    BridgeWritebackStrategyDescriptorBasis,
};
pub use effect::{
    BridgeDerivedWritebackEffect, BridgeWritebackCausalityBasis, BridgeWritebackCausalityIdentity,
    BridgeWritebackEffectIdentity, BridgeWritebackFeedbackContext,
    BridgeWritebackFeedbackProvenance, BridgeWritebackNativeCausalityInputs,
};
pub use idempotence::{
    BridgeWritebackAuthoritativeStateBasis, BridgeWritebackIdempotenceBasis,
    BridgeWritebackIdempotenceClass, BridgeWritebackIdempotenceIdentity,
};
pub use loop_prevention::{
    BridgeWritebackLoopDisposition, BridgeWritebackLoopPreventionIdentity,
    BridgeWritebackLoopPreventionReport,
};
pub use outcome::{
    BridgeWritebackAuthorityOutcome, BridgeWritebackFailureClass, BridgeWritebackOutcomeClass,
};
pub use validation::ValidatedBridgeWritebackDeclaration;

pub(crate) use validation::validate_authority_inputs;
