#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryProviderSessionProtocolCounters {
    authority_checks: usize,
    closure_items_bound: usize,
    provider_calls: usize,
    token_mints: usize,
}

impl WorthQueryProviderSessionProtocolCounters {
    pub fn authority_checks(self) -> usize {
        self.authority_checks
    }

    pub fn closure_items_bound(self) -> usize {
        self.closure_items_bound
    }

    pub fn provider_calls(self) -> usize {
        self.provider_calls
    }

    pub fn token_mints(self) -> usize {
        self.token_mints
    }

    pub(super) fn checked_authority(&mut self) {
        self.authority_checks += 1;
    }

    pub(super) fn bound_closure_items(&mut self, count: usize) {
        self.closure_items_bound += count;
    }

    pub(super) fn called_provider(&mut self) {
        self.provider_calls += 1;
    }

    pub(super) fn minted_token(&mut self) {
        self.token_mints += 1;
    }
}
