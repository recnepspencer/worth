use super::market_seed::MarketSeed;
use super::scales::FintechScale;
use super::scenarios::setup_world;

#[test]
fn fintech_intraday_world_setup_builds_seeded_branchable_graph() {
    let mut fixture = setup_world();
    fixture.assert_shape(FintechScale::smoke());
    let baseline_nodes = fixture.live_node_count();
    let baseline_audit = fixture.read_primary_audit_surface().unwrap();
    let analysis = fixture.open_branch("analysis").unwrap();
    fixture.seed_market(MarketSeed::high_vol(11)).unwrap();
    let snapshot = fixture.capture_world_snapshot();

    assert!(baseline_nodes > 0);
    assert!(baseline_audit.desk.get(super::aspects::RISK) > 0);
    assert!(baseline_audit.scenario.get(super::aspects::RISK) > 0);
    assert_eq!(
        snapshot.diagnostic_graph.live_node_ids().len(),
        baseline_nodes
    );
    assert_eq!(fixture.instruments.len(), FintechScale::smoke().instruments);
    assert_eq!(analysis.name, "analysis");
    assert_eq!(fixture.current_branch().name, "analysis");
}

#[test]
#[ignore = "stress shape for hostile domain workflows"]
fn fintech_intraday_world_setup_stress_builds_over_10k_nodes() {
    let fixture = super::scenarios::setup_seeded_world_with(
        FintechScale::stress_10k(),
        super::regimes::MarketRegime::Calm,
        7,
    );
    fixture.assert_shape(FintechScale::stress_10k());
    assert!(fixture.live_node_count() >= 10_000);
}
