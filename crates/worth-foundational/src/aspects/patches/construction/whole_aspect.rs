use std::collections::BTreeMap;

use worth_proof::TransitionOutcome;

use crate::aspects::contracts::AspectContract;
use crate::aspects::patches::{
    AuthoritativePatchConstructionDenial, AuthoritativeRecordAspectPatch,
};
use crate::aspects::validation::ContractValidatedAspectArtifact;

impl AuthoritativeRecordAspectPatch {
    pub fn empty() -> Self {
        Self {
            whole_aspect_sets: Box::default(),
            whole_aspect_clears: Box::default(),
            field_patches: Box::default(),
        }
    }

    pub fn whole_aspect(
        sets: impl IntoIterator<Item = ContractValidatedAspectArtifact>,
        clears: impl IntoIterator<Item = AspectContract>,
    ) -> TransitionOutcome<Self, AuthoritativePatchConstructionDenial> {
        // Generic lazy iterators retain their original early-refusal mechanics.
        // The owned portable readmission path does not invoke these trees.
        let mut whole_aspect_clears = BTreeMap::new();
        let mut whole_aspect_sets = BTreeMap::new();
        for contract in clears {
            let key = contract.key().clone();
            if whole_aspect_clears.insert(key.clone(), contract).is_some() {
                return TransitionOutcome::denied(
                    AuthoritativePatchConstructionDenial::DuplicateWholeAspectClear(key),
                );
            }
        }

        for artifact in sets {
            let (entry, _proofs, _basis) = artifact.into_parts().into_parts();
            if whole_aspect_sets.contains_key(entry.key()) {
                return TransitionOutcome::denied(
                    AuthoritativePatchConstructionDenial::DuplicateWholeAspectSet(
                        entry.key().clone(),
                    ),
                );
            }
            if let Some(clear_contract) = whole_aspect_clears.get(entry.key())
                && (clear_contract.identity() != entry.contract_identity()
                    || clear_contract.revision() != entry.contract_revision())
            {
                return TransitionOutcome::denied(
                    AuthoritativePatchConstructionDenial::OverlappingWholeAspectContractMismatch(
                        entry.key().clone(),
                    ),
                );
            }
            whole_aspect_clears.remove(entry.key());
            whole_aspect_sets.insert(entry.key().clone(), entry);
        }

        TransitionOutcome::success(Self {
            whole_aspect_sets: whole_aspect_sets
                .into_values()
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            whole_aspect_clears: whole_aspect_clears
                .into_values()
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            field_patches: Box::default(),
        })
    }
}
