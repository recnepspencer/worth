mod outputs;

use self::outputs::{assemble_certification_outputs, certification_bundle_digest};
use super::output_manifest::worth_query_domain_capability_certification_output_manifest;
use super::reports::{
    worth_query_domain_capability_representative_report_in,
    worth_query_domain_capability_slope_report_in,
    WorthQueryDomainCapabilityCertificationCounterSnapshot,
    WorthQueryDomainCapabilityRepresentativeReport, WorthQueryDomainCapabilitySlopeReport,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainCapabilityCertificationOutput {
    name: &'static str,
    digest: String,
}

impl WorthQueryDomainCapabilityCertificationOutput {
    pub(crate) fn new(name: &'static str, digest: String) -> Self {
        Self { name, digest }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainCapabilityCertificationBundle {
    output_manifest: Vec<&'static str>,
    representative_report: WorthQueryDomainCapabilityRepresentativeReport,
    slope_report: WorthQueryDomainCapabilitySlopeReport,
    outputs: Vec<WorthQueryDomainCapabilityCertificationOutput>,
    certification_bundle_digest: String,
}

impl WorthQueryDomainCapabilityCertificationBundle {
    fn new(
        representative_report: WorthQueryDomainCapabilityRepresentativeReport,
        slope_report: WorthQueryDomainCapabilitySlopeReport,
    ) -> Self {
        let output_manifest =
            worth_query_domain_capability_certification_output_manifest().to_vec();
        let output_specs = assemble_certification_outputs(&representative_report, &slope_report);
        validate_output_manifest(&output_manifest, &output_specs);
        let certification_bundle_digest = certification_bundle_digest(&output_specs);
        let outputs = output_specs
            .iter()
            .map(|output| {
                WorthQueryDomainCapabilityCertificationOutput::new(
                    output.name(),
                    output.digest().to_string(),
                )
            })
            .collect::<Vec<_>>();

        Self {
            output_manifest,
            representative_report,
            slope_report,
            outputs,
            certification_bundle_digest,
        }
    }

    pub fn output_manifest(&self) -> &[&'static str] {
        &self.output_manifest
    }

    pub fn representative_report(&self) -> &WorthQueryDomainCapabilityRepresentativeReport {
        &self.representative_report
    }

    pub fn slope_report(&self) -> &WorthQueryDomainCapabilitySlopeReport {
        &self.slope_report
    }

    pub fn counter_snapshot(&self) -> &WorthQueryDomainCapabilityCertificationCounterSnapshot {
        self.slope_report.counter_snapshot()
    }

    pub fn outputs(&self) -> &[WorthQueryDomainCapabilityCertificationOutput] {
        &self.outputs
    }

    pub fn output_digest(&self, key: &str) -> Option<&str> {
        self.outputs
            .iter()
            .find(|output| output.name() == key)
            .map(WorthQueryDomainCapabilityCertificationOutput::digest)
    }

    pub fn certification_bundle_digest(&self) -> &str {
        &self.certification_bundle_digest
    }
}

pub fn certify_domain_capabilities(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryDomainCapabilityCertificationBundle {
    let installation = super::install_domain_capability_certification(execution);
    certify_domain_capabilities_in(installation.contributions())
}

pub(crate) fn certify_domain_capabilities_in(
    domain: &crate::domain_capabilities::WorthQueryInstalledDomainContributionSurface,
) -> WorthQueryDomainCapabilityCertificationBundle {
    let representative_report = worth_query_domain_capability_representative_report_in(domain);
    let slope_report =
        worth_query_domain_capability_slope_report_in(&representative_report, domain);

    WorthQueryDomainCapabilityCertificationBundle::new(representative_report, slope_report)
}

fn validate_output_manifest(
    output_manifest: &[&'static str],
    outputs: &[self::outputs::WorthQueryDomainCapabilityCertificationOutputSpec],
) {
    let actual_names = outputs
        .iter()
        .map(|output| output.name())
        .collect::<Vec<_>>();
    assert_eq!(
        actual_names.len(),
        actual_names
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        "domain-capability certification outputs must be duplicate-free"
    );
    assert_eq!(
        actual_names, output_manifest,
        "domain-capability certification outputs must match the compile-visible manifest exactly"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn certification_bundle_matches_required_manifest() {
        let host_request = crate::test_execution_policy::serial_request();
        let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

        let bundle = certify_domain_capabilities(request_execution);

        assert_eq!(
            bundle.output_manifest(),
            worth_query_domain_capability_certification_output_manifest()
        );
        assert!(bundle
            .output_digest("query_digest")
            .is_some_and(|digest| !digest.is_empty()));
        assert!(bundle
            .output_digest("support_materialization_slope_digest")
            .is_some_and(|digest| !digest.is_empty()));
        assert!(!bundle.certification_bundle_digest().is_empty());
    }
}
