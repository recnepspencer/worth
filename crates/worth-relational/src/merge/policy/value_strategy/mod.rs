mod auto_resolution;
mod value_basis;
pub(super) use value_basis::performance_receipt::PolicyValueLookupCounters;

use crate::schema::data::{AspectBinding, LoweredAspectContractBinding};

pub(super) use auto_resolution::{resolve_aspect_value_strategy, AutoResolutionStrategy};
pub(crate) use value_basis::{
    resolve_policy_aspect_value_basis, PolicyAspectValueBasis, ScalarPolicyAspectBinding,
    ScalarPolicyBindingDenial,
};

pub(super) fn scalar_policy_aspect_binding(
    record_kind: crate::merge::data::VisibleMergeRecordKind,
    binding: Option<&LoweredAspectContractBinding>,
) -> Result<ScalarPolicyAspectBinding, ScalarPolicyBindingDenial> {
    let binding = binding.ok_or(ScalarPolicyBindingDenial::MissingBinding)?;
    match (&binding.target, binding.contract.shape()) {
        (AspectBinding::EntityField { .. }, worth_foundational::AspectShape::Scalar(_)) => {
            ScalarPolicyAspectBinding::entity(record_kind, binding.aspect_key().clone())
        }
        (AspectBinding::EntityField { .. }, worth_foundational::AspectShape::Struct(_)) => {
            Err(ScalarPolicyBindingDenial::InvalidBuiltInPolicyValueShape)
        }
        (AspectBinding::RelationField { .. }, worth_foundational::AspectShape::Scalar(_)) => {
            ScalarPolicyAspectBinding::relation(record_kind, binding.aspect_key().clone())
        }
        (AspectBinding::RelationField { .. }, worth_foundational::AspectShape::Struct(_)) => {
            Err(ScalarPolicyBindingDenial::InvalidBuiltInPolicyValueShape)
        }
        _ => Err(ScalarPolicyBindingDenial::MissingBinding),
    }
}
