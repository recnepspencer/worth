use worth_proof::{Artifact, TransitionOutcome};

use super::{
    AuthoritativePatchApplicationDenial, AuthoritativeRecordAspectPatch, FieldLevelAspectPatch,
};
use crate::aspects::state::{
    AuthoritativeRecordAspectState, AuthoritativeRecordAspectStateAdmitted,
    AuthoritativeRecordAspectStateArtifact, CanonicalAspectStateMap,
};
use crate::aspects::structs::StructAspectValue;
use crate::aspects::validation::{
    validate_aspect_value, ContractValidatedAspectValue, ContractValidatedAspectValueView,
    ContractValidationInput,
};

impl AuthoritativeRecordAspectPatch {
    pub fn apply_to_optional(
        &self,
        state: Option<&AuthoritativeRecordAspectState>,
    ) -> TransitionOutcome<
        Option<AuthoritativeRecordAspectStateArtifact>,
        AuthoritativePatchApplicationDenial,
    > {
        let next_state = match state {
            Some(state) => self.apply_to_state(state),
            None => self.apply_to_absent_state(),
        };
        match next_state {
            Ok(state) if state.aspects().is_empty() => TransitionOutcome::success(None),
            Ok(state) => TransitionOutcome::success(Some(Artifact::<
                AuthoritativeRecordAspectStateAdmitted,
                _,
            >::new(state))),
            Err(denial) => TransitionOutcome::denied(denial),
        }
    }

    pub fn apply_to(
        &self,
        state: &AuthoritativeRecordAspectState,
    ) -> TransitionOutcome<
        AuthoritativeRecordAspectStateArtifact,
        AuthoritativePatchApplicationDenial,
    > {
        match self.apply_to_state(state) {
            Ok(next_state) => TransitionOutcome::success(Artifact::<
                AuthoritativeRecordAspectStateAdmitted,
                _,
            >::new(next_state)),
            Err(denial) => TransitionOutcome::denied(denial),
        }
    }

    fn apply_to_state(
        &self,
        state: &AuthoritativeRecordAspectState,
    ) -> Result<AuthoritativeRecordAspectState, AuthoritativePatchApplicationDenial> {
        let mut entries = state.aspects().cloned_entries();

        for cleared_key in self.whole_aspect_clears.keys() {
            entries.remove(cleared_key);
        }

        for (set_key, set_value) in &self.whole_aspect_sets {
            entries.insert(set_key.clone(), set_value.clone());
        }

        for field_patch in self.field_patches.values() {
            let updated_entry = apply_field_patch_to_entry(&entries, field_patch)?;
            entries.insert(field_patch.key().clone(), updated_entry);
        }

        Ok(AuthoritativeRecordAspectState::from_canonical_map(
            CanonicalAspectStateMap::from_canonical_entries(entries),
        ))
    }

    fn apply_to_absent_state(
        &self,
    ) -> Result<AuthoritativeRecordAspectState, AuthoritativePatchApplicationDenial> {
        if let Some(field_patch) = self.field_patches.values().next() {
            return Err(
                AuthoritativePatchApplicationDenial::MissingAspectForFieldPatch(
                    field_patch.key().clone(),
                ),
            );
        }
        Ok(AuthoritativeRecordAspectState::from_canonical_map(
            CanonicalAspectStateMap::from_canonical_entries(self.whole_aspect_sets.clone()),
        ))
    }
}

fn apply_field_patch_to_entry(
    entries: &std::collections::BTreeMap<
        crate::aspects::keys::AspectKey,
        ContractValidatedAspectValue,
    >,
    field_patch: &FieldLevelAspectPatch,
) -> Result<ContractValidatedAspectValue, AuthoritativePatchApplicationDenial> {
    let current_entry = entries.get(field_patch.key()).ok_or_else(|| {
        AuthoritativePatchApplicationDenial::MissingAspectForFieldPatch(field_patch.key().clone())
    })?;

    let ContractValidatedAspectValueView::Struct(current_struct) = current_entry.view() else {
        return Err(
            AuthoritativePatchApplicationDenial::FieldPatchRequiresStructValue(
                field_patch.key().clone(),
            ),
        );
    };

    let mut next_fields = current_struct
        .fields()
        .map(|(field_key, field_value)| (field_key.clone(), field_value.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();

    for field_key in field_patch.field_clears() {
        next_fields.remove(field_key);
    }

    for (field_key, field_value) in field_patch.field_sets() {
        next_fields.insert(field_key.clone(), field_value.clone());
    }

    let next_struct = StructAspectValue::new(next_fields)
        .map_err(AuthoritativePatchApplicationDenial::StructValueConstructionDenied)?;
    let validation = validate_aspect_value(
        field_patch.contract(),
        ContractValidationInput::Struct(next_struct),
    );

    match validation {
        TransitionOutcome::Success(artifact) => {
            let (entry, _proofs, _basis) = artifact.into_parts().into_parts();
            Ok(entry)
        }
        TransitionOutcome::Denied(denial) => {
            Err(AuthoritativePatchApplicationDenial::ContractValidationDenied(denial))
        }
        TransitionOutcome::Deferred(_)
        | TransitionOutcome::Stale(_)
        | TransitionOutcome::RebindRequired(_)
        | TransitionOutcome::Failed(_) => unreachable!("contract validation uses only denied"),
    }
}
