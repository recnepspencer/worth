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
        binding
            .contract
            .admits_projection_mask(required.mask())
            .map_err(|_| denied(Some(ordinal), Refusal::MaskRejected))?;
        if required.mask_basis().is_none() {
            return Err(denied(Some(ordinal), Refusal::MissingCanonicalMaskBasis));
        }
    }
    Ok(())
}
