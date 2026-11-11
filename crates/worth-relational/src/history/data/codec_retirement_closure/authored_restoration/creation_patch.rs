//! Recreate portable authored fields, leaving relation endpoints to their own authority.
use super::*;
pub(in crate::history::data::codec_retirement_closure) fn creation_patch(
    state: Option<&AuthoritativeRecordAspectState>,
    plan: &crate::schema::data::LoweredAspectContractPlan,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<PortableRecordAspectPatch, Stop> {
    let Some(state) = state else {
        return Ok(PortableRecordAspectPatch::new([]));
    };
    charge(
        context,
        state.owned_allocation_capacity_bytes().saturating_add(
            state
                .aspects()
                .len()
                .saturating_mul(plan.executable_bindings.len()),
        ),
    )?;
    Ok(PortableRecordAspectPatch::new(
        state
            .aspects()
            .entries()
            .filter(|(key, _)| {
                !plan.executable_bindings.iter().any(|binding| {
                    binding.aspect_key() == *key
                        && matches!(
                            binding.target,
                            crate::schema::data::AspectBinding::RelationSourceEndpoint
                                | crate::schema::data::AspectBinding::RelationTargetEndpoint
                        )
                })
            })
            .map(|(_, value)| PortableAspectPatchOperation::SetWhole {
                basis: PortableAspectContractBasis::from_contract(value.contract()),
                value: match value.view() {
                    ContractValidatedAspectValueView::Scalar(value) => {
                        ContractValidationInput::Scalar(value.clone())
                    }
                    ContractValidatedAspectValueView::Struct(value) => {
                        ContractValidationInput::Struct(value.clone())
                    }
                },
            }),
    ))
}
