//! A Current acknowledgment cannot discharge a cause on a different Product.

use std::sync::Arc;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::performed::certification::current_basis::CurrentBasis;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::performed::certification::CurrentProof;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::performed::PerformedMembers;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::own_write_fixture::with_committed_own_write;
pub trait AcknowledgementBasisCurrentProofForTest: Sized {
    fn with_current_fixture_basis(
        member: &Arc<worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::RequiredWorkMembership>,
        test: impl FnOnce(Self),
    );
}
impl AcknowledgementBasisCurrentProofForTest for CurrentProof<()> {
    fn with_current_fixture_basis(
        member: &Arc<worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::RequiredWorkMembership>,
        test: impl FnOnce(Self),
    ) {
        with_committed_own_write(|world, _, _, _| {
            world
                .application
                .with_host_advancement(|phase| {
                    let execution = phase
                        .execution_request_for(&world.application.product_runtime)
                        .unwrap();
                    let runtime = &world.application;

                    let (selected, _) = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::selection::select_required_basis(
                        &execution,
                        runtime,
                        runtime.current_world(),
                    )
                    .unwrap();
                    let basis = CurrentBasis::of(selected.selected().product().observation());
                    test(PerformedMembers::start().current(
                        (),
                        basis.clone(),
                        member.certify_current_basis(basis),
                    ));
                })
                .expect("the owner proof owns its request");
        });
    }
}
