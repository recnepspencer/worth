use crate::history::data::RelationalCommitReceipt;
use crate::lineage::authority::LineagePreparationAuthority;
use crate::lineage::data::{
    CommitNamingPlan, LineageMergeKeeper, LineageNameDenial, LineagePostureClaim,
    LineageRecordTarget, LineageSplitChild, PendingBirthTarget, PendingMergeKeeper,
    PendingMergeTarget, PersistentNameBirthRequest, PersistentNameHolder,
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
        graph_selection: Option<&crate::history::data::CanonicalOwnedGraphSelection>,
    ) -> Result<PreparedLineageFinalization, TransactionCommitError> {
        let postures = match naming_plan {
            Some(plan) => resolve_commit_naming_plan(plan, created_entities, created_relations)
                .map_err(TransactionCommitError::lineage_name_denial)?,
            None => Vec::new(),
        };
        self.finalize_commit_with_installed_schema(
            execution,
            staged,
            commit,
            merged_plan,
            changed_records,
            &postures,
            graph_selection,
        )
    }
}

fn resolve_pending_target(
    pending: &PendingBirthTarget,
    scheme_id: &str,
    occurrence_key: &str,
    role_path: &str,
    ordinal: usize,
    created_entities: &CommitCreatedEntityBindings,
    created_relations: &CommitCreatedRelationBindings,
) -> Result<LineageRecordTarget, LineageNameDenial> {
    match pending {
        PendingBirthTarget::CreatedEntity(created_ref) => created_entities
            .resolve(created_ref)
            .map(LineageRecordTarget::Entity)
            .ok_or_else(|| LineageNameDenial::BirthWithoutCreate {
                scheme_id: scheme_id.to_owned(),
                occurrence_key: occurrence_key.to_owned(),
                role_path: role_path.to_owned(),
                holder: PersistentNameHolder::PendingBirth {
                    occurrence_key: occurrence_key.to_owned(),
                    role_path: role_path.to_owned(),
                    ordinal,
                },
            }),
        PendingBirthTarget::CreatedRelation(created_ref) => created_relations
            .resolve(created_ref)
            .map(LineageRecordTarget::Relation)
            .ok_or_else(|| LineageNameDenial::BirthWithoutCreate {
                scheme_id: scheme_id.to_owned(),
                occurrence_key: occurrence_key.to_owned(),
                role_path: role_path.to_owned(),
                holder: PersistentNameHolder::PendingBirth {
                    occurrence_key: occurrence_key.to_owned(),
                    role_path: role_path.to_owned(),
                    ordinal,
                },
            }),
    }
}

