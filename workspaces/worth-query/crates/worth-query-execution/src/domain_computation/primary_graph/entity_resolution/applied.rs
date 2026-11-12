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
            super::applied_lookup_control::trigger(self);
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

impl WorthQueryInstalledEntityResolutionContext {
    /// Describe the actual D match without inventing a committed index epoch.
    pub(in crate::domain_computation) fn resolve_applied_identity<Schema, Entity>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        entity: &str,
        aspect: &str,
        field: &str,
        expected: &worth_foundational::facade::AspectValue,
    ) -> Result<WorthQueryApplicationEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    {
        let (resolved, examined) = self.resolve_applied_with_work(
            execution, before, candidate, entity, aspect, field, expected,
        );
        let resolved = resolved?;
        super::applied_read_observation::record(self, entity, before, candidate);
        let layout = self
            .layout
            .equality_field(entity, aspect, field)
            .expect("resolved installed field");
        Ok(WorthQueryResolvedEntity::from_applied(
            self,
            layout,
            entity,
            expected.clone(),
            resolved,
            examined,
        )
        .into_application_identity())
    }

    /// Only the exact candidate that issued the description may validate it.
    pub(in crate::domain_computation) fn validate_applied_identity<Schema, Entity>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        identity: &WorthQueryApplicationEntityIdentity<Schema, Entity>,
    ) -> Result<(), WorthQueryEntityResolutionDenial> {
        let foreign = || {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ForeignResolutionTruth,
                ),
                identity.entity_name(),
            )
        };
        if identity.runtime_authority() != self.runtime_authority
            || identity.binding_identity() != &self.binding_identity
            || identity.applied_qualification()
                != Some(candidate.candidate_identity().qualification())
        {
            return Err(foreign());
        }
        let locator = identity.identity_locator();
        let [field] = locator.field_path().fields() else {
            return Err(foreign());
        };
        let (selected, _) = self.resolve_applied_with_work(
            execution,
            before,
            candidate,
            identity.entity_name(),
            locator.aspect().aspect_key().as_str(),
            field.as_str(),
            identity.identity_value(),
        );
        let selected = selected?;
        if selected.entity != identity.entity_id() || selected.kind != identity.entity_kind() {
            return Err(foreign());
        }
        Ok(())
    }
}

impl WorthQueryInstalledEntityResolutionContext {
    pub(in crate::domain_computation) fn resolve_applied_field<Schema, Field>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        field: Field,
        value: Field::Value,
    ) -> Result<
        WorthQueryApplicationEntityIdentity<Schema, Field::Entity>,
        WorthQueryEntityResolutionDenial,
    >
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection,
    {
        let refused = || {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::Refused(
                    ResolutionRefusal::ValueEncodingRejected,
                ),
                field.field(),
            )
        };
        Field::Binding::validate(&value).map_err(|_| refused())?;
        let expected = Field::Binding::encode(&value).map_err(|_| refused())?;
        self.resolve_applied_identity(
            execution,
            before,
            candidate,
            field.entity(),
            field.aspect(),
            field.field(),
            &expected,
        )
    }
}
