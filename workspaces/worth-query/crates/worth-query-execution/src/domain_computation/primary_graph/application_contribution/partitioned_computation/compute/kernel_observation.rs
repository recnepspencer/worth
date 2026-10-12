//! A prepared run owns its callback; another run cannot drain its receipt.
pub(in super::super) struct KernelCostObservationForTest {
    bytes: u64,
    report: Box<dyn Fn(u64, u64) + Send + Sync>,
}
impl KernelCostObservationForTest {
    pub(super) fn new(report: impl Fn(u64, u64) + Send + Sync + 'static) -> Self {
        crate::domain_computation::execution_observation::owned_control_scope::current_id();
        Self {
            bytes: u64::try_from(size_of_val(&report)).expect("callback storage fits u64"),
            report: Box::new(report),
        }
    }
    pub(in super::super) fn record(&self, work: u64, span: u64) {
        (self.report)(work, span);
    }
    pub(super) fn bytes(&self) -> u64 {
        self.bytes
    }
}
