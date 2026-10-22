//! A prepared run owns its callback; another run cannot drain its receipt.
pub(in super::super) struct KernelCostObservationForTest {
    report: Box<dyn Fn(u64, u64) + Send + Sync>,
}
impl KernelCostObservationForTest {
    pub(super) fn new(report: impl Fn(u64, u64) + Send + Sync + 'static) -> Self {
        crate::domain_computation::execution_observation::owned_control_scope::current_id();
        Self {
            report: Box::new(report),
        }
    }
    pub(in super::super) fn record(&self, work: u64, span: u64) {
        (self.report)(work, span);
    }
}
