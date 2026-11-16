//! A sparse field demand selects one genuine preserved source output.
//!
//! Every demanded field is captured at that exact output or is unchanged there.
//! The only other fields allowed are those of live-owner member families, which
//! are captured completely on each member a mutation moves. Their entities never
//! equal the output's, so the two captures partition the vocabulary exactly.
use worth_query_declaration::facade::{
    application_aftermath::{
        DeclaredAuthoredCorrection, DeclaredCorrectionBinding, PortableCorrectionMechanism,
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
    let Some(output) = demand.output_fields() else {
        return Ok(());
    };
    if output.capture() != worth_query_declaration::facade::application_aftermath::PreImageOutputFieldCapture::ChangedPreserved {
        return Ok(());
    }
    let families = member_family_entities(binding.authored_identity())?;
    if families.contains(&output.entity())
        || output.role().trim().is_empty()
        || output.entity().trim().is_empty()
        || !demand
            .loci()
            .iter()
            .any(|locus| locus.entity() == output.entity())
        || demand
            .loci()
            .iter()
            .any(|locus| locus.entity() != output.entity() && !families.contains(&locus.entity()))
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

/// The entities of the identity's member families. Only live-owner member
/// families may stand beside a sparse output: an anchored or owned-member
/// identity creates or retires its own anchor, which a preserved output is not.
fn member_family_entities(identity: Option<&DeclaredAuthoredCorrection>) -> Result<Vec<&str>, ()> {
    let Some(identity) = identity else {
        return Ok(Vec::new());
    };
    std::iter::once(identity)
        .chain(identity.co_anchors())
        .map(|family| {
            if family.live_member_owner().is_none() || family.member_owner().is_some() {
                return Err(());
            }
            match family.parts().logical_key.as_ref() {
                ApplicationSchemaMember::Field { entity, .. } => Ok(entity.as_str()),
                _ => Err(()),
            }
        })
        .collect()
}
