//! One canonical merge applies clears, sets and field updates without a map clone.
use super::{
    AuthoritativePatchApplicationDenial as Denial, AuthoritativeRecordAspectPatch,
    FieldLevelAspectPatch,
};
use crate::aspects::state::{
    AuthoritativeRecordAspectState as State, AuthoritativeRecordAspectStateAdmitted,
    AuthoritativeRecordAspectStateArtifact, CanonicalAspectStateMap,
};
use crate::aspects::validation::{
    validate_aspect_value, ContractValidatedAspectValue, ContractValidatedAspectValueView,
    ContractValidationInput,
};
use worth_proof::{Artifact, TransitionOutcome};

impl AuthoritativeRecordAspectPatch {
    pub fn apply_to_optional(
        &self,
        state: Option<&State>,
    ) -> TransitionOutcome<Option<AuthoritativeRecordAspectStateArtifact>, Denial> {
        match self.apply_canonical(state) {
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
        state: &State,
    ) -> TransitionOutcome<AuthoritativeRecordAspectStateArtifact, Denial> {
        match self.apply_canonical(Some(state)) {
            Ok(state) => TransitionOutcome::success(Artifact::<
                AuthoritativeRecordAspectStateAdmitted,
                _,
            >::new(state)),
            Err(denial) => TransitionOutcome::denied(denial),
        }
    }

    fn apply_canonical(&self, state: Option<&State>) -> Result<State, Denial> {
        if state.is_none()
            && let Some(field) = self.field_patches.first()
        {
            return Err(Denial::MissingAspectForFieldPatch(field.key().clone()));
        }
        let count = self.selected_entries(state).count();
        let mut entries = Vec::with_capacity(count);
        let mut fields = self.field_patches.iter().peekable();
        for current in self.selected_entries(state) {
            if fields
                .peek()
                .is_some_and(|field| field.key() < current.key())
            {
                return Err(Denial::MissingAspectForFieldPatch(
                    fields.peek().unwrap().key().clone(),
                ));
            }
            let entry = if fields
                .peek()
                .is_some_and(|field| field.key() == current.key())
            {
                apply_field_patch_to_entry(current, fields.next().unwrap())?
            } else {
                current.clone()
            };
            entries.push(entry);
        }
        if let Some(field) = fields.next() {
            return Err(Denial::MissingAspectForFieldPatch(field.key().clone()));
        }
        Ok(State::from_canonical_map(
            CanonicalAspectStateMap::from_canonical_entries(entries.into_boxed_slice()),
        ))
    }

    pub(super) fn selected_entries<'a>(
        &'a self,
        state: Option<&'a State>,
    ) -> impl Iterator<Item = &'a ContractValidatedAspectValue> {
        let mut current = state
            .into_iter()
            .flat_map(|state| state.aspects().entries().map(|(_, value)| value))
            .peekable();
        let mut sets = self.whole_aspect_sets.iter().peekable();
        let mut clears = self.whole_aspect_clears.iter().peekable();
        std::iter::from_fn(move || loop {
            let use_set = match (current.peek(), sets.peek()) {
                (None, None) => return None,
                (None, Some(_)) => true,
                (Some(_), None) => false,
                (Some(left), Some(right)) => {
                    if left.key() == right.key() {
                        current.next();
                        true
                    } else {
                        right.key() < left.key()
                    }
                }
            };
            let entry = if use_set {
                sets.next()?
            } else {
                current.next()?
            };
            while clears.peek().is_some_and(|clear| clear.key() < entry.key()) {
                clears.next();
            }
            if use_set
                || !clears
                    .peek()
                    .is_some_and(|clear| clear.key() == entry.key())
            {
                return Some(entry);
            }
        })
    }
}

fn apply_field_patch_to_entry(
    current: &ContractValidatedAspectValue,
    patch: &FieldLevelAspectPatch,
) -> Result<ContractValidatedAspectValue, Denial> {
    let ContractValidatedAspectValueView::Struct(current_struct) = current.view() else {
        return Err(Denial::FieldPatchRequiresStructValue(patch.key().clone()));
    };
    let next_struct = super::struct_field_application::apply(current_struct, patch);
    match validate_aspect_value(
        patch.contract(),
        ContractValidationInput::Struct(next_struct),
    ) {
        TransitionOutcome::Success(artifact) => Ok(artifact.into_parts().into_parts().0),
        TransitionOutcome::Denied(denial) => Err(Denial::ContractValidationDenied(denial)),
        TransitionOutcome::Deferred(_)
        | TransitionOutcome::Stale(_)
        | TransitionOutcome::RebindRequired(_)
        | TransitionOutcome::Failed(_) => unreachable!("contract validation uses only denied"),
    }
}
