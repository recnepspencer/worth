//! Seal material from the actual performed original before caller completion.
use super::super::WorthQueryPrimaryGraphCommittedApplication;
use super::*;
use crate::domain_computation::primary_graph::retained_correction::record::{
    MaterialReady, RecordedOutputField, RecordedOutputRole, RecordedSourceFacts,
    RecordedSourceFactsUnavailable,
};
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, CanonicalFieldPath,
    DurableCompositePublicationAttemptIdentity, FieldKey, LocatorAuthority,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture;

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph::provider) fn seal_correction_material(
        &self,
        application: &WorthQueryPrimaryGraphCommittedApplication,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let evidence = application.commit_evidence();
        let Some(input) = evidence
            .original_authority()
            .retained_governed_input_carrier()
        else {
            return Ok(());
        };
        if input.material_input().is_none() {
            return Ok(());
        }
        let attempt = application
            .committed_product_publication()
            .publication_attempt();
        let Some(home_owner) = attempt.owner_identity().durable() else {
            return Ok(());
        };
        let original = DurableCompositePublicationAttemptIdentity::from_recorded(
            home_owner,
            attempt.ordinal(),
        );
        let owner = self
            .correction_material
            .get()
            .ok_or(Denial::AdmissionRequired)?;
        let roles: Vec<_> = evidence
            .output_correspondence()
            .native_witness_roles()
            .map(|(role, posture, entity_name, entity)| RecordedOutputRole {
                role: role.to_owned(),
                posture: match posture {
                    ApplicationMutationOutputPosture::Preserve => 0,
                    ApplicationMutationOutputPosture::Create => 1,
                    ApplicationMutationOutputPosture::Retire => 2,
                },
                entity_name: entity_name.to_owned(),
                entity,
            })
            .collect();
        let mut fields = Vec::new();
        let mut content =
            worth_relational::facade::content::RelationalContentAttachments::default();
        if let Some(disclosure) = input.correction() {
            for declared in disclosure
                .association()
                .declaration()
                .source_output_fields()
            {
                request.consult().map_err(Denial::Execution)?;
                let parts = declared.parts();
                let role = roles
                    .iter()
                    .find(|role| {
                        role.role == parts.role
                            && role.entity_name == parts.entity
                            && role.posture == 1
                    })
                    .ok_or(Denial::OriginalMismatch)?;
                let locator = AspectFieldLocator::new(
                    LocatorAuthority::Planned,
                    AspectKey::new(&parts.aspect).ok_or(Denial::OriginalMismatch)?,
                    CanonicalFieldPath::single(
                        FieldKey::new(&parts.field).ok_or(Denial::OriginalMismatch)?,
                    ),
                );
                let value = evidence
                    .committed_changes()
                    .committed_field_ref(request, role.entity, &locator)
                    .map_err(|cause| {
                        Denial::Execution(worth_execution::WorkCeilingDenial::Stopped(cause))
                    })?
                    .ok_or(Denial::Unavailable)?;
                if let AspectValue::ContentRef(identity) = value {
                    let frame = evidence
                        .committed_changes()
                        .content_for_field(request, role.entity, &parts.entity, &locator)
                        .map_err(|_| Denial::MaterialContentUnavailable)?;
                    if frame.identity() != *identity {
                        return Err(Denial::OriginalMismatch);
                    }
                    content
                        .attach(request, &frame)
                        .map_err(|_| Denial::MaterialContentUnavailable)?;
                }
                fields.push(RecordedOutputField {
                    role: parts.role.clone(),
                    entity_name: parts.entity.clone(),
                    entity: role.entity,
                    location: super::capture::location(&locator),
                    value: value.clone(),
                });
            }
        }
        super::authored_key::retain(application, &roles, &mut fields, request)?;
        let source_facts = match evidence.rebased_source_facts() {
            Ok(facts) if facts.is_empty() => RecordedSourceFacts::Empty,
            Ok(facts) => match crate::domain_computation::primary_graph::application_checkpoint::facts::encode(&facts) {
                Some(bytes) => RecordedSourceFacts::Exact {
                    version: crate::domain_computation::primary_graph::application_checkpoint::facts::WIRE_VERSION, bytes,
                },
                None => RecordedSourceFacts::Unavailable(RecordedSourceFactsUnavailable::UnsupportedEncoding),
            },
            Err(_) => RecordedSourceFacts::Unavailable(RecordedSourceFactsUnavailable::OriginalRebase),
        };
        owner.ensure_ready(
            MaterialReady {
                original,
                commit: application.commit_reference().clone(),
                roles,
                output_fields: fields,
                content,
                source_facts,
            },
            request,
        )
    }
}
