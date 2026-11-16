//! Complete field capture belongs to the primary authored retired output.
use worth_query_declaration::facade::{
    application_aftermath::{
        DeclaredCorrectionBinding, PortableCorrectionMechanism, PreImageOutputFieldCapture,
    },
    application_operation::{
        ApplicationMutationBindingDescriptor, ApplicationMutationOutputPosture as Posture,
        ApplicationMutationOutputRoleCardinality as Cardinality,
    },
    application_schema::ApplicationSchemaMember,
};

pub(super) fn validate(
    source: &ApplicationMutationBindingDescriptor,
    binding: &DeclaredCorrectionBinding,
    mechanism: Option<&PortableCorrectionMechanism>,
) -> Result<(), ()> {
    let Some(PortableCorrectionMechanism::RecordedInverse(inverse)) = mechanism else {
        return Ok(());
    };
    let demand = inverse.preimage_demand();
    let Some(output) = demand
        .output_fields()
        .filter(|output| output.capture() == PreImageOutputFieldCapture::CompleteRetired)
    else {
        return Ok(());
    };
    let identity = binding.authored_identity().ok_or(())?;
    let parts = identity.parts();
    let ApplicationSchemaMember::Field {
        entity,
        aspect,
        field,
        ..
    } = parts.logical_key.as_ref()
    else {
        return Err(());
    };
    if parts.source_posture != Posture::Retire
        || parts.source_role != output.role()
        || entity != output.entity()
        || identity.member_owner().is_some()
        || identity.live_member_owner().is_some()
        || identity.is_optional()
        || demand.maximum_encoded_bytes() == 0
        || !demand.loci().iter().any(|locus| {
            locus.entity() == entity && locus.aspect() == aspect && locus.field() == field
        })
        || demand.loci().iter().any(|locus| locus.entity() != entity)
        || demand
            .loci()
            .iter()
            .enumerate()
            .any(|(index, locus)| demand.loci()[..index].contains(locus))
    {
        return Err(());
    }
    let mut roles = source
        .output_roles()
        .iter()
        .filter(|role| role.name() == output.role());
    let role = roles.next().ok_or(())?;
    if roles.next().is_some()
        || role.entity() != entity
        || role.posture() != Posture::Retire
        || role.cardinality() != Cardinality::ExactlyOne
    {
        return Err(());
    }
    Ok(())
}
