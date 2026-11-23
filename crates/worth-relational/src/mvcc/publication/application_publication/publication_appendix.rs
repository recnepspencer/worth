//! Judge the producer body before a fresh publication-only record is staged.
use super::{
    RelationalInstalledApplicationPublicationAuthority, RelationalRequiredOutputPublication,
    RequiredOutputProducingImplementation, RequiredOutputRolePosture, RequiredOutputRoleTarget,
};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::intents::mutation_intent::CreateIntent;
use crate::transactions::data::intents::mutation_intent::EntityMutationIntent;
use crate::transactions::data::intents::mutation_intent::MutationIntent;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::EntityReference;
use crate::transactions::data::primitives::EntitySpec;
use crate::transactions::data::primitives::WorkerIntentBatch;

/// A publication appendix can allocate its own single entity. It cannot update
/// a producer record, acquire a relation, or carry an arbitrary effect batch.
/// The caller's receipt owner continues to own the meaning of the entity.
pub struct RelationalPublicationAppendix {
    entity: EntitySpec,
    position: usize,
}
impl RelationalPublicationAppendix {
    pub fn fresh_entity(entity: EntitySpec, position: usize) -> Self {
        Self { entity, position }
    }
}

/// Selectors for adopting a genuine current publication. Neither a role claim
/// nor an implementation descriptor establishes ownership by itself.
pub struct RelationalPublicationAdoptionRequest<'a> {
    pub producer: &'a str,
    pub installed: &'a RelationalInstalledApplicationPublicationAuthority,
    pub previous: &'a RelationalRequiredOutputPublication,
    pub implementation: &'a RequiredOutputProducingImplementation,
    pub roles: &'a [RequiredOutputRoleTarget],
}

pub struct RelationalPublicationBodySelection {
    batch: WorkerIntentBatch,
    adoption: Option<std::sync::Arc<crate::history::data::CanonicalRequiredOutputOrigin>>,
    basis: AdmittedRelationalBranchBasis,
}
impl RelationalPublicationBodySelection {
    pub(crate) fn into_parts(
        self,
    ) -> (
        WorkerIntentBatch,
        Option<std::sync::Arc<crate::history::data::CanonicalRequiredOutputOrigin>>,
        AdmittedRelationalBranchBasis,
    ) {
        (self.batch, self.adoption, self.basis)
    }

    pub fn omitted_appendix(&self) -> bool {
        self.adoption.is_some()
    }
}

/// The judgment allocates no record or symbol identities. A real publication
/// stages the original receipt in its original place and follows the normal
/// canonical plan, retaining that plan's allocation and byte ordering.
pub(crate) fn prepare_publication_appendix(
    runtime_instance_id: u64,
    dictionary: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    mut body: WorkerIntentBatch,
    content: &crate::content::RelationalContentAttachments,
    appendix: RelationalPublicationAppendix,
    adoption: Option<RelationalPublicationAdoptionRequest<'_>>,
) -> Result<RelationalPublicationBodySelection, TransactionCommitError> {
    if appendix.position > body.intents.len() {
        return Err(super::origin_denial(
            "publication appendix position is outside its body",
        ));
    }
    let prior = adoption
        .as_ref()
        .map(|request| std::sync::Arc::clone(&request.previous.origin));
    let selected =
        crate::branch::root_selection::SelectedRelationalBranchState::from_admitted_basis(basis);
    let omitted = match adoption {
        Some(adoption)
            if basis.is_current()
                && content.is_empty()
                && adoption.previous.runtime_instance_id == runtime_instance_id
                && adoption.previous.origin.producer == adoption.producer
                && adoption.previous.origin.meaning.implementation == *adoption.implementation
                && adoption.previous.matches_current_ownership_at(
                    execution,
                    adoption.installed,
                    basis,
                )? =>
        {
            crate::storage::residency::probe_head(
                execution, &selected.with_symbol_admission(dictionary), |partitions| {
                    let read = crate::branch::selected_read::SelectedRelationalBranchRead::in_round(&selected, partitions);
                    execution.run_native_with_checkpoint(|context| -> Result<bool, TransactionCommitError> {
                        if !matches_live_members(context, &read, &adoption, execution.allocation_policy())? { return Ok(false); }
                        let schema = basis.inner.root.retained_schema_authority();
                        let plans = schema.aspect_plans();
                        for intent in &body.intents {
                            crate::execution::request_stage::checkpoint(context, 1).map_err(super::custody::execution_stop)?;
                            match intent {
                                MutationIntent::Entity(EntityMutationIntent::UpdateFields(intent)) => {
                                    let prepared = crate::authority::mutation::intents::entity_aspect_preparation::prepare_fields(execution, intent, &read, plans)?;
                                    if prepared.changes_state(context)? { return Ok(false); }
                                }
                                MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(intent)) => {
                                    let prepared = crate::authority::mutation::intents::entity_aspect_preparation::prepare_portable(execution, intent, &read, plans)?;
                                    if prepared.changes_state(context)? { return Ok(false); }
                                }
                                // Revalidation has no mutation effect; actual validation still
                                // checks its full identity and installed invariants.
                                MutationIntent::Entity(EntityMutationIntent::Revalidate(_))
                                | MutationIntent::Relation(crate::transactions::data::intents::mutation_intent::RelationMutationIntent::Revalidate(_)) => {}
                                _ => return Ok(false),
                            }
                        }
                        Ok(true)
                    }).map_err(super::custody::execution_stop)?.0
                },
            ).map_err(|failure| crate::authority::commit::preparation::storage_read_denial(basis, failure))??
        }
        _ => false,
    };
    if !omitted {
        body.intents.insert(
            appendix.position,
            MutationIntent::Create(CreateIntent::Entity(appendix.entity)),
        );
    }
    Ok(RelationalPublicationBodySelection {
        batch: body,
        adoption: if omitted { prior } else { None },
        basis: basis.clone(),
    })
}

