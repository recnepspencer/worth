//! A sparse field demand selects one genuine preserved source output.
use worth_query_declaration::facade::{
    application_aftermath::{DeclaredCorrectionBinding, PortableCorrectionMechanism},
    application_operation::{
        ApplicationMutationBindingDescriptor, ApplicationMutationOutputPosture as Posture,
        ApplicationMutationOutputRoleCardinality as Cardinality,
    },
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
    let Some(output) = demand.changed_output() else {
        return Ok(());
    };
    if binding.authored_identity().is_some()
        || demand.loci().is_empty()
        || output.role().trim().is_empty()
        || output.entity().trim().is_empty()
        || demand
            .loci()
            .iter()
            .any(|locus| locus.entity() != output.entity())
    {
        return Err(());
    }
    if demand
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
        || role.entity() != output.entity()
        || role.posture() != Posture::Preserve
        || role.cardinality() != Cardinality::ExactlyOne
    {
        return Err(());
    }
    Ok(())
}
