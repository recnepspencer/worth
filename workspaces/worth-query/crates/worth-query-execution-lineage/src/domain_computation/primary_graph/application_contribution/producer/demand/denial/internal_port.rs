//! Crate-boundary internal port for `WorthQueryOutputDemandDenial` internal
//! diagnostics. Never exported on `worth-query-execution::facade` or Host.

use super::WorthQueryOutputDemandDenial;

pub trait OutputDemandDenialInternalPort {
    fn set_readmission_failure(&mut self, failure: &'static str);
}

impl OutputDemandDenialInternalPort for WorthQueryOutputDemandDenial {
    fn set_readmission_failure(&mut self, failure: &'static str) {
        self.readmission_failure = Some(failure);
    }
}
