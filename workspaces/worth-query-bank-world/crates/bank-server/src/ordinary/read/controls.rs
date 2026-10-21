use std::num::NonZeroUsize;

use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

const MAXIMUM_ORDINARY_RESULTS: usize = 1_024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankReadControlDenial {
    ZeroResultLimit,
    ResultLimitTooLarge { maximum: usize },
}

#[derive(Clone)]
pub struct BankReadControls {
    request: WorthQueryRequestScope,
    maximum_results: NonZeroUsize,
}

impl BankReadControls {
    pub fn current(
        request: WorthQueryRequestScope,
        maximum_results: usize,
    ) -> Result<Self, BankReadControlDenial> {
        let maximum_results =
            NonZeroUsize::new(maximum_results).ok_or(BankReadControlDenial::ZeroResultLimit)?;
        if maximum_results.get() > MAXIMUM_ORDINARY_RESULTS {
            return Err(BankReadControlDenial::ResultLimitTooLarge {
                maximum: MAXIMUM_ORDINARY_RESULTS,
            });
        }
        Ok(Self {
            request,
            maximum_results,
        })
    }

    pub const fn request(&self) -> &WorthQueryRequestScope {
        &self.request
    }

    pub const fn maximum_results(&self) -> usize {
        self.maximum_results.get()
    }

    pub(crate) const fn maximum_result_count(&self) -> NonZeroUsize {
        self.maximum_results
    }
}
