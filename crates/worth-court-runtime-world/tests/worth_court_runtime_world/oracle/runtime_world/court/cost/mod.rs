

#[allow(unused_imports)] use crate::court::reference_context::cost::*;
#[allow(unused_imports)]
pub(crate) use {crate::court::growth_context::cost::profiles};



pub(crate) use {worth_runtime_world::facade::*};
pub(crate) use {crate::court::reference_context::cost::population::Axis, crate::court::reference_context::cost::population::Population};
pub(crate) use {std::time::Instant};
#[test]
fn court_structural_costs_do_not_scan_unrelated_b_h_u_a_p_o_populations() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut baseline = None;
    let mut publication_baseline = None;
    for axis in crate::court::reference_context::cost::population::Axis::ALL {
        for size in [1, 4, 8] {
            let mut population = crate::court::reference_context::cost::population::Population::build(request_execution, axis, size);
            let (_, work) = observe_probe(&population);
            assert_eq!(
                *baseline.get_or_insert(work),
                work,
                "axis={axis:?}, size={size}"
            );
            let (_, costs) = publication_probe(&mut population);
            assert_eq!(*publication_baseline.get_or_insert(costs), costs);
            population.finish();
        }
    }
}
#[test]
fn court_concurrent_writers_keep_per_attempt_costs_constant() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut baseline = None;
    for width in [1, 4, 8] {
        let (_, costs) = crate::court::reference_context::cost::writers::run(request_execution, width);
        assert_eq!(*baseline.get_or_insert(costs), costs);
    }
}
