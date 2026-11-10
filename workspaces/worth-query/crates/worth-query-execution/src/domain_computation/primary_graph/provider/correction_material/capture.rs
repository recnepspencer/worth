//! Capture the exact admitted input and prior truth at the acknowledged open.
use super::*;
use crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage;
use crate::domain_computation::primary_graph::{
    retained_correction::record::{
        MaterialCapture, RecordedDisclosure, RecordedLocation, RecordedPreImageField, RecordedScope,
    },
    WorthQueryApplicationCommitAuthorityBinding,
};
use worth_foundational::facade::AspectFieldLocator;
use worth_relational::facade::identity::{EntityId, PartitionId};

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph::provider) fn capture_correction_material(
        &self,
        authority: &WorthQueryApplicationCommitAuthorityBinding,
        preimage: Option<&WorthQueryRetainedPreImage>,
        opened: &worth_runtime_world::facade::WorldOpenedPublicationAttempt,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let Some(input) = authority.retained_governed_input_carrier() else {
            return Ok(());
        };
        let Some((canonical, mutation, input_identity)) = input.material_input() else {
            return Ok(());
        };
        let owner = self
            .correction_material
            .get()
            .ok_or(Denial::AdmissionRequired)?;
        let original = opened.association().attempt();
        let binding = authority.principal_scope();
        let entity = |value: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding| {
            EntityId::new(PartitionId::new(value.partition_id()), value.local_slot(), value.generation())
        };
        let scope = input.original_scope().map(|scope| RecordedScope {
            location: location(scope.locator()),
            value: scope.value().clone(),
            subject: scope.subject(),
        });
        let disclosure = input.correction().map(|disclosure| RecordedDisclosure {
            query: *disclosure.query().as_bytes(),
            parameters: disclosure
                .parameters()
                .bindings()
                .iter()
                .map(|(name, value)| (name.to_string(), value.clone()))
                .collect(),
            identity: *disclosure.parameters().identity().bytes(),
        });
        let fields = preimage.map(|preimage| {
            preimage
                .fields()
                .iter()
                .map(|field| RecordedPreImageField {
                    entity: field.locus().entity().to_owned(),
                    aspect: field.locus().aspect().to_owned(),
                    field: field.locus().field().to_owned(),
                    value: field.value().clone(),
                    target: field.target_record().clone(),
                    kind: field.entity_kind(),
                    location: location(field.locator()),
                    encoded_bytes: field.encoded_bytes(),
                })
                .collect()
        });
        owner.ensure_capture(
            MaterialCapture {
                original,
                program: *opened.program_revision().bytes(),
                mutation: mutation.to_owned(),
                key: *authority.idempotency_binding().key_identity(),
                intent: authority.idempotency_binding().intent_text(),
                input_identity,
                intent_identity: *authority.idempotency_binding().intent_identity(),
                input: canonical.recorded_bytes().to_vec(),
                principal: entity(binding.principal()),
                scope_entity: entity(binding.scope()),
                scope,
                disclosure,
                preimage: fields,
                content: preimage
                    .map(|preimage| preimage.content().clone())
                    .unwrap_or_default(),
            },
            request,
        )
    }
}
pub(super) fn location(locator: &AspectFieldLocator) -> RecordedLocation {
    RecordedLocation {
        aspect: locator.aspect().aspect_key().as_str().to_owned(),
        fields: locator
            .field_path()
            .fields()
            .iter()
            .map(|field| field.as_str().to_owned())
            .collect(),
    }
}
