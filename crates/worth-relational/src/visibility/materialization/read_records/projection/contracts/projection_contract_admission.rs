use super::{
    ProjectionAspectScope, ProjectionContractRefusal as Refusal, ProjectionRecordDomain,
    RelationalProjectionContractDenial as Denial,
};
use crate::identity::data::KindId;

pub(crate) fn admit_declared_projection_aspects(
    scope: &ProjectionAspectScope,
    plan: Option<&crate::schema::data::LoweredAspectContractPlan>,
    domain: ProjectionRecordDomain,
    kind_id: KindId,
) -> Result<(), Denial> {
    let denied = |requirement, refusal| Denial {
        domain,
        kind_id,
        requirement,
        refusal,
    };
    if scope.is_empty() {
        return Ok(());
    }
    let plan = plan.ok_or_else(|| denied(None, Refusal::MissingAspectPlan))?;
    for (ordinal, required) in scope.requirements().iter().enumerate() {
        let binding = plan
            .executable_bindings
            .iter()
            .find(|binding| binding.aspect_key() == required.aspect_key())
            .ok_or_else(|| denied(Some(ordinal), Refusal::UndeclaredAspect))?;
        // A scalar's sole physical field denotes the entire value. Only its
        // installed binding may translate that field selection into a whole mask.
        let scalar_field = !required.mask().paths().is_empty()
            && required.mask().paths().iter().all(|path| {
                matches!(path.fields(), [field]
                if binding.targets_entity_scalar_field(field))
            });
        let whole = worth_foundational::facade::aspects()
            .projection_mask()
            .whole_aspect();
        binding
            .contract
            .admits_projection_mask(if scalar_field {
                &whole
            } else {
                required.mask()
            })
            .map_err(|_| denied(Some(ordinal), Refusal::MaskRejected))?;
        if required.mask_basis().is_none() {
            return Err(denied(Some(ordinal), Refusal::MissingCanonicalMaskBasis));
        }
    }
    Ok(())
}
