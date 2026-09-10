use super::SignalBranchCellState;

impl<D, I, T> SignalBranchCellState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn conditional_execution_graph_mut(
        &mut self,
    ) -> &mut crate::data::graph::SignalGraph {
        self.state.graph_mut()
    }
}