fn resolve_commit_naming_plan(
    plan: &CommitNamingPlan,
    created_entities: &CommitCreatedEntityBindings,
    created_relations: &CommitCreatedRelationBindings,
) -> Result<Vec<LineagePostureClaim>, LineageNameDenial> {
    let mut postures = Vec::with_capacity(
        plan.pending_births.len()
            + plan.pending_splits.len()
            + plan.pending_merges.len()
            + plan.postures.len(),
    );
    for (ordinal, pending) in plan.pending_births.iter().enumerate() {
        let target = resolve_pending_target(
            &pending.target,
            &pending.scheme_id,
            &pending.occurrence_key,
            &pending.role_path,
            ordinal,
            created_entities,
            created_relations,
        )?;
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
    for split in &plan.pending_splits {
        let mut children = Vec::with_capacity(split.children.len());
        for (ordinal, child) in split.children.iter().enumerate() {
            let target = resolve_pending_target(
                &child.target,
                &child.scheme_id,
                &child.occurrence_key,
                &child.role_path,
                ordinal,
                created_entities,
                created_relations,
            )?;
            children.push(LineageSplitChild {
                target,
                birth: PersistentNameBirthRequest {
                    scheme_id: child.scheme_id.clone(),
                    occurrence_key: child.occurrence_key.clone(),
                    role_path: child.role_path.clone(),
                    parent_identities: vec![split.parent_name],
                    referenced_names: child.referenced_names.clone(),
                    canonical_discriminator: child.canonical_discriminator.clone(),
                    holder: target.to_holder(),
                },
            });
        }
        postures.push(LineagePostureClaim::Split {
            parent_target: split.parent_target,
            parent_name: split.parent_name,
            children,
        });
    }
    for (ordinal, merge) in plan.pending_merges.iter().enumerate() {
        let target = match &merge.target {
            PendingMergeTarget::Existing(existing) => *existing,
            PendingMergeTarget::Created(pending) => {
                let (sid, occ, role) = match &merge.keeper {
                    PendingMergeKeeper::KeepParent(_) => ("", "", ""),
                    PendingMergeKeeper::MintNew {
                        scheme_id,
                        occurrence_key,
                        role_path,
                        ..
                    } => (scheme_id.as_str(), occurrence_key.as_str(), role_path.as_str()),
                };
                resolve_pending_target(
                    pending,
                    sid,
                    occ,
                    role,
                    ordinal,
                    created_entities,
                    created_relations,
                )?
            }
        };
        let keeper = match &merge.keeper {
            PendingMergeKeeper::KeepParent(id) => LineageMergeKeeper::KeepParent(*id),
            PendingMergeKeeper::MintNew {
                scheme_id,
                occurrence_key,
                role_path,
                referenced_names,
                canonical_discriminator,
            } => {
                let mut parent_ids: Vec<_> = merge.parents.iter().map(|(_, id)| *id).collect();
                parent_ids.sort_unstable();
                LineageMergeKeeper::MintNew {
                    birth: PersistentNameBirthRequest {
                        scheme_id: scheme_id.clone(),
                        occurrence_key: occurrence_key.clone(),
                        role_path: role_path.clone(),
                        parent_identities: parent_ids,
                        referenced_names: referenced_names.clone(),
                        canonical_discriminator: canonical_discriminator.clone(),
                        holder: target.to_holder(),
                    },
                }
            }
        };
        postures.push(LineagePostureClaim::Merge {
            parents: merge.parents.clone(),
            target,
            keeper,
        });
    }
    postures.extend(plan.postures.iter().cloned());
    Ok(postures)
}

#[cfg(test)]
impl<'runtime> LineagePreparationAuthority<'runtime> {
    /// Test-only production-authority entry verifying Relational ignores
    /// caller-supplied live/retired maps and derives them from its own
    /// `LineageSubsystem`.
    pub(crate) fn ensure_lineage_for_commit_with_postures(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        staged: &mut WorkingState,
        commit: &RelationalCommitReceipt,
        merged_plan: &[MutationIntent],
        changed_records: &[RecordRef],
        input: &crate::lineage::data::LineageCommitPostureInput,
    ) -> Result<PreparedLineageFinalization, TransactionCommitError> {
        let basis = self
            .runtime
            .lineage_naming_basis
            .read_guard(commit.branch_id.clone());
        let prepared = super::posture_finalization::finalize_postures_with_installer(
            &self.runtime.lineage_identity,
            staged,
            commit,
            merged_plan,
            changed_records,
            &input.scheme_registry,
            None,
            &basis,
            &input.postures,
            None,
            |staged, entity_id, lineage_id| {
                super::staged_effects::install_lineage_id(execution, staged, entity_id, lineage_id)
            },
        )?;
        drop(basis);
        self.runtime
            .performance_access()
            .count_lineage_finalization(
                prepared.artifact().event_batch().events().len(),
                prepared.artifact().decision_log().decisions().len(),
            );
        Ok(prepared)
    }
}

#[cfg(test)]
pub(crate) fn finalize_postures_in_working_state(
    allocator: &crate::runtime::LineageIdentityAllocator,
    staged: &mut WorkingState,
    commit: &RelationalCommitReceipt,
    merged_plan: &[MutationIntent],
    changed_records: &[RecordRef],
    input: &crate::lineage::data::LineageCommitPostureInput,
) -> Result<PreparedLineageFinalization, TransactionCommitError> {
    super::posture_finalization::finalize_postures_with_installer(
        allocator,
        staged,
        commit,
        merged_plan,
        changed_records,
        &input.scheme_registry,
        None,
        input,
        &input.postures,
        None,
        super::staged_effects::install_lineage_id_in_working_state,
    )
}

#[cfg(test)]
pub(crate) fn test_finalize_with_schema(
    allocator: &crate::runtime::LineageIdentityAllocator,
    staged: &mut WorkingState,
    commit: &RelationalCommitReceipt,
    changed_records: &[RecordRef],
    schema: &crate::schema::data::RelationalSchemaRegistry,
) -> Result<PreparedLineageFinalization, TransactionCommitError> {
    let empty = crate::lineage::data::LineageCommitPostureInput::default();
    super::posture_finalization::finalize_postures_with_installer(
        allocator,
        staged,
        commit,
        &[],
        changed_records,
        schema.persistent_name_schemes(),
        Some(schema),
        &empty,
        &[],
        None,
        super::staged_effects::install_lineage_id_in_working_state,
    )
}
