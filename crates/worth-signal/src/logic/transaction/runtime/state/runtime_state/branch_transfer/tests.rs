use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;
use crate::state::SignalBranchId;

use super::super::{AuthorityTransferPacket, BranchLifecycleTransfer};

#[test]
fn transfer_validation_denies_mismatched_packet_before_active_or_stored_state_moves() {
    let mut graph = SignalGraph::new();
    let source = graph.create_node();
    let derived = graph.create_node();
    graph
        .set_dependencies(derived, [DependencyEdge::new(source, Aspect::new(0))])
        .expect("populated transfer fixture installs");
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    let active_before = runtime.current_branch();
    let state = runtime
        .capture_heavy_branch_state()
        .expect("non-moving capture prepares disputed transfer input");
    let invalid_branch_id = SignalBranchId(active_before.id.0 + 100);

    let denial = runtime.prepare_branch_lifecycle_transfer(BranchLifecycleTransfer::Move(
        AuthorityTransferPacket::new(invalid_branch_id, state),
    ));
    assert!(denial.is_err());
    assert_eq!(runtime.current_branch(), active_before);
    assert_eq!(
        runtime.graph().dependency_sources_of(derived),
        Ok(vec![source]),
        "validation failure cannot leave the active graph at Default"
    );
}
