//! Prepare only the exact current mirror regions from the prepared Native root.
use worth_relational_contracts::internal_port as contracts;
pub(crate) fn prepare_mirror_publication(
    mirror: &worth_relational_storage::internal_port::runtime::state::subsystems::storage::residency_demand::MirrorResidencyDemandBinding,
    symbols: &worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    branch: &contracts::history::data::coordinates::BranchId,
    root: &worth_relational_branch::internal_port::branch::root::RelationalBranchRoot,
    publication: &worth_relational_branch::internal_port::branch::symbol_publication::SymbolPublicationAdmission,
    clone_mode: worth_relational_storage::internal_port::storage::overlay::partition::PartitionCloneMode,
    committed: std::collections::BTreeMap<
        contracts::identity::data::primitives::PartitionId,
        (
            worth_relational_storage::internal_port::storage::overlay::partition::PartitionState,
            worth_relational_storage::internal_port::storage::overlay::partition::PartitionMutationJournal,
        ),
    >,
) -> Result<
    worth_relational_storage::internal_port::storage::mirror_plan::PreparedMirrorPublication,
    worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure,
>{
    contracts::execution::request_custody::check_request(execution)
        .map_err(worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure::Request)?;
    let mut prepared = worth_relational_storage::internal_port::storage::mirror_plan::PreparedMirrorPublication::empty();
    if !mirror.is_main_branch(branch) {
        return Ok(prepared);
    }
    let edition = mirror.edition();
    for (id, (partition, journal)) in committed {
        worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess::demand_head_partition(
            &edition.with_symbol_admission(symbols),
            execution,
            id,
        )?;
        let target = mirror.admit_current_target(id)?;
        match target {
            Some(target) => {
                target.demand_preserved_symbols(execution, symbols)?;
                let readmission = prepare_symbol_readmission(root, publication, symbols, id)?;
                prepared.prepare_existing(target, clone_mode, partition, journal, readmission);
            }
            None => {
                let lease = root
                    .read()
                    .resident_full_region_witness(id)?
                    .ok_or(contracts::storage::data::storage_unavailable::RelationalStorageUnavailable {
                    subject: contracts::storage::data::storage_unavailable::RelationalStorageSubject::Region(id),
                    cause:
                        contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause::SourceUnavailable,
                })?;
                // The immutable branch root is a real reader. Give the mirror
                // its own owned payload now, rather than depending on a
                // later no-op lifecycle edit to detach this initial cell.
                let mirror = lease.into_edit().freeze();
                prepared.prepare_new(
                    worth_relational_storage::internal_port::runtime::state::subsystems::storage::resident_target::ResidentMirrorTarget::new(
                        mirror, false,
                    ),
                    &journal,
                );
            }
        }
    }
    Ok(prepared)
}

fn prepare_symbol_readmission(
    root: &worth_relational_branch::internal_port::branch::root::RelationalBranchRoot,
    publication: &worth_relational_branch::internal_port::branch::symbol_publication::SymbolPublicationAdmission,
    symbols: &worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    id: contracts::identity::data::primitives::PartitionId,
) -> Result<
    worth_relational_storage::internal_port::storage::residency::resident_history::ResidentHistoryReadmission,
    worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure,
>{
    use worth_relational_storage::internal_port::storage::data::{
        RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
    };
    let change = root.commit_change().expect(
        /* Proof: SymbolPublicationAdmission::prepare requires this prepared root's
         * CommitChange before mirror preparation. */
        "publication root has change",
    );
    let symbols_for_region =
        publication
            .for_region(change, id)
            .ok_or(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Region(id),
                cause: RelationalStorageUnavailableCause::MissingSymbolMaintenancePredecessor,
            })?;
    Ok(worth_relational_storage::internal_port::storage::residency::resident_history::ResidentHistoryReadmission::new(
        root.retained_schema_authority(),
        symbols,
        &worth_relational_durability::internal_port::storage::residency::region_load::CHECKPOINT_REGION_READMISSION,
    )
    .with_symbol_publication(symbols_for_region))
}
