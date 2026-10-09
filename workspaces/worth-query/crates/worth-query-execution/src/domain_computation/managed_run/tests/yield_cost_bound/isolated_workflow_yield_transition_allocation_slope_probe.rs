//! Isolated workflow yield transition allocation slope probe.

#[cfg(feature = "allocation-probes")]
use super::*;

#[test]
#[cfg(feature = "allocation-probes")]
fn isolated_workflow_yield_transition_allocation_slope_probe() {
    if std::env::var_os("WORTH_QUERY_WORKFLOW_YIELD_ALLOCATION_PROBE").is_none() {
        return;
    }
    let baseline = measured_workflow_target(0);
    let wide = measured_workflow_target(UNRELATED_WIDTH);
    assert_eq!(baseline.allocations, wide.allocations);
    assert_eq!(baseline.reallocations, wide.reallocations);
}
