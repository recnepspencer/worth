use serde::{Deserialize, Serialize};
use worth_foundational::facade::{
    AspectKey, AuthoritativeRecordAspectState, ContractValidatedAspectValueView,
};

use crate::publication::patch::data::PublishedAuthoritativePatch;

use super::{ProjectionAspectRequirement, ProjectionAspectScope};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ProjectionAspectFilterMode {
    Any,
    All,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionAspectFilter {
    mode: ProjectionAspectFilterMode,
    projection_scope: ProjectionAspectScope,
}

impl ProjectionAspectFilter {
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.projection_scope.owned_allocation_capacity_bytes()
    }

    pub fn new(mode: ProjectionAspectFilterMode, projection_scope: ProjectionAspectScope) -> Self {
        Self {
            mode,
            projection_scope,
        }
    }

    pub fn whole_aspects(
        mode: ProjectionAspectFilterMode,
        aspects: impl IntoIterator<Item = AspectKey>,
    ) -> Self {
        Self::new(mode, ProjectionAspectScope::whole_aspects(aspects))
    }

    pub const fn mode(&self) -> ProjectionAspectFilterMode {
        self.mode
    }

    pub fn projection_scope(&self) -> &ProjectionAspectScope {
        &self.projection_scope
    }

    pub fn matches_authoritative_state(
        &self,
        state: Option<&AuthoritativeRecordAspectState>,
    ) -> bool {
        match self.mode {
            ProjectionAspectFilterMode::Any => self
                .projection_scope
                .requirements()
                .iter()
                .any(|requirement| requirement_matches_state(requirement, state)),
            ProjectionAspectFilterMode::All => self
                .projection_scope
                .requirements()
                .iter()
                .all(|requirement| requirement_matches_state(requirement, state)),
        }
    }

    pub fn matches_published_patch(&self, patch: &PublishedAuthoritativePatch) -> bool {
        match self.mode {
            ProjectionAspectFilterMode::Any => self
                .projection_scope
                .requirements()
                .iter()
                .any(|requirement| requirement_matches_patch(requirement, patch)),
            ProjectionAspectFilterMode::All => self
                .projection_scope
                .requirements()
                .iter()
                .all(|requirement| requirement_matches_patch(requirement, patch)),
        }
    }
}

fn requirement_matches_state(
    requirement: &ProjectionAspectRequirement,
    state: Option<&AuthoritativeRecordAspectState>,
) -> bool {
    let Some(value) = state.and_then(|state| state.get(requirement.aspect_key())) else {
        return false;
    };
    if requirement.mask().is_whole_aspect() {
        return true;
    }
    let ContractValidatedAspectValueView::Struct(struct_value) = value.view() else {
        return false;
    };
    requirement.mask().paths().iter().all(|path| {
        let [field] = path.fields() else {
            return false;
        };
        struct_value.get(field).is_some()
    })
}

fn requirement_matches_patch(
    requirement: &ProjectionAspectRequirement,
    patch: &PublishedAuthoritativePatch,
) -> bool {
    if requirement.mask().is_whole_aspect() {
        return patch
            .changed_aspect_keys()
            .any(|aspect_key| aspect_key == requirement.aspect_key());
    }
    requirement.mask().paths().iter().all(|path| {
        let [field] = path.fields() else {
            return false;
        };
        patch
            .field_sets_for(requirement.aspect_key())
            .any(|field_set| &field_set.field == field)
            || patch
                .field_clears_for(requirement.aspect_key())
                .any(|cleared_field| cleared_field == field)
    })
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::{
        admit_authoritative_record_aspect_state, validate_aspect_value, AspectContract,
        AspectContractRevision, AspectIdentity, AspectValue, ContractValidationInput, FieldKey,
        ScalarAspectType, StructAspectShape, StructAspectValue,
    };
    use worth_proof::TransitionOutcome;

    use super::*;

    #[test]
    fn field_mask_filter_matches_only_present_struct_field() {
        let aspect_key = AspectKey::new("summary").unwrap();
        let title = FieldKey::new("title").unwrap();
        let status = FieldKey::new("status").unwrap();
        let filter = ProjectionAspectFilter::new(
            ProjectionAspectFilterMode::All,
            ProjectionAspectScope::fields(aspect_key.clone(), [title.clone()]),
        );
        let sibling_filter = ProjectionAspectFilter::new(
            ProjectionAspectFilterMode::All,
            ProjectionAspectScope::fields(aspect_key.clone(), [status]),
        );
        let state = summary_state(
            aspect_key,
            title,
            AspectValue::String("visible-title".into()),
        );

        assert!(filter.matches_authoritative_state(Some(&state)));
        assert!(!sibling_filter.matches_authoritative_state(Some(&state)));
    }

    #[test]
    fn field_mask_filter_does_not_match_scalar_aspect_presence() {
        let aspect_key = AspectKey::new("name").unwrap();
        let field = FieldKey::new("name").unwrap();
        let filter = ProjectionAspectFilter::new(
            ProjectionAspectFilterMode::All,
            ProjectionAspectScope::fields(aspect_key.clone(), [field]),
        );
        let state = scalar_state(aspect_key, AspectValue::String("Ada".into()));

        assert!(!filter.matches_authoritative_state(Some(&state)));
    }

    fn summary_state(
        aspect_key: AspectKey,
        field: FieldKey,
        value: AspectValue,
    ) -> AuthoritativeRecordAspectState {
        let shape = StructAspectShape::new([worth_foundational::facade::FieldDeclaration::new(
            field.clone(),
            ScalarAspectType::String,
            worth_foundational::facade::FieldRequirement::Required,
            worth_foundational::facade::AbsenceLaw::Required,
            worth_foundational::facade::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .unwrap()])
        .unwrap();
        let contract = AspectContract::struct_aspect(
            aspect_key,
            AspectIdentity(1),
            AspectContractRevision(1),
            shape,
        );
        let struct_value = StructAspectValue::new([(field, value)]).unwrap();
        authoritative_state(&contract, ContractValidationInput::Struct(struct_value))
    }

    fn scalar_state(aspect_key: AspectKey, value: AspectValue) -> AuthoritativeRecordAspectState {
        let contract = AspectContract::scalar(
            aspect_key,
            AspectIdentity(2),
            AspectContractRevision(1),
            ScalarAspectType::String,
        );
        authoritative_state(&contract, ContractValidationInput::Scalar(value))
    }

    fn authoritative_state(
        contract: &AspectContract,
        input: ContractValidationInput,
    ) -> AuthoritativeRecordAspectState {
        let TransitionOutcome::Success(validated) = validate_aspect_value(contract, input) else {
            panic!("test value should validate");
        };
        let TransitionOutcome::Success(state) =
            admit_authoritative_record_aspect_state([validated])
        else {
            panic!("validated aspect should admit");
        };
        let (state, _proofs, _basis) = state.into_parts().into_parts();
        state
    }
}
