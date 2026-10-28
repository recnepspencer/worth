//! Resolve and verify one authoritative committed outbox record.
use super::*;

pub(super) struct CommittedOutboxRead<'a> {
    pub(super) runtime: &'a mut worth_relational::facade::runtime::RelationalRuntime,
    pub(super) layout: &'a WorthQueryDispatchOutboxLayout,
    pub(super) binding: &'a super::super::WorthQueryCommittedDispatchOutboxBinding,
    pub(super) expected_commit: &'a worth_relational::facade::history::RelationalCommitReceipt,
    pub(super) expected_runtime: u64,
    pub(super) retained_basis: &'a super::super::WorthQueryRetainedApplicationCommitBasis,
}

impl CommittedOutboxRead<'_> {
    pub(super) fn resolve(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryCommittedDispatchOutboxOwnerObservation, Denial> {
        let RecordRef::Entity(entity_id) = self.binding.record_ref() else {
            return Err(Denial::NotAuthoritative);
        };
        let entity_id = *entity_id;
        let (created_at, values, owner_work) = self.read_record(execution, entity_id)?;
        let record = restore_record(values)?;
        if !record.same_persisted_fields(self.binding.record()) {
            return Err(Denial::RecordMismatch);
        }
        let committed = self
            .runtime
            .history()
            .immutable_commit_receipt_for_version(created_at)
            .ok_or(Denial::NotAuthoritative)?;
        if &committed != self.expected_commit {
            return Err(Denial::CommitMismatch);
        }
        Ok(WorthQueryCommittedDispatchOutboxOwnerObservation::seal(
            self.binding.record().clone(),
            committed,
            RecordRef::Entity(entity_id),
            self.expected_runtime,
            WorthQueryCommittedDispatchOutboxReadWork::from_owner(owner_work),
        ))
    }

    fn read_record(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_id: worth_relational::facade::identity::EntityId,
    ) -> Result<
        (
            worth_relational::facade::identity::VersionId,
            Vec<AspectValue>,
            worth_relational::facade::runtime::RelationalRetainedCommitProjectionWork,
        ),
        Denial,
    > {
        let fields = required_fields(self.layout)?;
        let aspect = self
            .layout
            .correlation_locator
            .aspect()
            .aspect_key()
            .clone();
        let scope =
            ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
                aspect.clone(),
                fields.clone(),
            )]);
        let projection = self
            .runtime
            .snapshots()
            .project_retained_entity_for_commit(
                self.expected_runtime,
                self.expected_commit,
                self.retained_basis.lease(),
                entity_id,
                self.layout.entity_kind,
                scope,
                execution,
                |record| {
                    Some(
                        fields
                            .iter()
                            .map(|field| record.aspect_field_value(&aspect, field).cloned())
                            .collect::<Option<Vec<_>>>()
                            .map(|values| (record.created_at_version(), values))
                            .ok_or(Denial::NotAuthoritative),
                    )
                },
            )
            .map_err(map_retained_snapshot_denial)?;
        let (projected, work) = projection.into_parts();
        let (created_at, values) = projected.ok_or(Denial::Missing)??;
        Ok((created_at, values, work))
    }
}
