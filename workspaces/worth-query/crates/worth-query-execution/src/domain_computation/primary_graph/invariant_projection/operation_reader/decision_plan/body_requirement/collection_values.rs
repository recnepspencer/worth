//! Native collection membership and scalar observations share one retained decision.
use super::*;

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub(super) fn collection_field_values<Entity>(
        &mut self,
        root: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        source: &Source,
        execution: &QueryRequestExecution<'_>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<PreparedRequirementOperand, HandlerExecutionDenial> {
        let Source::RelatedFieldValues {
            relation,
            from,
            to,
            field,
            ..
        } = source
        else {
            unreachable!("collection field selector");
        };
        let selection = Source::RelatedItemCount {
            relation: relation.clone(),
            from: from.clone(),
            to: to.clone(),
            direction: Direction::Outgoing,
        };
        let mut observed =
            self.observed_requirement_collection(root, &selection, execution, context)?;
        let kind = self.reader.layout.entity_kind(to).ok_or_else(|| HandlerExecutionDenial::new(
            crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(
                crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact, to,
            )))?;
        let list_bytes = observed
            .ids
            .len()
            .checked_mul(size_of::<ExpressionValue>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(overflow)?;
        observed.reservation.grow(list_bytes).map_err(resource)?;
        let mut values = Vec::with_capacity(observed.ids.len());
        // O(distinct endpoints + their field representations), after the shared
        // adjacency owner's bounded deterministic deduplication.
        for entity_id in observed.ids {
            let child = WorthQueryInvariantEntityIdentity::<Schema, ()> {
                entity_id,
                kind,
                entity: to.clone().into(),
                authority_identity: self.reader.authority_identity,
                _marker: std::marker::PhantomData,
            };
            let locator = self
                .tracked_body_locator(&child, field)
                .map_err(HandlerExecutionDenial::new)?;
            let value = self.reader.body_scalar_observe(&child, &locator, |raw| {
                let bytes = u64::try_from(raw.map_or(0, |raw| raw.semantic_byte_width()))
                    .map_err(|_| overflow())?;
                context
                    .checkpoint(bytes.saturating_add(1))
                    .map_err(stopped)?;
                observed.reservation.grow(bytes).map_err(resource)?;
                field_value(source, raw)
            })?;
            values.push(value);
        }
        Ok(PreparedRequirementOperand {
            value: ExpressionValue::list(values),
            _reservation: observed.reservation,
        })
    }
}

fn resource(cause: WorthQueryManagedComputationResourceDenial) -> HandlerExecutionDenial {
    HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
}
