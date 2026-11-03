mod source_loci;
mod state_lowering;
mod struct_lowering;

use serde_json::Value;
use worth_proof::TransitionOutcome;

use super::{JsonCompatibilityLoweringDenial, JsonCompatibilityLoweringOutcome};
use crate::aspects::{
    validate_aspect_value, AspectContract, AspectShape, ContractValidatedAspectArtifact,
    ReferenceAspectType,
};
use crate::compatibility::reference_lowering::lower_json_entity_ref;
use crate::compatibility::scalar_lowering::lower_json_scalar;
use crate::locators::BoundarySourceLocator;

pub use state_lowering::lower_json_record_aspect_state;

use self::source_loci::contract_validation_source;
use self::struct_lowering::lower_json_struct;

pub fn lower_json_aspect_value(
    contract: &AspectContract,
    source: BoundarySourceLocator,
    value: &Value,
) -> JsonCompatibilityLoweringOutcome<ContractValidatedAspectArtifact> {
    let lowered = match contract.shape() {
        AspectShape::Scalar(expected) => {
            lower_json_scalar(&source, value, *expected).map(Into::into)
        }
        AspectShape::Reference(ReferenceAspectType::Entity) => {
            lower_json_entity_ref(&source, value).map(Into::into)
        }
        AspectShape::Struct(shape) => lower_json_struct(&source, value, shape).map(Into::into),
        AspectShape::Opaque(_) => Err(JsonCompatibilityLoweringDenial::JsonShapeNotAdmitted {
            source: source.clone(),
            expected: "opaque value",
        }),
    };

    let Ok(input) = lowered else {
        return TransitionOutcome::denied(lowered.expect_err("checked above"));
    };

    match validate_aspect_value(contract, input) {
        TransitionOutcome::Success(artifact) => TransitionOutcome::success(artifact),
        TransitionOutcome::Denied(denial) => {
            TransitionOutcome::denied(JsonCompatibilityLoweringDenial::ContractValidationDenied {
                source: contract_validation_source(&source, &denial),
                denial,
            })
        }
        TransitionOutcome::Deferred(_)
        | TransitionOutcome::Stale(_)
        | TransitionOutcome::RebindRequired(_)
        | TransitionOutcome::Failed(_) => unreachable!("contract validation uses only denied"),
    }
}
