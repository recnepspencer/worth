/// Scoped observer of actual original-account settlements on the current thread.
/// It grants no request, budget or publication authority.
pub struct RelationalWorkAccountRecorderForTest;
impl Default for RelationalWorkAccountRecorderForTest {
    fn default() -> Self {
        Self::new()
    }
}
impl RelationalWorkAccountRecorderForTest {
    pub const fn new() -> Self {
        Self
    }
    pub fn record_for_test<R>(&self, body: impl FnOnce() -> R) -> (R, Vec<u64>) {
        super::account_observation::observe(body)
    }
}
