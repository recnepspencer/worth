//! Native current attestation observation, never conversion of descriptive bytes into a grant.
mod related;
use super::{
    ApplicationSchema, WorthQueryApplicationOperationInvariantProjectionReader,
    WorthQueryInvariantEntityIdentity,
};
use crate::domain_computation::primary_graph::{
    application_contribution::{advancement_denial, QueryRequestExecution},
    operation_body::PreparedRequirementOperand,
    HandlerExecutionDenial, WorthQueryAdvancementDenial, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind,
};
use worth_foundational::expression_api::ExpressionValue;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyRequirementDirection as Direction,
    ApplicationOperationBodyRequirementSource as Operand, ApplicationReadableScalarValueBinding,
    ApplicationSigningEvidenceSource as Source,
    ApplicationSigningSubjectProvenanceBinding as Binding,
};

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub(super) fn signed_evidence_operand<Entity>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        source: &Source,
        execution: &QueryRequestExecution<'_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<PreparedRequirementOperand, HandlerExecutionDenial> {
        for target in source.decision_reads() {
            self.admit_decision_target(&target)
                .map_err(HandlerExecutionDenial::new)?;
        }
        let collection = Operand::RelatedItemCount {
            relation: source.lifecycle.current_relation.clone(),
            from: source.lifecycle.source_entity.clone(),
            to: source.evidence_entity.clone(),
            direction: Direction::Outgoing,
        };
        // Reuse the bounded collection observer and its exact adjacency sealing.
        let observed =
            self.observed_requirement_collection(identity, &collection, execution, context)?;
        if observed.ids.len() > source.lifecycle.maximum_evidence {
            return Err(invalid());
        }
        let (program_revision, _program_memory) = self.active_evidence_program(execution)?;
        let kind = self
            .reader
            .layout
            .entity_kind(&source.evidence_entity)
            .ok_or_else(invalid)?;
        let mut found = false;
        for entity_id in &observed.ids {
            context.checkpoint(1).map_err(|cause| {
                HandlerExecutionDenial::new(advancement_denial(
                    worth_execution::WorkCeilingDenial::Stopped(cause),
                ))
            })?;
            let evidence = WorthQueryInvariantEntityIdentity::<Schema, ()> {
                entity_id: *entity_id,
                kind,
                entity: source.evidence_entity.clone().into(),
                authority_identity: self.reader.authority_identity,
                _marker: std::marker::PhantomData,
            };
            let locator = self
                .tracked_body_locator(&evidence, &source.provenance)
                .map_err(HandlerExecutionDenial::new)?;
            found |= self
                .reader
                .body_scalar_observe(&evidence, &locator, |raw| {
                    let raw = raw.ok_or_else(invalid)?;
                    let bytes = raw
                        .owned_allocation_capacity_bytes()
                        .checked_mul(8)
                        .and_then(|bytes| u64::try_from(bytes).ok())
                        .ok_or_else(invalid)?;
                    let _memory = execution.reserve(bytes).map_err(|cause| {
                        HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
                    })?;
                    let provenance = Binding::decode(raw).map_err(|_| invalid())?;
                    let subject = provenance.authentication().subject;
                    if subject.partition != identity.entity_id.partition_value_u64()
                        || subject.slot != identity.entity_id.local_slot_value()
                        || subject.generation != identity.entity_id.generation_value()
                        || provenance
                            .dependencies()
                            .iter()
                            .any(|dependency| dependency.source != subject)
                    {
                        return Err(invalid());
                    }
                    let metadata = &provenance.authentication().metadata;
                    Ok(metadata.program_revision == program_revision
                        && metadata.binding == source.mutation_binding
                        && metadata.operation == source.operation
                        && metadata.purpose == source.purpose)
                })?;
        }
        Ok(PreparedRequirementOperand {
            value: ExpressionValue::bool(found),
            _reservation: observed.reservation,
        })
    }

    // Platform-owned activation is already fenced by native program readmission/CAS;
    // it is not a supplier field permission or a host-provided expected revision.
    fn active_evidence_program(&mut self, execution: &QueryRequestExecution<'_>) -> Result<(String,
    crate::domain_computation::primary_graph::application_contribution::QueryMemoryReservation), HandlerExecutionDenial>{
        let entity_id = self
            .reader
            .program_activation
            .and_then(|cell| cell.published())
            .ok_or_else(invalid)?;
        let layout = self.reader.layout.program_activation().clone();
        let identity = WorthQueryInvariantEntityIdentity::<Schema, ()> {
            entity_id,
            kind: layout.entity_kind,
            entity: "native program activation".into(),
            authority_identity: self.reader.authority_identity,
            _marker: std::marker::PhantomData,
        };
        self.reader
            .body_scalar_observe(&identity, &layout.program_revision_locator, |raw| {
                let Some(worth_foundational::facade::AspectValue::String(
                    worth_foundational::facade::InternedString::Raw(text),
                )) = raw
                else {
                    return Err(invalid());
                };
                let bytes = u64::try_from(text.len()).map_err(|_| invalid())?;
                let reservation = execution.reserve(bytes).map_err(|cause| {
                    HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
                })?;
                Ok((text.clone(), reservation))
            })
    }
}
fn invalid() -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::InvalidAuthoritativeValue,
        "current signed evidence",
    ))
}
