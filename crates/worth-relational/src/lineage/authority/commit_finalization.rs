use crate::history::data::RelationalCommitReceipt;
use crate::lineage::authority::LineagePreparationAuthority;
use crate::lineage::data::{
    CommitNamingPlan, LineageCommitPostureInput, LineageNameDenial, LineagePostureClaim,
    LineageRecordTarget, PendingBirthTarget, PersistentNameBirthRequest, PersistentNameHolder,
    PreparedLineageFinalization,
};
use crate::runtime::WorkingState;
use crate::transactions::data::{
    CommitCreatedEntityBindings, CommitCreatedRelationBindings, MutationIntent, RecordRef,
    TransactionCommitError,
};

impl<'runtime> LineagePreparationAuthority<'runtime> {
    pub(crate) fn ensure_lineage_for_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        staged: &mut WorkingState,
        commit: &RelationalCommitReceipt,
        merged_plan: &[MutationIntent],
        changed_records: &[RecordRef],
        created_entities: &CommitCreatedEntityBindings,
        created_relations: &CommitCreatedRelationBindings,
        naming_plan: Option<&CommitNamingPlan>,
    ) -> Result<PreparedLineageFinalization, TransactionCommitError> {
        let Some(plan) = naming_plan else {
            return self.ensure_lineage_for_commit_with_postures(
                execution,
                staged,
                commit,
                merged_plan,
                changed_records,
                &LineageCommitPostureInput::default(),
            );
        };
        let input = resolve_commit_naming_plan(plan, created_entities, created_relations)
            .map_err(TransactionCommitError::lineage_name_denial)?;
        self.ensure_lineage_for_commit_with_postures(
            execution,
            staged,
            commit,
            merged_plan,
            changed_records,
            &input,
        )
    }
}

fn resolve_commit_naming_plan(
    plan: &CommitNamingPlan,
    created_entities: &CommitCreatedEntityBindings,
    created_relations: &CommitCreatedRelationBindings,
) -> Result<LineageCommitPostureInput, LineageNameDenial> {
    let mut postures = Vec::with_capacity(plan.pending_births.len() + plan.postures.len());
    for (ordinal, pending) in plan.pending_births.iter().enumerate() {
        let target = match &pending.target {
            PendingBirthTarget::CreatedEntity(created_ref) => {
                let Some(entity_id) = created_entities.resolve(created_ref) else {
                    return Err(LineageNameDenial::BirthWithoutCreate {
                        scheme_id: pending.scheme_id.clone(),
                        occurrence_key: pending.occurrence_key.clone(),
                        role_path: pending.role_path.clone(),
                        holder: PersistentNameHolder::PendingBirth {
                            occurrence_key: pending.occurrence_key.clone(),
                            role_path: pending.role_path.clone(),
                            ordinal,
                        },
                    });
                };
                LineageRecordTarget::Entity(entity_id)
            }
            PendingBirthTarget::CreatedRelation(created_ref) => {
                let Some(relation_id) = created_relations.resolve(created_ref) else {
                    return Err(LineageNameDenial::BirthWithoutCreate {
                        scheme_id: pending.scheme_id.clone(),
                        occurrence_key: pending.occurrence_key.clone(),
                        role_path: pending.role_path.clone(),
                        holder: PersistentNameHolder::PendingBirth {
                            occurrence_key: pending.occurrence_key.clone(),
                            role_path: pending.role_path.clone(),
                            ordinal,
                        },
                    });
                };
                LineageRecordTarget::Relation(relation_id)
            }
        };
        postures.push(LineagePostureClaim::Create {
            target,
            birth: PersistentNameBirthRequest {
                scheme_id: pending.scheme_id.clone(),
                occurrence_key: pending.occurrence_key.clone(),
                role_path: pending.role_path.clone(),
                parent_identities: pending.parent_identities.clone(),
                referenced_names: pending.referenced_names.clone(),
                canonical_discriminator: pending.canonical_discriminator.clone(),
                holder: target.to_holder(),
            },
        });
    }
    postures.extend(plan.postures.iter().cloned());
    Ok(LineageCommitPostureInput {
        scheme_registry: plan.scheme_registry.clone(),
        live_holders: Default::default(),
        live_lineages: Default::default(),
        retired_lineages: Default::default(),
        postures,
    })
}
