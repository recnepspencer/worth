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
            whole_aspect_sets: BTreeMap::new(),
            whole_aspect_clears: BTreeMap::new(),
            field_patches: BTreeMap::new(),
        }
    }

    pub fn whole_aspect(
        sets: impl IntoIterator<Item = ContractValidatedAspectArtifact>,
        clears: impl IntoIterator<Item = AspectContract>,
    ) -> TransitionOutcome<Self, AuthoritativePatchConstructionDenial> {
        let mut patch = Self::empty();
        for contract in clears {
            let key = contract.key().clone();
            if patch
                .whole_aspect_clears
                .insert(key.clone(), contract)
                .is_some()
            {
                return TransitionOutcome::denied(
                    AuthoritativePatchConstructionDenial::DuplicateWholeAspectClear(key),
                );
            }
        }

        for artifact in sets {
            let (entry, _proofs, _basis) = artifact.into_parts().into_parts();
            if patch.whole_aspect_sets.contains_key(entry.key()) {
                return TransitionOutcome::denied(
                    AuthoritativePatchConstructionDenial::DuplicateWholeAspectSet(
                        entry.key().clone(),
                    ),
                );
            }
            if let Some(clear_contract) = patch.whole_aspect_clears.get(entry.key())
                && (clear_contract.identity() != entry.contract_identity()
                    || clear_contract.revision() != entry.contract_revision())
            {
                return TransitionOutcome::denied(
                    AuthoritativePatchConstructionDenial::OverlappingWholeAspectContractMismatch(
                        entry.key().clone(),
                    ),
                );
            }
            patch.whole_aspect_clears.remove(entry.key());
            patch.whole_aspect_sets.insert(entry.key().clone(), entry);
        }

        TransitionOutcome::success(patch)
    }
}
