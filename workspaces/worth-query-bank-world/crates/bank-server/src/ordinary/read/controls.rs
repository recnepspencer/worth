use std::num::NonZeroUsize;

use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

const MAXIMUM_ORDINARY_RESULTS: usize = 1_024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankReadControlDenial {
    ZeroResultLimit,
    ZeroWorkLimit,
    ResultLimitTooLarge { maximum: usize },
}

#[derive(Clone)]
pub struct BankReadControls {
    request: WorthQueryRequestScope,
    maximum_results: NonZeroUsize,
    maximum_work: NonZeroUsize,
}

impl BankReadControls {
    pub fn current(
        request: WorthQueryRequestScope,
        maximum_results: usize,
        maximum_work: usize,
    ) -> Result<Self, BankReadControlDenial> {
        let maximum_results =
            NonZeroUsize::new(maximum_results).ok_or(BankReadControlDenial::ZeroResultLimit)?;
        let maximum_work =
            NonZeroUsize::new(maximum_work).ok_or(BankReadControlDenial::ZeroWorkLimit)?;
        if maximum_results.get() > MAXIMUM_ORDINARY_RESULTS {
            return Err(BankReadControlDenial::ResultLimitTooLarge {
                maximum: MAXIMUM_ORDINARY_RESULTS,
            });
        }
        Ok(Self {
            request,
            maximum_results,
            maximum_work,
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

    pub const fn maximum_work(&self) -> NonZeroUsize {
        self.maximum_work
    }

    pub(crate) fn application_query_controls(
        &self,
    ) -> worth_query_host::facade::primary_graph::WorthQueryProductQueryControls<'_> {
        worth_query_host::facade::primary_graph::WorthQueryProductQueryControls::new(
            self.maximum_results,
            self.maximum_work,
            &self.request,
        )
    }
}
