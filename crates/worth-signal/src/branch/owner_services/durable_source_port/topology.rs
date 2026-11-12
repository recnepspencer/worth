//! Read-only topology observations retain no graph or branch custody.
use super::{SignalDurableBranchSourcePort, SignalDurableSourceDenial as D};
impl<Dm, I, T> SignalDurableBranchSourcePort<Dm, I, T>
where
    Dm: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub fn topology_observations(
        &self,
    ) -> Result<Vec<(usize, crate::data::graph::SignalGraphLifecycleProbe)>, D> {
        let branches = self.branches()?;
        let owner = self.owner()?;
        let admission = owner
            .admit()
            .map_err(super::super::basis_port::denial_mapping::map_basis_admission_denial)
            .map_err(D::Admission)?;
        let mut observations = Vec::with_capacity(branches.len());
        for (_, handle) in branches {
            let id = handle.id;
            let cell = owner
                .lookup_cell(&admission, id)
                .map_err(|cause| super::super::owner::basis::map_basis_registry_denial(cause, id))
                .map_err(D::Observation)?;
            observations.push(cell.inspect_durable_topology(&admission)?);
        }
        Ok(observations)
    }
}
