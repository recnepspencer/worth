//! Moved whole from `domain_computation::primary_graph::application_contribution::conditional::output_readiness_contract`: the items below and every impl they own.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOutputReadinessContractDenial {
    detail: String,
    locus:
        Option<worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus>,
}

impl From<crate::domain_computation::vocabulary::primary_graph::native_field_contract::NativeFieldContractDenial>
    for WorthQueryOutputReadinessContractDenial
{
    fn from(denial: crate::domain_computation::vocabulary::primary_graph::native_field_contract::NativeFieldContractDenial) -> Self {
        Self {
            locus: Some(denial.locus),
            detail: denial.detail,
        }
    }
}

impl WorthQueryOutputReadinessContractDenial {
    /// The refusal the readiness contract owner builds: its detail and, for a field, its locus.
    pub(in crate::domain_computation) fn new(
        detail: String,
        locus: Option<
            worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
        >,
    ) -> Self {
        Self { detail, locus }
    }

    pub fn field_locus(
        &self,
    ) -> Option<&worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus>
    {
        self.locus.as_ref()
    }
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl std::fmt::Display for WorthQueryOutputReadinessContractDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for WorthQueryOutputReadinessContractDenial {}
