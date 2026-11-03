//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandDenial {
    #[doc(hidden)]
    pub fn readmission_failure_for_test(&self) -> &Option<&'static str> {
        &self.readmission_failure
    }
}

impl WorthQueryOutputDemandDenial {
    #[doc(hidden)]
    pub fn readmission_failure_mut_for_test(&mut self) -> &mut Option<&'static str> {
        &mut self.readmission_failure
    }
}

impl WorthQueryOutputDemandDenial {
    #[doc(hidden)]
    pub fn new_for_test( kind: WorthQueryOutputDemandDenialKind, subject: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self::new(kind, subject)
    }
}
