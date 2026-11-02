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
    fn prepare_managed_source_fork(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &worth_relational::facade::mvcc::ManagedIndexSourceForkPreparation<'_>,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, CompanionPreflightStop> {
        if target.runtime_instance_id() != self.runtime_instance_id {
            return Err(CompanionPreflightStop::ForeignCell);
        }
        self.managed_views.prepare_source_fork(execution, target)
    }
    fn prepare_managed_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &worth_relational::facade::runtime::ManagedIndexSourcePreparation<'_>,
    ) -> Result<
        Vec<worth_relational::facade::runtime::PreparedManagedIndexSourceRoot>,
        worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial,
    > {
        if source.selected_basis().identity().runtime_instance_id() != self.runtime_instance_id {
            return Err(worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial::ForeignSource);
        }
        self.managed_views.prepare_sources(execution, source)
    }
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
        let mut effect = if let Some(managed) = self.managed_views.reserve_source_image(context)? {
            let next = context.prepare_managed_source_image(&managed)?;
            context.seal_managed_source_replacement(reserved, Arc::new(root), managed, next)?
        } else {
            context.seal_replacement(reserved, Arc::new(root))?
        };
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
