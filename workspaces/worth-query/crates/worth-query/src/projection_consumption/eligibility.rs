mod outcomes;
use outcomes::{
    admitted_projection_consumption, deferred_projection_consumption,
    denied_projection_consumption, source_mismatched_projection_consumption,
};

use super::contracts::MaterializedProjectionContract;
use super::declaration::ProjectionConsumptionDeclaration;
use super::facts::{ProjectionFactKind, ProjectionFactRequest};
use super::identity::{
    compose_eligibility_admitted_digest, compose_eligibility_deferred_failure_digest,
    compose_eligibility_denied_failure_digest, compose_eligibility_source_mismatch_failure_digest,
    compose_eligibility_warning_kinds_digest,
};
use super::source::ProjectionSourceFamily;
use super::support::{support_for_kind, ProjectionConsumptionSupportPosture};
use super::visibility::visibility_denial;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectionConsumptionDenialReason {
    FactFamilyNotVisible { field_key: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeferredProjectionConsumptionReason {
    WriteReceiptContractBindingPending,
    SourceFamilySupportPending,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectionConsumptionWarningKind {
    QueryContextRowBound,
    PreviewDerivedContext,
}

impl ProjectionConsumptionWarningKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::QueryContextRowBound => "query_context_row_bound",
            Self::PreviewDerivedContext => "preview_derived_context",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionWarnings {
    warning_kinds: Vec<ProjectionConsumptionWarningKind>,
    warning_digest: String,
}

impl ProjectionConsumptionWarnings {
    pub(crate) fn for_certification(
        kind: ProjectionConsumptionWarningKind,
    ) -> Result<Self, IdentityDerivationDenial> {
        let warning_kinds = vec![kind];
        let warning_digest = compose_eligibility_warning_kinds_digest(&warning_kinds)?;
        Ok(Self {
            warning_kinds,
            warning_digest,
        })
    }

    pub fn warning_kinds(&self) -> &[ProjectionConsumptionWarningKind] {
        &self.warning_kinds
    }

    pub fn warning_digest(&self) -> &str {
        &self.warning_digest
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProjectionConsumptionEligibilityCounters {
    requested_fact_count: usize,
    evaluated_fact_count: usize,
    warning_count: usize,
    denied_count: usize,
    deferred_count: usize,
    source_mismatch_count: usize,
}

impl ProjectionConsumptionEligibilityCounters {
    pub fn requested_fact_count(&self) -> usize {
        self.requested_fact_count
    }

    pub fn evaluated_fact_count(&self) -> usize {
        self.evaluated_fact_count
    }

    pub fn warning_count(&self) -> usize {
        self.warning_count
    }

    pub fn denied_count(&self) -> usize {
        self.denied_count
    }

    pub fn deferred_count(&self) -> usize {
        self.deferred_count
    }

    pub fn source_mismatch_count(&self) -> usize {
        self.source_mismatch_count
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionEligibilityTrace {
    rule_label: &'static str,
    explanation: &'static str,
}

impl ProjectionConsumptionEligibilityTrace {
    pub fn rule_label(&self) -> &'static str {
        self.rule_label
    }

    pub fn explanation(&self) -> &'static str {
        self.explanation
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedProjectionConsumption {
    declaration: ProjectionConsumptionDeclaration,
    declaration_digest: String,
    query_digest: String,
    basis_digest: String,
    result_shape_digest: String,
    authorized_projection_identity: String,
    warning_kinds: Vec<ProjectionConsumptionWarningKind>,
    counters: ProjectionConsumptionEligibilityCounters,
    trace: ProjectionConsumptionEligibilityTrace,
    eligibility_digest: String,
}

impl AdmittedProjectionConsumption {
    pub(crate) fn declaration(&self) -> &ProjectionConsumptionDeclaration {
        &self.declaration
    }

    pub fn eligibility_digest(&self) -> &str {
        &self.eligibility_digest
    }

    pub fn trace(&self) -> &ProjectionConsumptionEligibilityTrace {
        &self.trace
    }

    pub(crate) fn warning_kinds(&self) -> &[ProjectionConsumptionWarningKind] {
        &self.warning_kinds
    }

    pub fn bind_contract(
        &self,
    ) -> Result<MaterializedProjectionContract, IdentityDerivationDenial> {
        Ok(super::contracts::bind_materialized_projection_contract(
            self,
        )?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeniedProjectionConsumption {
    declaration_digest: String,
    reason: ProjectionConsumptionDenialReason,
    counters: ProjectionConsumptionEligibilityCounters,
    trace: ProjectionConsumptionEligibilityTrace,
    failure_digest: String,
}

impl DeniedProjectionConsumption {
    pub fn reason(&self) -> &ProjectionConsumptionDenialReason {
        &self.reason
    }

    pub fn counters(&self) -> &ProjectionConsumptionEligibilityCounters {
        &self.counters
    }

    pub fn trace(&self) -> &ProjectionConsumptionEligibilityTrace {
        &self.trace
    }

    pub(crate) fn failure_digest(&self) -> &str {
        &self.failure_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeferredProjectionConsumption {
    declaration_digest: String,
    source_family: ProjectionSourceFamily,
    reason: DeferredProjectionConsumptionReason,
    counters: ProjectionConsumptionEligibilityCounters,
    trace: ProjectionConsumptionEligibilityTrace,
    failure_digest: String,
}

impl DeferredProjectionConsumption {
    pub fn source_family(&self) -> ProjectionSourceFamily {
        self.source_family
    }

    pub fn reason(&self) -> &DeferredProjectionConsumptionReason {
        &self.reason
    }

    pub fn trace(&self) -> &ProjectionConsumptionEligibilityTrace {
        &self.trace
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceMismatchedProjectionConsumption {
    declaration_digest: String,
    source_family: ProjectionSourceFamily,
    requested_fact_kind: ProjectionFactKind,
    counters: ProjectionConsumptionEligibilityCounters,
    trace: ProjectionConsumptionEligibilityTrace,
    failure_digest: String,
}

impl SourceMismatchedProjectionConsumption {
    pub fn requested_fact_kind(&self) -> ProjectionFactKind {
        self.requested_fact_kind
    }

    pub fn source_family(&self) -> ProjectionSourceFamily {
        self.source_family
    }

    pub fn trace(&self) -> &ProjectionConsumptionEligibilityTrace {
        &self.trace
    }

    pub(crate) fn failure_digest(&self) -> &str {
        &self.failure_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectionConsumptionEligibility {
    Admitted(AdmittedProjectionConsumption),
    AdmittedWithWarnings(AdmittedProjectionConsumption, ProjectionConsumptionWarnings),
    Denied(DeniedProjectionConsumption),
    Deferred(DeferredProjectionConsumption),
    SourceMismatch(SourceMismatchedProjectionConsumption),
}

pub fn evaluate_projection_consumption_eligibility(
    declaration: &ProjectionConsumptionDeclaration,
) -> Result<ProjectionConsumptionEligibility, IdentityDerivationDenial> {
    let mut counters = ProjectionConsumptionEligibilityCounters {
        requested_fact_count: declaration.requested().requested_count(),
        ..ProjectionConsumptionEligibilityCounters::default()
    };
    let mut warnings = Vec::new();

    for requested_fact in declaration.requested().requested() {
        counters.evaluated_fact_count += 1;
        match admit_requested_fact(declaration, requested_fact, &mut counters)? {
            RequestedFactAdmission::Admitted(Some(warning)) => warnings.push(warning),
            RequestedFactAdmission::Admitted(None) => {}
            RequestedFactAdmission::Terminal(terminal) => return Ok(terminal),
        }
    }

    Ok(admitted_projection_consumption(
        declaration,
        counters,
        warnings,
    )?)
}

enum RequestedFactAdmission {
    Admitted(Option<ProjectionConsumptionWarningKind>),
    Terminal(ProjectionConsumptionEligibility),
}

fn admit_requested_fact(
    declaration: &ProjectionConsumptionDeclaration,
    requested_fact: &ProjectionFactRequest,
    counters: &mut ProjectionConsumptionEligibilityCounters,
) -> Result<RequestedFactAdmission, IdentityDerivationDenial> {
    let warning = match support_for_kind(declaration.source(), requested_fact.kind()) {
        ProjectionConsumptionSupportPosture::Admitted => None,
        ProjectionConsumptionSupportPosture::AdmittedWithWarnings(kind) => {
            counters.warning_count += 1;
            Some(kind)
        }
        ProjectionConsumptionSupportPosture::Deferred(reason) => {
            counters.deferred_count = 1;
            return Ok(RequestedFactAdmission::Terminal(
                deferred_projection_consumption(declaration, reason, counters.clone())?,
            ));
        }
        ProjectionConsumptionSupportPosture::SourceMismatch => {
            counters.source_mismatch_count = 1;
            return Ok(RequestedFactAdmission::Terminal(
                source_mismatched_projection_consumption(
                    declaration,
                    requested_fact.kind(),
                    counters.clone(),
                )?,
            ));
        }
    };
    if let Some(denial) = visibility_denial(declaration, requested_fact) {
        counters.denied_count = 1;
        return Ok(RequestedFactAdmission::Terminal(
            denied_projection_consumption(declaration, denial, counters.clone())?,
        ));
    }
    Ok(RequestedFactAdmission::Admitted(warning))
}
