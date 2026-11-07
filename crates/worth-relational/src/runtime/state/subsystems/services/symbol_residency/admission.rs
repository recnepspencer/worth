//! Dictionary-specific closure admission and persistent snapshot carry.
use super::*;
impl RuntimeSymbolResidency {
    pub(in crate::runtime::state::subsystems::services) fn demand_region(
        &self,
        owner: &RuntimeSymbolTable,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        region: crate::storage::residency::RelationalRegionSymbolAdmissionIdentity,
        predecessor: Option<(
            crate::storage::residency::RelationalRegionSymbolAdmissionIdentity,
            &BTreeSet<Symbol>,
            u64,
        )>,
        ids: impl FnOnce() -> Result<(BTreeSet<Symbol>, u64), Failure>,
    ) -> Result<(), Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        let admitted = owner.admissions_snapshot();
        if admitted.contains_key(&region) {
            return Ok(());
        }
        let covered_head = region.covered_head();
        let required = match predecessor {
            Some((previous, supplement, visits)) if admitted.contains_key(&previous) => {
                let _ = visits;
                (supplement.clone(), supplement.len() as u64)
            }
            _ => ids()?,
        };
        use crate::storage::substrate::maintenance::{ChangeWork, MaintenanceRefusal};
        ChangeWork::run_phase(execution, "symbol_admission", |work| {
            work.prepare(required.1, 0)?;
            Ok::<_, MaintenanceRefusal>(self.demand_ids(owner, execution, required.0))
        }).map_err(|cause| match cause {
            MaintenanceRefusal::Execution(cause) => Failure::Request(cause),
            MaintenanceRefusal::MissingPredecessor | MaintenanceRefusal::UnavailableChange => unreachable!(/* Proof: This private scope performs only ChangeWork::prepare and returns its demand result as a value; it cannot issue semantic maintainer refusals. */ "symbol work scope only refuses Execution"),
        })??;
        owner.publish_admissions(
            std::iter::once(region.clone())
                .chain(covered_head)
                .chain(region.covered_owned())
                .collect(),
        );
        Ok(())
    }

    pub(in crate::runtime::state::subsystems::services) fn shares_directory(
        &self,
        previous: &Self,
    ) -> bool {
        Arc::ptr_eq(&self.by_id, &previous.by_id) && Arc::ptr_eq(&self.by_value, &previous.by_value)
    }
}
