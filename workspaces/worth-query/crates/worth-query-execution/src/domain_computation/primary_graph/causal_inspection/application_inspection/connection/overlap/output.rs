//! Classify only admitted output candidates while preserving the exact join.
use super::*;
use worth_relational::facade::{
    publication::PublishedAuthoritativeFieldEffect as FieldEffect, runtime::RelationalFieldPresence,
};
use WorthQueryProducerOutputConnectionDenial as Refusal;

impl ObservationConnection<'_, '_> {
    /// Diagnostic O(roles × observed fields × (installed fields + producer patches)).
    pub(in super::super) fn output_role(
        &self,
        context: &mut MapKernelContext<'_, '_>,
        changes: &WorthQueryApplicationCommittedChanges,
        correspondence: &WorthQueryApplicationOutputCorrespondence,
    ) -> Result<usize, Stop> {
        let mut refusal = Refusal::NoDisclosedOutputCandidate;
        for (role, (_, posture, entity_name, entity)) in
            correspondence.native_witness_roles().enumerate()
        {
            checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
            if posture == WorthQueryApplicationOutputPosture::Retire {
                continue;
            }
            for row in self.original.source.original_row_footprints() {
                for field in &row.aspects {
                    checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
                    // Never describe a field outside the freshly admitted diagnostic mask.
                    if !self.projected(
                        &field.entity_name,
                        field.aspect.as_str(),
                        field.field.as_str(),
                    ) {
                        continue;
                    }
                    if field.entity_name != entity_name {
                        refusal.include(Refusal::OutputKindMismatch);
                        continue;
                    }
                    if field.entity != entity {
                        refusal.include(Refusal::OutputEntityMismatch);
                        continue;
                    }
                    if !row.complete {
                        refusal.include(Refusal::IncompleteObservation);
                        continue;
                    }
                    let Some(revision) = field.native_revision else {
                        refusal.include(Refusal::NativeRevisionUnavailable);
                        continue;
                    };
                    if revision.presence() != RelationalFieldPresence::Present {
                        refusal.include(Refusal::NativeFieldAbsent);
                        continue;
                    }
                    if revision.version() != changes.commit_reference().version_id {
                        refusal.include(Refusal::NativeRevisionNotCreatedByProducer);
                        continue;
                    }
                    let Some(coordinate) = self
                        .layout
                        .resolve_declared_field_coordinate(
                            self.contracts,
                            entity_name,
                            &field.aspect,
                            &field.field,
                            |work| checkpoint(context, work),
                        )
                        .map_err(MapKernelFailure::Stop)?
                    else {
                        refusal.include(Refusal::InstalledFieldUnavailable);
                        continue;
                    };
                    if coordinate.contract_revision() != field.contract_revision {
                        refusal.include(Refusal::ContractRevisionMismatch);
                        continue;
                    }
                    let cause = match changes
                        .field_effect(context, entity, coordinate)
                        .map_err(MapKernelFailure::Stop)?
                    {
                        FieldEffect::Present => return Ok(role),
                        FieldEffect::FieldCleared => Refusal::CanonicalFieldCleared,
                        FieldEffect::AspectCleared => Refusal::CanonicalAspectCleared,
                        FieldEffect::Unchanged => Refusal::CanonicalFieldUnchanged,
                        FieldEffect::Unsupported => Refusal::CanonicalEffectUnsupported,
                        FieldEffect::Ambiguous => Refusal::CanonicalEffectAmbiguous,
                    };
                    refusal.include(cause);
                }
            }
        }
        Err(MapKernelFailure::Domain(ConnectionRefusal::Output(refusal)))
    }
}
