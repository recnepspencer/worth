//! The shared-ending tests need an actual registry-issued Ready and row stop.

use crate::domain_computation::primary_graph::application_output_demand::registry::tests::required_work::RequiredWorkWorthQueryOutputDemandRegistryForTest;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
/// Test doors on a library type, owned by the moved courts.
pub trait RefusalRowWorthQueryOutputDemandRegistryForTest: Sized {
    fn fixture_refusing_ready(
        &self) -> (
        WorthQueryOutputDemandInterest,
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::selection::SelectedReadyReadmission,
    );
    fn fixture_stop_kind(
        &self,
        key: &WorthQueryOutputDemandKey,
    ) -> Option<WorthQueryOutputDemandDenialKind>;
}

impl RefusalRowWorthQueryOutputDemandRegistryForTest for WorthQueryOutputDemandRegistry {
    fn fixture_refusing_ready(
        &self,
    ) -> (
        WorthQueryOutputDemandInterest,
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::selection::SelectedReadyReadmission,
    ){
        let (interest, _) = self.fixture_admitted_work_membership(occurrence());
        let key = (*interest.key_for_test()).clone();(*(*
        (*self.state_for_test()).lock().unwrap().records_mut_for_test()).get_mut(&key).unwrap().readmission_source_mut_for_test()) = Some(Arc::new(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::source_readmission::RequiredOutputReadmission {
            source: Arc::new(()), producer: std::sync::OnceLock::new(),
            limits: worth_query_execution::facade::domain_computation::WorthQueryOutputDemandResourceProfile::standard().limits(), retained_program_basis: None,
        }));(*(*
        // This local lifetime fixture declares an already delivered answer;
        // only the ending's custody and row attribution are under test.
        (*self.state_for_test())
            .lock()
            .unwrap()
            .records_mut_for_test())
            .get_mut(&key)
            .unwrap()
            .state_mut_for_test()) = crate::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready();
        let ready = self.interest_ready_readmission_for_test(&interest).unwrap().unwrap();
        (interest, ready)
    }

    fn fixture_stop_kind(
        &self,
        key: &WorthQueryOutputDemandKey,
    ) -> Option<WorthQueryOutputDemandDenialKind> {(*(*
        (*self.state_for_test()).lock().unwrap().records_for_test())[key]
            .required_stop_for_test())
            .clone()
    }
}
