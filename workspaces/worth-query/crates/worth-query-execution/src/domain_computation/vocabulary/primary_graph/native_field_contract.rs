//! Moved whole from `domain_computation::primary_graph::application_contribution::native_field_contract`: the items below and every impl they own.
use worth_query_declaration::facade::application_schema as schema;

#[derive(Debug)]
pub struct NativeFieldContractDenial {
    pub(in crate::domain_computation) locus: schema::ApplicationFieldBindingLocus,
    pub(in crate::domain_computation) detail: String,
}

impl NativeFieldContractDenial {
    pub(in crate::domain_computation) fn detail(&self) -> &str {
        &self.detail
    }
    pub(in crate::domain_computation) fn field_locus(
        &self,
    ) -> Option<&schema::ApplicationFieldBindingLocus> {
        Some(&self.locus)
    }
}

impl NativeFieldContractDenial {
    #[doc(hidden)]
    pub fn field_locus_for_test(&self) -> Option<&schema::ApplicationFieldBindingLocus> {
        Self::field_locus(self)
    }
}

impl NativeFieldContractDenial {
    #[doc(hidden)]
    pub fn detail_for_test(&self) -> &str {
        Self::detail(self)
    }
}
