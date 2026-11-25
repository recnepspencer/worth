//! Applied entity selection carries no committed index-generation identity.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    indexes::{AppliedEntityFieldLookupDenial as Stop, AppliedEntityFieldMatch},
    runtime::{
        RelationalAppliedCandidateReadIdentity, RelationalAppliedCandidateReadQualification,
        RelationalPinnedRead,
    },
};

pub struct ResolvedAppliedEntity {
    pub entity: EntityId,
    pub kind: KindId,
    pub candidate: RelationalAppliedCandidateReadQualification,
}

impl WorthQueryInstalledEntityResolutionContext {
    pub fn resolve_applied_with_work(
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
        self.resolve_applied_merged_with_work(
            execution, before, applied, None, entity, aspect, field, expected,
        )
    }

    /// Resolve at `applied`, or, for a successor round (PR C §1.5), over its
    /// own-output view: `own` is the chain head's pre-pin and the records the
    /// chain wrote exclusively. Those records match as they did at the head,
    /// every other record as it does at `applied`; the merged candidates
    /// decide the answer, so a foreign move or a second match is never hidden
    /// behind one pin's unique result. A lookup that overflowed its limit at
    /// either pin is ambiguous: the filtered set it hides is not known.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph) fn resolve_applied_merged_with_work(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &RelationalPinnedRead,
        applied: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        own: Option<(
            &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
            &dyn Fn(EntityId) -> bool,
        )>,
        entity: &str,
        aspect: &str,
        field: &str,
        expected: &worth_foundational::facade::AspectValue,
    ) -> (
        Result<ResolvedAppliedEntity, WorthQueryEntityResolutionDenial>,
        usize,
    ) {
        let mut examined = 0usize;
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
            super::applied_lookup_control::trigger(self, field);
            let mut lookup = |at: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>| {
                let selected = at
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
                            examined = examined.saturating_add(cause.examined_entry_count());
                            map_index_denial(cause.kind(), field)
                        }
                        Stop::Execution(cause) => entity_denial(
                            WorthQueryEntityResolutionDenialKind::Stopped(cause),
                            field,
                        ),
                    })?;
                examined = examined.saturating_add(selected.examined_entry_count());
                Ok(selected)
            };
            let selected = lookup(applied)?;
            let resolved = |id| ResolvedAppliedEntity {
                entity: id,
                kind: layout.entity_kind,
                candidate: selected.qualification(),
            };
            let Some((head, covers)) = own else {
                return match selected.selection() {
                    AppliedEntityFieldMatch::Absent => {
                        Err(refused(ResolutionRefusal::UnknownEntity))
                    }
                    AppliedEntityFieldMatch::Ambiguous => {
                        Err(refused(ResolutionRefusal::AmbiguousEntity))
                    }
                    AppliedEntityFieldMatch::Unique(id) => Ok(resolved(id)),
                };
            };
            let at_head = lookup(head)?;
            if selected.overflowed() || at_head.overflowed() {
                return Err(refused(ResolutionRefusal::AmbiguousEntity));
            }
            let mut merged = selected
                .candidate_entity_ids()
                .iter()
                .copied()
                .filter(|id| !covers(*id))
                .chain(
                    at_head
                        .candidate_entity_ids()
                        .iter()
                        .copied()
                        .filter(|id| covers(*id)),
                )
                .collect::<Vec<_>>();
            merged.sort_unstable();
            merged.dedup();
            match merged.as_slice() {
                [] => Err(refused(ResolutionRefusal::UnknownEntity)),
                [id] => Ok(resolved(*id)),
                _ => Err(refused(ResolutionRefusal::AmbiguousEntity)),
            }
        })();
        (result, examined)
    }
}

impl WorthQueryInstalledEntityResolutionContext {
    /// Describe the actual D match without inventing a committed index epoch.
    pub fn resolve_applied_identity<Schema, Entity>(
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
        self.resolve_applied_identity_viewed(
            execution, before, candidate, None, entity, aspect, field, expected,
        )
    }

    /// As `resolve_applied_identity`, over a successor round's own-output
    /// view when it has one: the same merged answer its reader resolves.
    #[allow(clippy::too_many_arguments)]
    pub fn resolve_applied_identity_viewed<Schema, Entity>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        own: Option<&crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
        entity: &str,
        aspect: &str,
        field: &str,
        expected: &worth_foundational::facade::AspectValue,
    ) -> Result<WorthQueryApplicationEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    {
        let covers = |entity| own.is_some_and(|own| own.covers(entity));
        let (resolved, examined) = self.resolve_applied_merged_with_work(
            execution,
            before,
            candidate,
            own.map(|own| (own.head(), &covers as &dyn Fn(_) -> bool)),
            entity,
            aspect,
            field,
            expected,
        );
        let resolved = resolved?;
        super::applied_read_observation::record(self, entity, before, candidate);
        let layout = self
            .layout
            .equality_field(entity, aspect, field)
            .expect("resolved installed field");
        Ok(WorthQueryResolvedEntity::from_applied_parts(
            self,
            layout,
            entity,
            expected.clone(),
            resolved.entity,
            layout.entity_kind,
            candidate.candidate_identity().clone(),
            examined,
        )
        .into_application_identity())
    }

    /// Only the exact candidate that issued the description may validate it.
    pub fn validate_applied_identity<Schema, Entity>(
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
    pub fn resolve_applied_field<Schema, Field>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &RelationalPinnedRead,
        candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        own: Option<&crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
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
        self.resolve_applied_identity_viewed(
            execution,
            before,
            candidate,
            own,
            field.entity(),
            field.aspect(),
            field.field(),
            &expected,
        )
    }
}
