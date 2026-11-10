use crate::domain_capabilities::identity::compose_certification_bundle_digest;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::super::reports::{
    WorthQueryDomainCapabilityRepresentativeReport, WorthQueryDomainCapabilitySlopeReport,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainCapabilityCertificationOutputSpec {
    name: &'static str,
    digest: String,
}

impl WorthQueryDomainCapabilityCertificationOutputSpec {
    pub(crate) fn new(name: &'static str, digest: String) -> Self {
        Self { name, digest }
    }

    pub(crate) fn name(&self) -> &'static str {
        self.name
    }

    pub(crate) fn digest(&self) -> &str {
        &self.digest
    }
}

pub(crate) fn assemble_certification_outputs(
    representative: &WorthQueryDomainCapabilityRepresentativeReport,
    slopes: &WorthQueryDomainCapabilitySlopeReport,
) -> Vec<WorthQueryDomainCapabilityCertificationOutputSpec> {
    let mut outputs = [
        "query_digest",
        "intent_declaration_digest",
        "domain_capability_contribution_request_digest",
        "domain_capability_contribution_eligibility_digest",
        "admitted_domain_capability_contribution_digest",
        "canonical_runtime_materialization_digest",
        "admission_artifact_digest",
        "support_artifact_digest",
        "workflow_artifact_digest",
        "continuity_artifact_digest",
        "aftermath_artifact_digest",
        "explanation_artifact_digest",
        "capability_support_row_digest",
        "domain_invariant_denial_digest",
        "decision_trace_digest",
        "support_traceability_digest",
        "public_boundary_digest",
        "failure_digest",
    ]
    .into_iter()
    .map(|name| {
        WorthQueryDomainCapabilityCertificationOutputSpec::new(
            name,
            representative
                .digest_for(name)
                .unwrap_or_else(|| panic!("missing representative digest {name}")),
        )
    })
    .collect::<Vec<_>>();
    outputs.extend([
        WorthQueryDomainCapabilityCertificationOutputSpec::new(
            "counter_snapshot",
            slopes.counter_snapshot().digest().to_string(),
        ),
        WorthQueryDomainCapabilityCertificationOutputSpec::new(
            "contribution_width",
            slopes.counter_snapshot().contribution_width().to_string(),
        ),
        WorthQueryDomainCapabilityCertificationOutputSpec::new(
            "trace_width",
            slopes.counter_snapshot().trace_width().to_string(),
        ),
        WorthQueryDomainCapabilityCertificationOutputSpec::new(
            "category_width",
            slopes.counter_snapshot().category_width().to_string(),
        ),
        WorthQueryDomainCapabilityCertificationOutputSpec::new(
            "support_width",
            slopes.counter_snapshot().support_width().to_string(),
        ),
        WorthQueryDomainCapabilityCertificationOutputSpec::new(
            "contribution_materialization_slope_digest",
            slopes
                .contribution_materialization_slope_digest()
                .to_string(),
        ),
        WorthQueryDomainCapabilityCertificationOutputSpec::new(
            "trace_materialization_slope_digest",
            slopes.trace_materialization_slope_digest().to_string(),
        ),
        WorthQueryDomainCapabilityCertificationOutputSpec::new(
            "category_materialization_slope_digest",
            slopes.category_materialization_slope_digest().to_string(),
        ),
        WorthQueryDomainCapabilityCertificationOutputSpec::new(
            "support_materialization_slope_digest",
            slopes.support_materialization_slope_digest().to_string(),
        ),
    ]);
    outputs
}

pub(crate) fn certification_bundle_digest(
    outputs: &[WorthQueryDomainCapabilityCertificationOutputSpec],
) -> Result<String, IdentityDerivationDenial> {
    compose_certification_bundle_digest(
        outputs
            .iter()
            .map(|output| (output.name(), output.digest())),
    )
}
