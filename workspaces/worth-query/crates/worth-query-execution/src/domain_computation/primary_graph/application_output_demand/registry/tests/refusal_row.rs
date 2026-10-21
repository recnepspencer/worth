//! The shared-ending tests need an actual registry-issued Ready and row stop.
use super::*;
impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn fixture_refusing_ready(
        &self,
    ) -> (
        WorthQueryOutputDemandInterest,
        super::super::SelectedReadyReadmission,
    ) {
        let (interest, _) = self.fixture_admitted_work_membership(occurrence());
        let key = interest.key.clone();
        self.state.lock().unwrap().records.get_mut(&key).unwrap().readmission_source = Some(Arc::new(super::super::source_readmission::RequiredOutputReadmission {
            source: Arc::new(()), producer: std::sync::OnceLock::new(),
            limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandResourceProfile::standard().limits(), retained_program_basis: None,
        }));
        // This local lifetime fixture declares an already delivered answer;
        // only the ending's custody and row attribution are under test.
        self.state
            .lock()
            .unwrap()
            .records
            .get_mut(&key)
            .unwrap()
            .state = super::closed_retirement::ready();
        let ready = self.interest_ready_readmission(&interest).unwrap().unwrap();
        (interest, ready)
    }
    pub(in crate::domain_computation::primary_graph) fn fixture_stop_kind(
        &self,
        key: &WorthQueryOutputDemandKey,
    ) -> Option<WorthQueryOutputDemandDenialKind> {
        self.state.lock().unwrap().records[key]
            .required_stop
            .clone()
    }
}
