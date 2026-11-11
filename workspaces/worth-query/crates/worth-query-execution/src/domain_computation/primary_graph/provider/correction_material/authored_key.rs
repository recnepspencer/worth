//! Retain a declared authored key even when it is not a disclosed output field.
use super::super::WorthQueryPrimaryGraphCommittedApplication;
use super::*;
use crate::domain_computation::primary_graph::retained_correction::record::{
    RecordedOutputField, RecordedOutputRole,
};
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, CanonicalFieldPath, FieldKey, LocatorAuthority,
};
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationOutputPosture as Posture,
    application_schema::ApplicationSchemaMember,
};

pub(super) fn retain(
    application: &WorthQueryPrimaryGraphCommittedApplication,
    roles: &[RecordedOutputRole],
    fields: &mut Vec<RecordedOutputField>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let evidence = application.commit_evidence();
    let Some(authored) = evidence
        .original_authority()
        .retained_governed_input_carrier()
        .and_then(|input| input.correction())
        .and_then(|disclosure| disclosure.association().declaration().authored_identity())
    else {
        return Ok(());
    };
    let parts = authored.parts();
    let ApplicationSchemaMember::Field {
        entity,
        aspect,
        field,
        ..
    } = parts.logical_key.as_ref()
    else {
        return Err(Denial::OriginalMismatch);
    };
    let code = match parts.source_posture {
        Posture::Preserve => return Err(Denial::OriginalMismatch),
        Posture::Create => 1,
        Posture::Retire => 2,
    };
    let role = roles
        .iter()
        .find(|role| {
            role.role == parts.source_role && role.entity_name == *entity && role.posture == code
        })
        .ok_or(Denial::OriginalMismatch)?;
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new(aspect).ok_or(Denial::OriginalMismatch)?,
        CanonicalFieldPath::single(FieldKey::new(field).ok_or(Denial::OriginalMismatch)?),
    );
    let value = match parts.source_posture {
        Posture::Create => evidence
            .committed_changes()
            .committed_field_ref(request, role.entity, &locator)
            .map_err(|cause| Denial::Execution(worth_execution::WorkCeilingDenial::Stopped(cause)))?
            .ok_or(Denial::Unavailable)?,
        Posture::Retire => evidence
            .retained_preimage()
            .ok_or(Denial::Unavailable)?
            .fields()
            .iter()
            .find(|value| {
                value.target_record()
                    == &worth_relational::facade::transactions::RecordRef::Entity(role.entity)
                    && value.locator() == &locator
                    && value.locus().entity() == entity
            })
            .ok_or(Denial::Unavailable)?
            .value(),
        Posture::Preserve => unreachable!("unsupported posture refused before key selection"),
    };
    let location = super::capture::location(&locator);
    if let Some(existing) = fields
        .iter()
        .find(|value| value.role == role.role && value.location == location)
    {
        if existing.entity != role.entity
            || existing.entity_name != *entity
            || &existing.value != value
        {
            return Err(Denial::OriginalMismatch);
        }
    } else {
        fields.push(RecordedOutputField {
            role: role.role.clone(),
            entity_name: entity.clone(),
            entity: role.entity,
            location,
            value: value.clone(),
        });
    }
    Ok(())
}
