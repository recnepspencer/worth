//! Stage a whole same-cell transaction without moving the canonical head.
use super::{AuthorityState, BranchState};
impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn durable_successor_framework_charge() -> Result<
        crate::data::retained_storage::RetainedStorageCharge,
        crate::data::retained_storage::RetainedStoragePreparationDenial,
    > {
        use crate::data::retained_storage::RetainedStorageCharge as Charge;
        Charge::capacity::<Self>(1)?
            .checked_add(Charge::capacity::<crate::data::graph::SignalGraph>(1)?)?
            .checked_add(Charge::capacity::<super::DerivedState<D, I>>(1)?)
    }
    pub(crate) fn stage_durable_successor(&mut self) -> Box<Self> {
        let graph = self.authority.graph.fork_durable_successor();
        Box::new(Self {
            authority: AuthorityState {
                graph,
                config: self.authority.config.fork_persistent(),
            },
            derived: Box::new(self.derived.fork_persistent()),
            ancestry: self.ancestry.clone(),
            mutation_ledger: self.mutation_ledger.clone(),
            installed_definition: self.installed_definition.clone(),
        })
    }
    pub(crate) fn shares_authored_storage_with(&self, other: &Self) -> bool {
        self.graph().shares_authored_storage_with(other.graph())
            && self.config().shares_authored_storage_with(other.config())
    }
}
