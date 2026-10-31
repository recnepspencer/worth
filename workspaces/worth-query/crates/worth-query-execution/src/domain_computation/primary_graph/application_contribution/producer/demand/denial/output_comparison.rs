//! Original Bridge comparison stage/cause, separate from recovery grouping.
use super::WorthQueryOutputDemandDenial;
impl WorthQueryOutputDemandDenial {
    pub(in crate::domain_computation::primary_graph) fn output_comparison(
        cause: worth_runtime_bridge::facade::BridgeOutputComparisonDenial,
    ) -> Self {
        let mut denial = Self::new(
            super::super::bridge_denial::bridge_denial_kind(cause.cause().kind()),
            "",
        );
        denial.evidence_mut().output_comparison = Some(cause);
        denial
    }
    /// The exact admission/baseline/candidate stage and original Bridge cause.
    pub fn output_comparison_denial(
        &self,
    ) -> Option<&worth_runtime_bridge::facade::BridgeOutputComparisonDenial> {
        self.evidence
            .as_ref()
            .and_then(|evidence| evidence.output_comparison.as_ref())
    }
}
