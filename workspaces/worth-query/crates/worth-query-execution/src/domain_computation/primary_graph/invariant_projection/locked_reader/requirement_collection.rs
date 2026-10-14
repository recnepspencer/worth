//! Bounded distinct live endpoints; decision readers seal their liveness facts.
use super::{
    ApplicationSchema, WorthQueryApplicationInvariantProjectionReader,
    WorthQueryInvariantEntityIdentity,
};
use crate::domain_computation::primary_graph::{
    application_contribution::{advancement_denial, QueryMemoryReservation, QueryRequestExecution},
    HandlerExecutionDenial, WorthQueryAdvancementDenial, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryManagedComputationResourceDenial,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyRequirementDirection as Direction,
    ApplicationOperationBodyRequirementSource as Source,
};
use worth_relational::facade::identity::EntityId;

pub(in crate::domain_computation::primary_graph::invariant_projection) struct ObservedRequirementItems
{
    pub(in crate::domain_computation::primary_graph::invariant_projection) ids: Vec<EntityId>,
    pub(in crate::domain_computation::primary_graph::invariant_projection) reservation:
        QueryMemoryReservation,
}

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionReader<'_, Schema> {
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn body_related_items<
        Entity,
    >(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        source: &Source,
        execution: &QueryRequestExecution<'_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<ObservedRequirementItems, HandlerExecutionDenial> {
        let Source::RelatedItemCount {
            relation,
            from,
            to,
            direction,
        } = source
        else {
            unreachable!("collection observer accepts collection source")
        };
        let missing = || {
            HandlerExecutionDenial::new(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                relation,
            ))
        };
        let layout = self
            .layout
            .relation(relation)
            .cloned()
            .ok_or_else(missing)?;
        let (anchor_kind, endpoint_kind, endpoint_name) = match direction {
            Direction::Outgoing => (layout.from, layout.to, to.as_str()),
            Direction::Incoming => (layout.to, layout.from, from.as_str()),
        };
        if !self.identity_is_local(identity, source.scope_entity()) || identity.kind != anchor_kind
        {
            return Err(missing());
        }
        if !self.work_budget.can_afford(1) {
            return Err(HandlerExecutionDenial::new(
                WorthQueryAdvancementDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::WorkExhausted,
                ),
            ));
        }
        self.work_budget.consume(1);
        self.work.record_endpoint();
        self.endpoint::<Entity>(
            self.execution,
            identity.entity_id,
            anchor_kind,
            source.scope_entity(),
        )
        .map_err(HandlerExecutionDenial::new)?;
        self.realized_scope.record(identity.entity_id);
        self.admit_adjacency_list(relation)
            .map_err(HandlerExecutionDenial::new)?;
        let view = self
            .branch_view(relation)
            .map_err(HandlerExecutionDenial::new)?;
        let read = match direction {
            Direction::Outgoing => view.bounded_outgoing_relations_of_kind_admitted(
                self.execution,
                identity.entity_id,
                layout.kind,
                self.work_budget.remaining(),
                |_| Ok(()),
            ),
            Direction::Incoming => view.bounded_incoming_relations_of_kind_admitted(
                self.execution,
                identity.entity_id,
                layout.kind,
                self.work_budget.remaining(),
                |_| Ok(()),
            ),
        }
        .map_err(|cause| {
            HandlerExecutionDenial::new(WorthQueryApplicationAttemptDenial::from(cause))
        })?
        .map_err(|limit| {
            HandlerExecutionDenial::new(self.adjacency_limit_denial(limit, relation))
        })?;
        self.work_budget.consume(read.work_units());
        self.work.record_adjacency(
            read.relation_records_examined(),
            read.endpoint_records_reserved(),
        );
        let records = read.into_records();
        let bytes = records
            .len()
            .checked_mul(size_of::<EntityId>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(overflow)?;
        let reservation = execution.reserve(bytes).map_err(|cause| {
            HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
        })?;
        let sorting = u64::try_from(records.len())
            .ok()
            .and_then(|count| {
                count.checked_mul(u64::from(usize::BITS - records.len().leading_zeros()) + 1)
            })
            .ok_or_else(overflow)?;
        context.checkpoint(sorting).map_err(stopped)?;
        let mut ids = Vec::with_capacity(records.len());
        for record in records {
            let endpoint = match direction {
                Direction::Outgoing => record.target,
                Direction::Incoming => record.source,
            };
            self.endpoint::<()>(self.execution, endpoint, endpoint_kind, endpoint_name)
                .map_err(HandlerExecutionDenial::new)?;
            self.realized_scope.record(endpoint);
            ids.push(endpoint);
        }
        ids.sort_unstable();
        ids.dedup();
        Ok(ObservedRequirementItems { ids, reservation })
    }
}

fn overflow() -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(
        WorthQueryManagedComputationResourceDenial::CapacityOverflow,
    ))
}
fn stopped(cause: worth_execution::MapKernelStop) -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(advancement_denial(
        worth_execution::WorkCeilingDenial::Stopped(cause),
    ))
}
