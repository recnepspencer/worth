use worth_foundational::{
    admit_authoritative_record_aspect_state, prepare_aspect_contract_for_digest,
    prepare_aspect_mask_for_digest, prepare_aspect_patch_for_digest,
    prepare_aspect_state_for_digest, AbsenceLaw, AspectContract, AspectMask, FieldDeclaration,
    FieldRequirement, ScalarAspectType, StructAspectShape,
};
use worth_proof::TransitionOutcome;

use crate::certification::foundational_vocabulary::{field, identity, key, revision};

pub(super) fn ready_contract(
    contract: AspectContract,
) -> worth_foundational::DigestPreparationReadyAspectContractArtifact {
    let ready = match prepare_aspect_contract_for_digest(contract) {
        TransitionOutcome::Success(ready) => ready,
        _ => unreachable!("digest preparation uses only success"),
    };
    ready
}

pub(super) fn ready_mask<Mode>(
    aspect_key: worth_foundational::AspectKey,
    mask: AspectMask<Mode>,
) -> worth_foundational::DigestPreparationReadyAspectMaskArtifact<Mode>
where
    Mode: worth_foundational::DigestPreparationMaskMode,
{
    let ready = match prepare_aspect_mask_for_digest(aspect_key, mask) {
        TransitionOutcome::Success(ready) => ready,
        _ => unreachable!("digest preparation uses only success"),
    };
    ready
}

pub(super) fn ready_patch(
    patch: &worth_foundational::AuthoritativeRecordAspectPatch,
) -> worth_foundational::DigestPreparationReadyAspectPatchArtifact {
    let ready = match prepare_aspect_patch_for_digest(patch) {
        TransitionOutcome::Success(ready) => ready,
        _ => unreachable!("digest preparation uses only success"),
    };
    ready
}

pub(super) fn admitted_state<const N: usize>(
    entries: [worth_foundational::ContractValidatedAspectArtifact; N],
) -> worth_foundational::AuthoritativeRecordAspectStateArtifact {
    let TransitionOutcome::Success(state) = admit_authoritative_record_aspect_state(entries) else {
        panic!("expected authoritative state admission");
    };
    state
}

pub(super) fn ready_state(
    state: worth_foundational::AuthoritativeRecordAspectStateArtifact,
) -> worth_foundational::DigestPreparationReadyAspectStateArtifact {
    let ready = match prepare_aspect_state_for_digest(state) {
        TransitionOutcome::Success(ready) => ready,
        _ => unreachable!("digest preparation uses only success"),
    };
    ready
}

pub(super) fn task_summary_contract() -> AspectContract {
    AspectContract::struct_aspect(
        key("task.summary"),
        identity(20),
        revision(1),
        task_summary_shape([
            ("title", ScalarAspectType::String),
            ("done", ScalarAspectType::Bool),
        ]),
    )
}

pub(super) fn task_summary_contract_with_reversed_declaration_order() -> AspectContract {
    AspectContract::struct_aspect(
        key("task.summary"),
        identity(20),
        revision(1),
        task_summary_shape([
            ("done", ScalarAspectType::Bool),
            ("title", ScalarAspectType::String),
        ]),
    )
}

fn task_summary_shape<const N: usize>(fields: [(&str, ScalarAspectType); N]) -> StructAspectShape {
    StructAspectShape::new(fields.map(|(name, value_type)| {
        FieldDeclaration::new(
            field(name),
            value_type,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law")
    }))
    .expect("unique fields")
}