fn matches_live_members(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    read: &impl CurrentPartitionAccess,
    request: &RelationalPublicationAdoptionRequest<'_>,
    policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<bool, TransactionCommitError> {
    let members = &request.previous.origin.members;
    let mut seen = worth_execution::ExecutionArrayBuilder::allocate(members.len(), policy)
        .map_err(allocation_stop)?;
    let mut live = 0;
    for member in members {
        crate::execution::request_stage::checkpoint(context, 1)
            .map_err(super::custody::execution_stop)?;
        live += usize::from(member.posture != RequiredOutputRolePosture::Retire);
        seen.push(std::cell::Cell::new(false))
            .map_err(allocation_stop)?;
    }
    if request.roles.len() != live {
        return Ok(false);
    }
    let seen = seen.seal().map_err(allocation_stop)?;
    for role in request.roles {
        crate::execution::request_stage::checkpoint(
            context,
            (usize::BITS - members.len().leading_zeros()) as u64 + 1,
        )
        .map_err(super::custody::execution_stop)?;
        let EntityReference::Existing(id) = role.target else {
            return Ok(false);
        };
        let Ok(index) = members.binary_search_by_key(&id, |member| member.id) else {
            return Ok(false);
        };
        if seen[index].replace(true) {
            return Ok(false);
        }
        let old = &members[index];
        crate::execution::request_stage::checkpoint(
            context,
            role.role
                .len()
                .saturating_add(role.entity_name.len())
                .saturating_add(old.role.len())
                .saturating_add(old.entity_name.len())
                .saturating_add(1) as u64,
        )
        .map_err(super::custody::execution_stop)?;
        if old.posture == RequiredOutputRolePosture::Retire
            || role.posture != RequiredOutputRolePosture::Preserve
            || role.kind != old.kind
            || role.role != old.role
            || role.entity_name != old.entity_name
        {
            return Ok(false);
        }
        let Some(partition) = read.get_current_partition(id.partition_id) else {
            return Ok(false);
        };
        let Some(row) = partition.entity_arena().get(&id) else {
            return Ok(false);
        };
        if !row.is_live()
            || row.kind_id() != Some(old.kind)
            || partition
                .entity_arena()
                .current_metadata_at(id.slot_index())
                .map(|m| m.effective_at)
                != Some(old.after_write)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn allocation_stop(cause: worth_execution::ExecutionAllocationDenial) -> TransactionCommitError {
    TransactionCommitError::conflict(
        crate::mvcc::transaction::staging::RelationalTransactionStagingDenial::from(cause)
            .into_conflict(),
    )
}
