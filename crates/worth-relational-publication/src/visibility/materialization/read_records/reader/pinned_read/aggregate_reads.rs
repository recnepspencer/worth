//! Bind aggregate reads to the immutable Native pin.
use super::RelationalPinnedRead;
use mutation::aggregate_identity::pinned::denial::PinnedAggregateReadDenial;
use mutation::aggregate_identity::pinned::lineage::read_aggregate_lineage;
use mutation::aggregate_identity::pinned::lineage::PinnedAggregateLineageRead;
use mutation::aggregate_identity::pinned::lineage::PinnedAggregateLineageScope;
use mutation::aggregate_identity::pinned::read_aggregate_scope;
use mutation::aggregate_identity::pinned::named::{
    NamedAggregateLookupDenial as LookupDenial, NamedAggregateRead,
};
use contracts::aggregate_identity::transition::group::model::{
    AggregateGroupScheme, CanonicalGroupMemberIdentity,
};
use contracts::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use contracts::history::data::coordinates::CommitId;
use contracts::identity::data::primitives::LineageId;
use contracts::lineage::data::name_resolution::{PersistentNameKey, PersistentNameOfRecord};
use contracts::lineage::data::postures::LineageRecordTarget;
use contracts::storage::data::storage_unavailable::{
    RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause as StorageCause,
};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_lineage::internal_port::lineage::access::name_observation_view::PersistentNameObservationViewOwnerPort as _;
use worth_relational_lineage::internal_port::lineage::access::name_observation_view::PersistentNameObservationView;
use worth_relational_lineage::internal_port::lineage::groups::GroupStateDenial;
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::naming_basis_binding::BranchNamingBasisGuard;
use mutation::aggregate_identity::pinned::result::PinnedAggregateRead;
use mutation::aggregate_identity::pinned::PinnedAggregateReadScope;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port::graph_member::key::NativeMemberKey;
use worth_relational_mutation::internal_port as mutation;

impl<Snapshot> RelationalPinnedRead<Snapshot> {
    /// Read one complete stored group and all its incident lineage from this
    /// exact root/schema. Work grows with its relation-kind roster, group and
    /// raw incident adjacency. No current catalog, whole-graph materialization
    /// or identity-only lookup is used.
    pub fn read_aggregate(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: PinnedAggregateReadScope<'_>,
    ) -> Result<PinnedAggregateRead, PinnedAggregateReadDenial> {
        read_aggregate_scope(
            self.pinned_basis(),
            self.candidate_aggregate(),
            execution,
            scope,
            |scheme, member| self.read_named_member(scheme, member),
        )
    }

    pub fn read_projected_aggregate(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: PinnedAggregateReadScope<'_, NativeMemberKey>,
    ) -> Result<PinnedAggregateRead<NativeMemberKey>, PinnedAggregateReadDenial> {
        read_aggregate_scope(
            self.pinned_basis(),
            self.candidate_aggregate(),
            execution,
            scope,
            |scheme, member| self.read_named_member(scheme, member),
        )
    }

    /// Scan the selected schema roster and occupied entity slots once, then
    /// follow actual merge targets. Work includes radix/slot navigation and
    /// raw incident adjacency at each hop; no constant-time lookup is claimed.
    pub fn read_aggregate_lineage(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        scope: PinnedAggregateLineageScope<'_>,
    ) -> Result<PinnedAggregateLineageRead, PinnedAggregateReadDenial> {
        read_aggregate_lineage(self.pinned_basis(), execution, scope, |scheme, identity| {
            self.with_named_group_basis(|_, names, _| {
                names
                    .aggregate_group_history(scheme.id().as_str(), identity)
                    .ok_or(LookupDenial::IdentityUnavailable)
            })
        })
    }
}

impl<Snapshot> RelationalPinnedRead<Snapshot> {
    fn read_named_member(
        &self,
        scheme: &AggregateGroupScheme,
        member: &NativeMemberKey,
    ) -> Result<NamedAggregateRead, LookupDenial> {
        self.with_named_group_basis(|basis, names, commit| {
            let record = LineageRecordTarget::Entity(member.carrier());
            let PersistentNameOfRecord::Live {
                name: PersistentNameKey::Entity(carrier),
                ..
            } = names.name_of_record(record)
            else {
                return Err(LookupDenial::MemberUnavailable);
            };
            let key = CanonicalGroupMemberIdentity::for_member(carrier, member);
            let state = basis
                .group_state_at(self.pinned_basis().branch_id(), commit)
                .map_err(|denial| match denial {
                    GroupStateDenial::StorageUnavailable(cause) => storage(cause),
                    _ => storage(StorageCause::Damaged),
                })?;
            let groups = state
                .read(basis.scope())
                .scheme(scheme.id().as_str())
                .ok_or(LookupDenial::MemberUnavailable)?;
            let identity = groups
                .owner_of(&key)
                .ok_or(LookupDenial::MemberUnavailable)?;
            let group = groups
                .group(&identity)
                .ok_or(LookupDenial::IdentityUnavailable)?;
            if !group
                .members
                .get(&key)
                .is_some_and(|actual| actual.carrier == *member)
            {
                return Err(LookupDenial::MemberUnavailable);
            }
            let history = names
                .aggregate_group_history(scheme.id().as_str(), identity)
                .ok_or(LookupDenial::IdentityUnavailable)?;
            Ok(NamedAggregateRead {
                identity,
                members: group.members,
                history,
            })
        })
    }

    fn with_named_group_basis<T>(
        &self,
        read: impl FnOnce(
            &BranchNamingBasisGuard<'_>,
            PersistentNameObservationView<'_>,
            CommitId,
        ) -> Result<T, LookupDenial>,
    ) -> Result<T, LookupDenial> {
        use worth_relational_lineage::internal_port::lineage::index::PersistentNameIndexDenial;

        if self.applied_changes.is_some() {
            return Err(LookupDenial::PublicationRequired);
        }
        let Some(commit) = self.pinned_basis().commit_id() else {
            return Err(LookupDenial::IdentityUnavailable);
        };
        let binding = self
            .aggregate_naming
            .as_ref()
            .ok_or_else(|| storage(StorageCause::SourceUnavailable))?;
        let basis = binding.read_guard(self.pinned_basis().branch_id().clone());
        let index = basis.name_index().map_err(|denial| match denial {
            PersistentNameIndexDenial::StorageUnavailable(cause) => storage(*cause),
            _ => storage(StorageCause::SourceUnavailable),
        })?;
        let result = read(
            &basis,
            PersistentNameObservationView::new(index, basis.scope(), commit),
            commit,
        );
        basis.scope().settle().map_err(storage)?;
        if basis.scope().pruned().is_some() {
            return Err(Cause::HistoricalVisibility(contracts::visibility::snapshot_states::historical_denial::HistoricalVisibilityDenial::RetentionUnavailable).into());
        }
        result
    }
}

fn storage(cause: StorageCause) -> Cause {
    Cause::StorageUnavailable(RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Lineage {
            first: LineageId(0),
            last: LineageId(0),
        },
        cause,
    })
}
