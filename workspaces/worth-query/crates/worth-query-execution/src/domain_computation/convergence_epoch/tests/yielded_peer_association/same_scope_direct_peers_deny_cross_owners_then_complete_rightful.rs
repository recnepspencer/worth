//! same scope direct peers deny cross owners then complete rightfully.
use super::*;

#[test]
fn same_scope_direct_peers_deny_cross_owners_then_complete_rightfully() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let DirectYieldedPeer {
            runtime: runtime_a,
            bridge: bridge_a,
            yielded: yielded_a,
        } = direct_yielded_peer(execution);
        let DirectYieldedPeer {
            runtime: runtime_b,
            bridge: bridge_b,
            yielded: yielded_b,
        } = direct_yielded_peer(execution);

        assert_ne!(yielded_a.epoch_identity(), yielded_b.epoch_identity());
        assert_ne!(
            yielded_a.graph_authority_identity(),
            yielded_b.graph_authority_identity()
        );
        let yielded_a =
            deny_cross_direct_owners(execution, yielded_a, &runtime_a, &runtime_b, &bridge_b);
        let yielded_b =
            deny_cross_direct_owners(execution, yielded_b, &runtime_b, &runtime_a, &bridge_a);

        let completed_a = complete_direct_peer(execution, yielded_a, &runtime_a, &bridge_a);
        let completed_b = complete_direct_peer(execution, yielded_b, &runtime_b, &bridge_b);
        assert_eq!(completed_a.state_identity, completed_b.state_identity);
        assert_ne!(
            completed_a.occurrence_identity,
            completed_b.occurrence_identity
        );
    });
}
