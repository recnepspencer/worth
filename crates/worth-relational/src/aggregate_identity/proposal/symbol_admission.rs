//! Admit consequence names inside the same abandonable symbol-discovery round.
use super::{binding_error, AggregateProposalBorrow, ArtifactState};
use crate::authority::mutation::MutationApplyDenial;
use crate::schema::data::AspectBinding;
use crate::storage::overlay::OverlayStateView;
use crate::transactions::data::{
    CreateIntent, EntityMutationIntent, MutationIntent, RelationMutationIntent,
};
use crate::validation::engine::state_view::InvariantStateView;
use std::collections::BTreeSet;

impl AggregateProposalBorrow<'_> {
    pub(crate) fn reserve_consequence_symbols(
        self,
        symbols: &mut crate::runtime::PendingSymbolMeanings,
        policy: crate::symbols::data::ClientKeySymbolPolicy,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), MutationApplyDenial> {
        let state = self
            .owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let ArtifactState::Ready(artifact) = &*state else {
            return Ok(());
        };
        let Some(consequences) = &artifact.consequences else {
            return Ok(());
        };
        let before = InvariantStateView::new(&self.selected, self.selected.version_id());
        let overlay = OverlayStateView::new(&self.selected, self.after);
        let after = InvariantStateView::new(&overlay, self.owner.identity.proposed_version_id());
        let mut kinds = BTreeSet::new();
        let mut raw = BTreeSet::new();
        for intent in &consequences.apply.merged_intents {
            if policy.interns_requested_strings() {
                intent
                    .collect_raw_client_keys(&mut raw, allocation)
                    .map_err(|denial| {
                        crate::transactions::data::TransactionCommitError::conflict(
                            crate::mvcc::RelationalTransactionStagingDenial::from(denial)
                                .into_conflict(),
                        )
                    })?;
            }
            let (entity, kind) = match intent {
                MutationIntent::Create(CreateIntent::Entity(spec)) => (true, Some(spec.kind_id)),
                MutationIntent::Create(CreateIntent::Relation(spec)) => (false, Some(spec.kind_id)),
                MutationIntent::Entity(EntityMutationIntent::UpdateFields(spec)) => (
                    true,
                    after
                        .entity_metadata(spec.entity_id)
                        .or_else(|| before.entity_metadata(spec.entity_id))
                        .map(|m| m.kind_id),
                ),
                MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(spec)) => (
                    false,
                    after
                        .relation_metadata(spec.relation_id)
                        .or_else(|| before.relation_metadata(spec.relation_id))
                        .map(|m| m.kind_id),
                ),
                MutationIntent::Relation(RelationMutationIntent::Delete(spec)) => (
                    false,
                    before
                        .relation_metadata(spec.relation_id)
                        .map(|m| m.kind_id),
                ),
                _ => return Err(binding_error("unsupported aggregate consequence intent").into()),
            };
            kinds.insert((
                entity,
                kind.ok_or_else(|| binding_error("aggregate consequence target unavailable"))?,
            ));
        }
        for text in raw {
            symbols.reserve(&text)?;
        }
        for (entity, kind) in kinds {
            let plans = self.owner.authority.aspect_plans();
            let plan = if entity {
                plans.entity_plans.get(&kind)
            } else {
                plans.relation_plans.get(&kind)
            };
            let plan =
                plan.ok_or_else(|| binding_error("aggregate consequence aspect plan unavailable"))?;
            for binding in &plan.executable_bindings {
                symbols.reserve(binding.aspect_key().as_str())?;
                if let worth_foundational::AspectShape::Struct(shape) = binding.contract.shape() {
                    for field in shape.fields() {
                        symbols.reserve(field.key().as_str())?;
                    }
                }
                match &binding.target {
                    AspectBinding::EntityField { field }
                    | AspectBinding::RelationField { field } => {
                        symbols.reserve(field.as_str())?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
