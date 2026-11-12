use serde::{Deserialize, Serialize};
use worth_proof::TransitionOutcome;

use super::{
    contract_for_readmission, PortableAspectContractLookup, PortableAspectFieldSet,
    PortableAspectPatchOperation, PortableAspectReadmissionDenial, PortableRecordAspectPatch,
};
use crate::aspects::{
    validate_aspect_value, AbsenceLaw, AspectContract, AspectMask, AuthoritativeRecordAspectPatch,
    CanonicalFieldPath, FieldLevelAspectPatch, MutationMask,
};

mod storage_bounds;
#[cfg(test)]
mod tests;

pub(super) struct OperationKeyOrdinal<'a> {
    key: &'a crate::aspects::AspectKey,
    ordinal: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortablePatchReadmissionPurpose {
    RecordCreation,
    RecordMutation,
    RecordDeletion,
}

pub fn readmit_portable_record_aspect_patch(
    candidate: PortableRecordAspectPatch,
    purpose: PortablePatchReadmissionPurpose,
    contracts: &impl PortableAspectContractLookup,
) -> TransitionOutcome<AuthoritativeRecordAspectPatch, PortableAspectReadmissionDenial> {
    match readmit_patch(candidate, purpose, contracts) {
        Ok(patch) => TransitionOutcome::success(patch),
        Err(denial) => TransitionOutcome::denied(denial),
    }
}

fn readmit_patch(
    candidate: PortableRecordAspectPatch,
    purpose: PortablePatchReadmissionPurpose,
    contracts: &impl PortableAspectContractLookup,
) -> Result<AuthoritativeRecordAspectPatch, PortableAspectReadmissionDenial> {
    let duplicate = first_duplicate(candidate.operations());
    let mut admitted = Vec::with_capacity(candidate.operations().len());
    for (ordinal, operation) in candidate.into_operations().into_iter().enumerate() {
        if Some(ordinal) == duplicate {
            return Err(PortableAspectReadmissionDenial::DuplicateAspectOperation(
                operation.basis().key().clone(),
            ));
        }
        let contract = contract_for_readmission(operation.basis(), contracts)?;
        let next = admit_operation(operation, &contract, purpose)?;
        admitted.push(next);
    }
    Ok(AuthoritativeRecordAspectPatch::from_disjoint_patches(
        admitted,
    ))
}

fn first_duplicate(operations: &[PortableAspectPatchOperation]) -> Option<usize> {
    let mut keys = Vec::with_capacity(operations.len());
    keys.extend(
        operations
            .iter()
            .enumerate()
            .map(|(ordinal, operation)| OperationKeyOrdinal {
                key: operation.basis().key(),
                ordinal,
            }),
    );
    keys.sort_unstable_by(|left, right| {
        left.key
            .cmp(right.key)
            .then(left.ordinal.cmp(&right.ordinal))
    });
    keys.windows(2)
        .filter(|pair| pair[0].key == pair[1].key)
        .map(|pair| pair[1].ordinal)
        .min()
}

fn admit_operation(
    operation: PortableAspectPatchOperation,
    contract: &AspectContract,
    purpose: PortablePatchReadmissionPurpose,
) -> Result<AuthoritativeRecordAspectPatch, PortableAspectReadmissionDenial> {
    match operation {
        PortableAspectPatchOperation::SetWhole { value, .. } => {
            let validated = transition_value(validate_aspect_value(contract, value), contract)?;
            let value = validated.into_parts().into_parts().0;
            Ok(AuthoritativeRecordAspectPatch {
                whole_aspect_sets: Box::new([value]),
                ..AuthoritativeRecordAspectPatch::empty()
            })
        }
        PortableAspectPatchOperation::ClearWhole { .. } => {
            admit_whole_clear(contract, purpose)?;
            Ok(AuthoritativeRecordAspectPatch {
                whole_aspect_clears: Box::new([contract.clone()]),
                ..AuthoritativeRecordAspectPatch::empty()
            })
        }
        PortableAspectPatchOperation::PatchFields {
            selected_fields,
            field_sets,
            field_clears,
            ..
        } => {
            if purpose == PortablePatchReadmissionPurpose::RecordCreation {
                return Err(
                    PortableAspectReadmissionDenial::FieldPatchDeniedForCreation(
                        contract.key().clone(),
                    ),
                );
            }
            let mask = AspectMask::<MutationMask>::new(
                selected_fields.into_iter().map(CanonicalFieldPath::single),
            );
            let mut sets = Vec::with_capacity(field_sets.len());
            sets.extend(
                field_sets
                    .into_iter()
                    .map(PortableAspectFieldSet::into_parts),
            );
            let patch =
                FieldLevelAspectPatch::from_owned_fields(contract, &mask, sets, field_clears)
                    .map_err(PortableAspectReadmissionDenial::PatchConstruction)?;
            Ok(AuthoritativeRecordAspectPatch {
                field_patches: Box::new([patch]),
                ..AuthoritativeRecordAspectPatch::empty()
            })
        }
    }
}

fn admit_whole_clear(
    contract: &AspectContract,
    purpose: PortablePatchReadmissionPurpose,
) -> Result<(), PortableAspectReadmissionDenial> {
    let admitted = match purpose {
        PortablePatchReadmissionPurpose::RecordCreation => false,
        PortablePatchReadmissionPurpose::RecordMutation => {
            contract.absence() != AbsenceLaw::Required
        }
        PortablePatchReadmissionPurpose::RecordDeletion => true,
    };
    if admitted {
        Ok(())
    } else {
        Err(PortableAspectReadmissionDenial::WholeClearDenied {
            key: contract.key().clone(),
            purpose,
            absence: contract.absence(),
        })
    }
}

fn transition_value(
    outcome: worth_proof::TransitionOutcome<
        crate::aspects::ContractValidatedAspectArtifact,
        crate::aspects::ContractValidationDenial,
    >,
    contract: &AspectContract,
) -> Result<crate::aspects::ContractValidatedAspectArtifact, PortableAspectReadmissionDenial> {
    match outcome {
        TransitionOutcome::Success(value) => Ok(value),
        TransitionOutcome::Denied(denial) => {
            Err(PortableAspectReadmissionDenial::ValueValidation {
                key: contract.key().clone(),
                denial,
            })
        }
    }
}
