//! Bind publication pins to index-owned lookup operands.
use super::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead};
use contracts::identity::data::primitives::KindId;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_relational_branch::internal_port::indexes::data::bounded_entity_field_lookup::BoundedEntityFieldLookupDenialOwnerPort as _;
use worth_relational_branch::internal_port::indexes::data::*;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_indexes::internal_port::indexes::access::prepared_entity_field_lookup::PreparedEntityFieldLookup;
use worth_relational_indexes::internal_port::indexes::access::selected_index::PinnedIndexLookup;
use worth_relational_indexes::internal_port::indexes::access::{
    applied_entity_field_lookup, bounded_entity_field_lookup,
    bounded_related_entity_ordered_lookup, bounded_relation_join_lookup,
    pinned_entity_field_lookup, related_entity_prefix,
};

impl RelationalPinnedRead {
    fn index_lookup(&self) -> PinnedIndexLookup<'_> {
        PinnedIndexLookup {
            snapshot: self.snapshot(),
            resident: self.resident_basis(),
            runtime_identity: self.runtime_instance_id(),
            current_version: self.pinned_current_version(),
            instrumentation: self.pinned_instrumentation(),
            indexes: &self.indexes,
        }
    }
    /// Execute the snapshot-input lane against retained definitions and generations.
    /// Its established liveness/accounting policy is unchanged.
    pub fn execute_bounded_entity_field_lookup(
        &self,
        request: BoundedEntityFieldLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BoundedEntityFieldLookupOutcome, BoundedEntityFieldLookupDenial> {
        contracts::execution::request_custody::check_request(execution).map_err(|cause| {
            BoundedEntityFieldLookupDenial::new(
                BoundedEntityFieldLookupDenialKind::Execution(cause),
                request.index_id(),
            )
        })?;
        self.instrumentation.count_query_index_attempt();
        self.lookup_pinned_entity_field(request, parity_mode, execution)
    }
    pub(crate) fn lookup_pinned_entity_field(
        &self,
        request: BoundedEntityFieldLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BoundedEntityFieldLookupOutcome, BoundedEntityFieldLookupDenial> {
        bounded_entity_field_lookup::lookup(self.index_lookup(), request, parity_mode, execution)
    }
}
impl<Snapshot> RelationalPinnedRead<Snapshot> {
    /// Prepare an installed field lookup from this capability's selected handles.
    /// A cold index stays cold even if its owner later builds a generation.
    pub fn prepare_entity_field_lookup(
        &self,
        index: DerivedIndexId,
        kind: KindId,
        locator: &AspectFieldLocator,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<PreparedEntityFieldLookup, BoundedEntityFieldLookupDenial> {
        pinned_entity_field_lookup::prepare(
            self.runtime_instance_id(),
            self.resident_basis(),
            self.pinned_instrumentation(),
            &self.indexes,
            index,
            kind,
            locator,
            execution,
        )
    }
}

impl RelationalPinnedRead {
    /// Traverse the ordered generation selected when this capability was pinned.
    pub fn execute_bounded_related_entity_ordered_lookup(
        &self,
        request: BoundedRelatedEntityOrderedLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BoundedRelatedEntityOrderedLookupOutcome, BoundedRelatedEntityOrderedLookupDenial>
    {
        self.instrumentation.count_query_index_attempt();
        execution.check_live().map_err(|stop| {
            bounded_related_entity_ordered_lookup::denial(
                BoundedRelatedEntityOrderedLookupDenialKind::Execution(
                    contracts::execution::denial::kernel_stop(stop),
                ),
                &request,
            )
        })?;
        self.lookup_pinned_related_ordering(request, parity_mode, execution)
    }
    pub(crate) fn lookup_pinned_related_ordering(
        &self,
        request: BoundedRelatedEntityOrderedLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BoundedRelatedEntityOrderedLookupOutcome, BoundedRelatedEntityOrderedLookupDenial>
    {
        bounded_related_entity_ordered_lookup::lookup(
            self.index_lookup(),
            request,
            parity_mode,
            execution,
        )
    }
    /// Verify the relation join selected at pin time without an installed owner.
    pub fn execute_bounded_relation_join_lookup(
        &self,
        request: BoundedRelationJoinLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BoundedRelationJoinLookupOutcome, BoundedRelationJoinLookupDenial> {
        self.instrumentation.count_query_index_attempt();
        self.lookup_pinned_relation_join(request, parity_mode, execution)
    }
    pub(crate) fn lookup_pinned_relation_join(
        &self,
        request: BoundedRelationJoinLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BoundedRelationJoinLookupOutcome, BoundedRelationJoinLookupDenial> {
        bounded_relation_join_lookup::lookup(self.index_lookup(), request, parity_mode, execution)
    }
    /// Seeks the leading-value range and returns one complete funded page.
    pub fn execute_bounded_related_entity_prefix_lookup(
        &self,
        request: BoundedRelatedEntityPrefixLookupRequest<'_>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BoundedRelatedEntityPrefixPage, BoundedRelatedEntityPrefixDenial> {
        related_entity_prefix::lookup(self.index_lookup(), request, execution)
    }
}

impl RelationalPinnedRead<RelationalAppliedCandidateReadIdentity> {
    pub fn lookup_applied_entity_field(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        predecessor: &PreparedEntityFieldLookup,
        expected: &AspectValue,
        parity: BoundedIndexParityMode,
        candidate_limit: usize,
    ) -> Result<
        applied_entity_field_lookup::AppliedEntityFieldLookupOutcome,
        applied_entity_field_lookup::AppliedEntityFieldLookupDenial,
    > {
        let read = applied_entity_field_lookup::AppliedIndexRead {
            runtime_identity: self.runtime_instance_id(),
            before_root: self.applied_before_root(),
            current_version: self.pinned_current_version(),
            resident: self.resident_basis(),
            fields: self.applied_field_image(),
            instrumentation: self.pinned_instrumentation(),
            qualification: self.candidate_identity().qualification(),
        };
        applied_entity_field_lookup::lookup(
            &read,
            execution,
            predecessor,
            expected,
            parity,
            candidate_limit,
        )
    }
}

#[doc(hidden)]
pub trait RelationalPinnedReadOwnerPort: Sized {
    fn lookup_pinned_entity_field(
        &self,
        request: BoundedEntityFieldLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BoundedEntityFieldLookupOutcome, BoundedEntityFieldLookupDenial>;

    fn lookup_pinned_related_ordering(
        &self,
        request: BoundedRelatedEntityOrderedLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BoundedRelatedEntityOrderedLookupOutcome, BoundedRelatedEntityOrderedLookupDenial>;

    fn lookup_pinned_relation_join(
        &self,
        request: BoundedRelationJoinLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BoundedRelationJoinLookupOutcome, BoundedRelationJoinLookupDenial>;
}

impl RelationalPinnedReadOwnerPort for RelationalPinnedRead {
    fn lookup_pinned_entity_field(
        &self,
        request: BoundedEntityFieldLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BoundedEntityFieldLookupOutcome, BoundedEntityFieldLookupDenial> {
        self.lookup_pinned_entity_field(request, parity_mode, execution)
    }

    fn lookup_pinned_related_ordering(
        &self,
        request: BoundedRelatedEntityOrderedLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BoundedRelatedEntityOrderedLookupOutcome, BoundedRelatedEntityOrderedLookupDenial>
    {
        self.lookup_pinned_related_ordering(request, parity_mode, execution)
    }

    fn lookup_pinned_relation_join(
        &self,
        request: BoundedRelationJoinLookupRequest,
        parity_mode: BoundedIndexParityMode,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BoundedRelationJoinLookupOutcome, BoundedRelationJoinLookupDenial> {
        self.lookup_pinned_relation_join(request, parity_mode, execution)
    }
}
