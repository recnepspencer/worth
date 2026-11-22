//! Sealed test doors over this module's private members.
use super::*;

impl ProducerRequestAdmissionRejection {
    #[doc(hidden)]
    pub fn into_denial_for_test(self) -> WorthQueryOutputDemandDenial {
        Self::into_denial(self)
    }
}

impl ProducerExecutionStop {
    #[doc(hidden)]
    pub fn into_output_denial_for_test(self) -> WorthQueryOutputDemandDenial {
        Self::into_output_denial(self)
    }
}
