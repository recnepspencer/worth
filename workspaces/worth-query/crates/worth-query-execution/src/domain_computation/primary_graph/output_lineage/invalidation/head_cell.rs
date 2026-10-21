//! Exact-map branch lookup installation under native head exclusion.
use super::source_alignment::BranchMarkRoot;
use super::SourceInvalidationOwner;
use worth_relational::facade::branch::RelationalBranchIdentity;
use worth_relational::facade::mvcc::CompanionBranchCell;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::mvcc::PublicationCompanionRegistrationStop;
use worth_relational::facade::runtime::RelationalRuntime;
#[derive(Debug)]
pub(in crate::domain_computation) enum HeadCellRegistrationStop {
    Admission(CompanionPreflightStop),
    Native(PublicationCompanionRegistrationStop),
    LookupChanged,
}
impl From<CompanionPreflightStop> for HeadCellRegistrationStop {
    fn from(stop: CompanionPreflightStop) -> Self {
        Self::Admission(stop)
    }
}
impl SourceInvalidationOwner {
    pub(in crate::domain_computation) fn mint_cell_at_head(
        &self,
        registration: &worth_runtime_bridge::facade::RelationalBridgeCanonicalSubscription,
        runtime: &RelationalRuntime,
        branch: &RelationalBranchIdentity,
    ) -> Result<(), HeadCellRegistrationStop> {
        if branch.runtime_instance_id() != self.runtime_instance_id {
            return Err(HeadCellRegistrationStop::Native(
                PublicationCompanionRegistrationStop::ForeignRuntime,
            ));
        }
        let (expected, initial) = {
            let mut branches = self
                .branches
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if branches
                .cells
                .get(branch.branch_id())
                .and_then(|slot| slot.admitted())
                .is_some()
            {
                return Ok(());
            }
            (branches.cells.clone(), self.vacant_root(&mut branches))
        };
        let install_branch = branch.branch_id().clone();
        registration
            .with_branch_cell_at_head(
                runtime,
                branch,
                initial,
                |cell: CompanionBranchCell<BranchMarkRoot>| {
                    let mut branches = self
                        .branches
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    if !branches.cells.ptr_eq(&expected) {
                        return Err(HeadCellRegistrationStop::LookupChanged);
                    }
                    branches
                        .cells
                        .insert(install_branch, cell.into_lookup_slot());
                    Ok(())
                },
            )
            .map_err(HeadCellRegistrationStop::Native)?
    }
}
