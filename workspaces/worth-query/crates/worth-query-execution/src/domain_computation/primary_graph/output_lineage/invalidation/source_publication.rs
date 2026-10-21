//! Source publication maintains exact marks in the native selected branch image.
use super::delivery;
use super::source_alignment::HistoricalMarkState;
use super::source_alignment::RetainedTouchDelivery;
use super::SourceInvalidationOwner;
use crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership;
use std::sync::Arc;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::mvcc::PreparedPublicationCompanionEffect;
use worth_relational::facade::mvcc::PublicationCompanionPreflight;
use worth_relational::facade::mvcc::RelationalPublicationCompanion;
impl RelationalPublicationCompanion for SourceInvalidationOwner {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        if context.runtime_instance_id() != self.runtime_instance_id {
            return Err(CompanionPreflightStop::ForeignCell);
        }
        context.checkpoint()?;
        let cell = self.selected_cell(context)?;
        let reserved = cell.reserve_preflight(context)?;
        let observed = reserved.current();
        let keys = delivery::selectors(context)?;
        let commit = context.canonical_commit().commit.commit_id;
        let delivery::MarkedDelivery {
            state,
            report,
            affected,
            selected,
            keys,
        } = delivery::mark(&observed.payload().current, keys, commit, context)?;
        super::delivery_observation::record_delivery(&report);
        super::marking_observation::record(&report, affected.as_ref());
        let mut root = (**observed.payload()).clone();
        root.past.insert(
            observed.position(),
            HistoricalMarkState {
                root_id: observed.root_id(),
                commit_id: observed.commit_id(),
                state: Arc::clone(&observed.payload().current),
                next_delivery: Arc::new(RetainedTouchDelivery { keys }),
            },
        );
        root.current = Arc::new(state);
        root.last_native_marking = Some(report);
        context.checkpoint()?;
        let mut effect = context.seal_replacement(reserved, Arc::new(root))?;
        if !selected.is_empty() {
            let observer = effect.attach_completion_observer(context)?;
            let branch =
                RequiredWorkMembership::prepared_native_branch(context.branch_id().clone());
            let mut prepared = Vec::with_capacity(selected.len());
            for membership in selected {
                context.checkpoint()?;
                prepared.push((
                    membership,
                    RequiredWorkMembership::prepared_native_hint(
                        observer.clone(),
                        Arc::clone(&branch),
                    ),
                ));
            }
            for (membership, hint) in prepared {
                membership.prepare_hint(hint);
            }
        }
        Ok(effect)
    }
}
