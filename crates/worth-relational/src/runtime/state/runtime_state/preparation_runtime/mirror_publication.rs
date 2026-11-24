//! Prepare only the exact current mirror regions from the prepared Native root.
pub(crate) fn prepare_mirror_publication(
    mirror: &crate::runtime::state::subsystems::storage::MirrorResidencyDemandBinding,
    symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    branch: &crate::history::data::coordinates::BranchId,
    root: &crate::branch::root::RelationalBranchRoot,
    publication: &crate::branch::symbol_publication::SymbolPublicationAdmission,
    clone_mode: crate::storage::overlay::PartitionCloneMode,
    committed: std::collections::BTreeMap<
        crate::identity::data::PartitionId,
        (
            crate::storage::overlay::PartitionState,
            crate::storage::overlay::PartitionMutationJournal,
        ),
    >,
) -> Result<
    crate::storage::PreparedMirrorPublication,
    crate::storage::residency::RelationalResidencyDemandFailure,
> {
    crate::execution::request_custody::check_request(execution)
        .map_err(crate::storage::residency::RelationalResidencyDemandFailure::Request)?;
    let mut prepared = crate::storage::PreparedMirrorPublication::empty();
    if !mirror.is_main_branch(branch) {
        return Ok(prepared);
    }
    let edition = mirror.edition();
    for (id, (partition, journal)) in committed {
        crate::storage::overlay::CurrentPartitionAccess::demand_head_partition(
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
                    .resident_full_region_witness(id)?
                    .ok_or(crate::storage::data::RelationalStorageUnavailable {
                    subject: crate::storage::data::RelationalStorageSubject::Region(id),
                    cause:
                        crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable,
                })?;
                // The immutable branch root is a real reader. Give the mirror
                // its own owned payload now, rather than depending on a
                // later no-op lifecycle edit to detach this initial cell.
                let mirror = lease.into_edit().freeze();
                prepared.prepare_new(
                    crate::runtime::state::subsystems::storage::ResidentMirrorTarget::new(
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
    root: &crate::branch::root::RelationalBranchRoot,
    publication: &crate::branch::symbol_publication::SymbolPublicationAdmission,
    symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    id: crate::identity::data::PartitionId,
) -> Result<
    crate::storage::residency::ResidentHistoryReadmission,
    crate::storage::residency::RelationalResidencyDemandFailure,
> {
    use crate::storage::data::{
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
    Ok(crate::storage::residency::ResidentHistoryReadmission::new(
        root.retained_schema_authority(),
        symbols,
    )
    .with_symbol_publication(symbols_for_region))
}
