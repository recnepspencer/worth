//! Applied entity selection carries no committed index-generation identity.
use super::*;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    indexes::{AppliedEntityFieldLookupDenial as Stop, AppliedEntityFieldMatch},
    runtime::{
        RelationalAppliedCandidateReadIdentity, RelationalAppliedCandidateReadQualification,
        RelationalPinnedRead,
    },
};

pub(in crate::domain_computation::primary_graph) struct ResolvedAppliedEntity {
    pub(in crate::domain_computation::primary_graph) entity: EntityId,
    pub(in crate::domain_computation::primary_graph) kind: KindId,
    pub(in crate::domain_computation::primary_graph) candidate:
        RelationalAppliedCandidateReadQualification,
}

impl WorthQueryInstalledEntityResolutionContext {
    pub(in crate::domain_computation::primary_graph) fn resolve_applied_with_work(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &RelationalPinnedRead,
        applied: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        entity: &str,
        aspect: &str,
        field: &str,
        expected: &worth_foundational::facade::AspectValue,
    ) -> (
        Result<ResolvedAppliedEntity, WorthQueryEntityResolutionDenial>,
        usize,
    ) {
        let mut examined = 0;
        let result = (|| {
            let refused =
                |kind| entity_denial(WorthQueryEntityResolutionDenialKind::Refused(kind), field);
            if applied
                .candidate_identity()
                .proposal()
                .runtime_instance_id()
                != self.relational_runtime_instance_id
                || before.snapshot().runtime_instance_id() != self.relational_runtime_instance_id
            {
                return Err(refused(ResolutionRefusal::ForeignResolutionTruth));
            }
            let layout = self
                .layout
                .equality_field(entity, aspect, field)
                .ok_or_else(|| refused(ResolutionRefusal::FieldNotInstalled))?;
            let index = layout
                .equality_index_id
                .ok_or_else(|| refused(ResolutionRefusal::EqualityIndexUnavailable))?;
            let predecessor = before
                .prepare_entity_field_lookup(index, layout.entity_kind, &layout.locator, execution)
                .map_err(|stop| map_index_denial(stop.kind(), field))?;
            let selected = applied
                .lookup_applied_entity_field(
                    execution,
                    &predecessor,
                    expected,
                    BoundedIndexParityMode::Production,
                    2,
                )
                .map_err(|stop| match stop {
                    Stop::ForeignPredecessor
                    | Stop::ExactBasisRequired
                    | Stop::InvalidCandidateLimit => {
                        refused(ResolutionRefusal::ForeignResolutionTruth)
                    }
                    Stop::PredecessorIndex(cause) => {
                        examined = cause.examined_entry_count();
                        map_index_denial(cause.kind(), field)
                    }
                    Stop::Execution(cause) => {
                        entity_denial(WorthQueryEntityResolutionDenialKind::Stopped(cause), field)
                    }
                })?;
            examined = selected.examined_entry_count();
            match selected.selection() {
                AppliedEntityFieldMatch::Absent => Err(refused(ResolutionRefusal::UnknownEntity)),
                AppliedEntityFieldMatch::Ambiguous => {
                    Err(refused(ResolutionRefusal::AmbiguousEntity))
                }
                AppliedEntityFieldMatch::Unique(id) => Ok(ResolvedAppliedEntity {
                    entity: id,
                    kind: layout.entity_kind,
                    candidate: selected.qualification(),
                }),
            }
        })();
        (result, examined)
    }
}
