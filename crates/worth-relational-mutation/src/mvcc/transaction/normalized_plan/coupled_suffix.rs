//! Resolve one writer's normalized suffix against the actual applied prefix bindings.
use super::*;
use crate::authority::mutation::intents::{
    field_authoring_candidate::{self, FieldAuthoringDomain},
    record_aspect_patch::MutationPatchDenial,
};
use crate::contracts::transactions::data::intents::mutation_intent::{
    ApplyEntityAspectPatchIntent, CreateIntent, EntityMutationIntent,
};
use crate::contracts::transactions::data::outcomes::conflict_class::record_aspect_patch_denial::RecordAspectPatchTarget;
use crate::contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use crate::contracts::transactions::data::primitives::{CreatedEntityRef, EntityReference};
use worth_foundational::facade::PortablePatchReadmissionPurpose;
impl OwnedNormalizedPlan {
    /// A successor round re-creates its chain's records under the same keys.
    /// Its applied plan writes them in place on the chain's allocations, so no
    /// second allocation exists. Fieldless re-creates are omitted as no-ops;
    /// populated re-creates lower under `RecordCreation` semantics (`SetWhole`)
    /// into `ApplyAspectPatch` so newly populated optional struct aspects apply.
    /// The recorded plan and transaction batches keep the unlowered Create.
    pub fn lower_coupled_recreates(
        mut self,
        bindings: &CommitCreatedEntityBindings,
        plans: &crate::contracts::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, TransactionCommitError> {
        let recreates = request
            .run_native_with_checkpoint(|context| {
                use crate::contracts::execution::request_stage::checkpoint;
                let mut found = Vec::new();
                for (index, intent) in self.plan.merged_intents.iter().enumerate() {
                    checkpoint(context, 1).map_err(stop)?;
                    let MutationIntent::Create(CreateIntent::Entity(spec)) = intent else {
                        continue;
                    };
                    let created = CreatedEntityRef {
                        partition_id: spec.partition_id,
                        kind_id: spec.kind_id,
                        client_key: spec.client_key.clone(),
                    };
                    checkpoint(
                        context,
                        bindings
                            .resolve_work_bound(&created)
                            .ok_or_else(copy_overflow)?,
                    )
                    .map_err(stop)?;
                    if let Some(entity_id) = bindings.resolve(&created) {
                        found.push((index, entity_id));
                    }
                }
                Ok::<_, TransactionCommitError>(found)
            })
            .map_err(request_denial)?
            .0?;
        let relinks = self
            .plan
            .merged_intents
            .iter()
            .any(|intent| matches!(intent, MutationIntent::Create(CreateIntent::Relation(_))));
        if recreates.is_empty() && !relinks {
            return Ok(self);
        }
        let mut kept = Vec::with_capacity(self.plan.merged_intents.len());
        let mut lower_funding = Vec::new();
        let mut next = recreates.into_iter().peekable();
        for (index, intent) in std::mem::take(&mut self.plan.merged_intents)
            .into_iter()
            .enumerate()
        {
            // A successor's relation creates were proved re-creates of its
            // chain's relations (`prove_succession`), which the head applied.
            if matches!(intent, MutationIntent::Create(CreateIntent::Relation(_))) {
                continue;
            }
            let Some(&(_, entity_id)) = next.peek().filter(|&&(idx, _)| idx == index) else {
                kept.push(intent);
                continue;
            };
            next.next();
            let MutationIntent::Create(CreateIntent::Entity(spec)) = intent else {
                unreachable!("recreate index matched a Create(Entity) intent");
            };
            if spec.fields.is_empty() {
                continue;
            }
            let candidate = field_authoring_candidate::lower(
                request,
                &spec.fields,
                PortablePatchReadmissionPurpose::RecordCreation,
                plans.entity_plans.get(&spec.kind_id),
                spec.kind_id,
                FieldAuthoringDomain::Entity,
            )
            .map_err(|denial| {
                MutationPatchDenial::candidate(
                    RecordAspectPatchTarget::EntityCreation {
                        kind_id: spec.kind_id,
                    },
                    denial,
                )
            })?;
            let (aspect_patch, funding) = candidate.into_portable();
            if let Some(funding) = funding {
                lower_funding.push(funding);
            }
            kept.push(MutationIntent::Entity(
                EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                    entity_id,
                    aspect_patch,
                }),
            ));
        }
        self.plan.merged_intents = kept;
        if !lower_funding.is_empty() {
            self._memory = reserve_copy(request, intent_bytes(&self.plan.merged_intents))?;
        }
        drop(lower_funding);
        Ok(self)
    }

    pub fn into_coupled_suffix(
        mut self,
        prefix: usize,
        bindings: &CommitCreatedEntityBindings,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, TransactionCommitError> {
        if prefix > self.plan.merged_intents.len() {
            return Err(
                crate::contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "coupled suffix starts beyond its normalized source",
                ),
            );
        }
        request
            .run_native_with_checkpoint(|context| {
                use crate::contracts::execution::request_stage::checkpoint;
                let copy = self
                    .plan
                    .merged_intents
                    .len()
                    .checked_sub(prefix)
                    .and_then(|count| count.checked_mul(std::mem::size_of::<MutationIntent>()))
                    .and_then(|bytes| u64::try_from(bytes).ok())
                    .ok_or_else(copy_overflow)?;
                checkpoint(context, copy).map_err(stop)?;
                for intent in &self.plan.merged_intents[..prefix] {
                    let units = intent
                        .copy_storage_bytes()
                        .and_then(|bytes| u64::try_from(bytes).ok())
                        .and_then(|bytes| bytes.checked_add(1))
                        .ok_or_else(copy_overflow)?;
                    checkpoint(context, units).map_err(stop)?;
                }
                self.plan.merged_intents.drain(..prefix);
                for intent in &mut self.plan.merged_intents {
                    checkpoint(context, 1).map_err(stop)?;
                    intent.visit_entity_references_mut(|reference| {
                        if let EntityReference::Created(created) = reference {
                            checkpoint(
                                context,
                                bindings
                                    .resolve_work_bound(created)
                                    .ok_or_else(copy_overflow)?,
                            )
                            .map_err(stop)?;
                            if let Some(id) = bindings.resolve(created) {
                                *reference = EntityReference::Existing(id);
                            }
                        }
                        Ok::<_, TransactionCommitError>(())
                    })?;
                }
                Ok::<_, TransactionCommitError>(())
            })
            .map_err(request_denial)?
            .0?;
        // Drain retains the source vector capacity; its original reservation stays
        // with this owner through sorting, candidate retention and final completion.
        Ok(self)
    }
}
fn stop(
    cause: impl Into<crate::contracts::execution::denial_cause::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
        kind: crate::contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
