use super::EdgeTopology;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageForkGrowth,
    RetainedStorageForkGrowthDenial as Denial, RetainedStoragePreparation as Work,
};

impl RetainedStorageForkGrowth for EdgeTopology {
    fn prepare_fork_growth(&mut self, work: &mut Work) -> Result<Charge, Denial> {
        work.reserve_visits(std::mem::size_of::<Self>())?;
        let Self {
            dependency_snapshots,
            dependency_snapshot_shapes,
            dependency_snapshot_storage_custody: _,
            dependency_edges,
            subscriber_edges,
            reverse_subscriptions,
            pending_revalidation_waiters,
            pending_revalidation_storage_custody: _,
        } = self;
        let mut growth = Charge::ZERO;
        macro_rules! add {
            ($root:expr) => {
                match $root.prepare_fork_growth(work) {
                    Ok(charge) => growth = growth.checked_add(charge)?,
                    Err(denial) => return Err(denial),
                }
            };
        }
        add!(dependency_snapshots);
        add!(dependency_snapshot_shapes);
        add!(dependency_edges);
        add!(subscriber_edges);
        add!(reverse_subscriptions);
        add!(pending_revalidation_waiters);
        Ok(growth)
    }
}
