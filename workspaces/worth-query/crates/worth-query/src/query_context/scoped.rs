use crate::basis_lifecycle::{
    BasisFamily, BasisIntentDenial, BasisLifecycleDeclarationError, BasisLifecycleIntentDraft,
    DeniedBasisCapability, RawBasisIntent, ScopedMaterializationBasis, ScopedObservationBasis,
};

use super::{
    admit_legacy_query_basis_context, attach_legacy_query_basis_metadata,
    bind_legacy_query_basis_context, build_legacy_query_basis_result_bundle,
    execute_legacy_query_basis_context, AdmittedQueryBasisContext, HistoricalAdmissionClass,
    HistoricalMaterializationCostClass, QueryBasisContextRequest, QueryBasisMetadata,
    QueryBasisResultBundle, QueryContextAdmissionError, QueryContextAdmissionFailureClass,
    QueryContextBindingSource, QueryContextBudgetClass, QueryContextCostClass,
    QueryContextCounters, QueryContextDriftOutcome, QueryContextExecutionArtifact,
    QueryContextFamily, QueryContextPredictionDriftOutcome, QueryContextPredictionReport,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScopedQueryContextAdmissionError {
    Intent(BasisIntentDenial),
    Eligibility(DeniedBasisCapability),
    Context(QueryContextAdmissionError),
}

impl ScopedQueryContextAdmissionError {
    pub fn intent_denial(&self) -> Option<&BasisIntentDenial> {
        match self {
            Self::Intent(denial) => Some(denial),
            Self::Eligibility(_) | Self::Context(_) => None,
        }
    }

    pub fn eligibility_denial(&self) -> Option<&DeniedBasisCapability> {
        match self {
            Self::Eligibility(denial) => Some(denial),
            Self::Intent(_) | Self::Context(_) => None,
        }
    }

    pub fn context_error(&self) -> Option<&QueryContextAdmissionError> {
        match self {
            Self::Context(error) => Some(error),
            Self::Intent(_) | Self::Eligibility(_) => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopedObservationQueryBasisContext {
    scoped_basis: ScopedObservationBasis,
    context: AdmittedQueryBasisContext,
}

impl ScopedObservationQueryBasisContext {
    pub fn scoped_basis(&self) -> &ScopedObservationBasis {
        &self.scoped_basis
    }

    pub(crate) fn context(&self) -> &AdmittedQueryBasisContext {
        &self.context
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopedMaterializationQueryBasisContext {
    scoped_basis: ScopedMaterializationBasis,
    context: AdmittedQueryBasisContext,
}

impl ScopedMaterializationQueryBasisContext {
    pub fn scoped_basis(&self) -> &ScopedMaterializationBasis {
        &self.scoped_basis
    }

    pub(crate) fn context(&self) -> &AdmittedQueryBasisContext {
        &self.context
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScopedQueryBasisContext {
    Observation(ScopedObservationQueryBasisContext),
    Materialization(ScopedMaterializationQueryBasisContext),
}

impl ScopedQueryBasisContext {
    pub fn family(&self) -> &QueryContextFamily {
        self.context().family()
    }

    pub fn query_digest(&self) -> &str {
        self.context().query_digest()
    }

    pub fn basis_digest(&self) -> &str {
        self.context().basis_digest()
    }

    pub fn basis_authority_family(&self) -> &crate::basis::BasisAuthorityFamily {
        self.context().basis_authority_family()
    }

    pub fn declared_basis_label(&self) -> &str {
        self.context().declared_basis_label()
    }

    pub(crate) fn admits_runtime_snapshot(
        &self,
        snapshot: &crate::memory_workspace::WorthQuerySnapshotIdentity,
    ) -> bool {
        self.context().admits_runtime_snapshot(snapshot)
    }

    pub fn cost_class(&self) -> &QueryContextCostClass {
        self.context().cost_class()
    }

    pub fn budget_class(&self) -> &QueryContextBudgetClass {
        self.context().budget_class()
    }

    pub fn counters(&self) -> &QueryContextCounters {
        self.context().counters()
    }

    pub fn drift_outcome(&self) -> &QueryContextDriftOutcome {
        self.context().drift_outcome()
    }

    pub fn historical_admission_class(&self) -> Option<&HistoricalAdmissionClass> {
        self.context().historical_admission_class()
    }

    pub fn historical_materialization_cost_class(
        &self,
    ) -> Option<&HistoricalMaterializationCostClass> {
        self.context().historical_materialization_cost_class()
    }

    pub fn materialization_path_identity_source(&self) -> Option<&str> {
        self.context().materialization_path_identity_source()
    }

    pub fn preview_provenance_identity_source(&self) -> Option<&str> {
        self.context().preview_provenance_identity_source()
    }

    pub fn prediction_report(&self) -> Option<&QueryContextPredictionReport> {
        self.context().prediction_report()
    }

    pub fn prediction_drift_outcome(&self) -> Option<&QueryContextPredictionDriftOutcome> {
        self.context().prediction_drift_outcome()
    }

    pub(crate) fn context(&self) -> &AdmittedQueryBasisContext {
        match self {
            Self::Observation(scoped) => scoped.context(),
            Self::Materialization(scoped) => scoped.context(),
        }
    }

    pub(crate) fn predicted_result_shape_width(&self) -> usize {
        self.context().predicted_result_shape_width()
    }
}

pub fn admit_query_basis_context(
    declaration: BasisLifecycleIntentDraft,
    source: QueryContextBindingSource<'_>,
) -> Result<ScopedQueryBasisContext, ScopedQueryContextAdmissionError> {
    let raw = declaration.into_raw();
    let request = legacy_request_for_declaration(&raw)?;
    let scoped_basis = scope_query_context_declaration(BasisLifecycleIntentDraft::new(raw))?;
    let binding = bind_legacy_query_basis_context(request, source)
        .map_err(ScopedQueryContextAdmissionError::Context)?;
    let context = admit_legacy_query_basis_context(binding)
        .map_err(ScopedQueryContextAdmissionError::Context)?;
    ensure_scoped_context_coherence(&scoped_basis, &context)
        .map_err(ScopedQueryContextAdmissionError::Context)?;

    Ok(match scoped_basis {
        QueryContextScopedBasis::Observation(scoped_basis) => {
            ScopedQueryBasisContext::Observation(ScopedObservationQueryBasisContext {
                scoped_basis,
                context,
            })
        }
        QueryContextScopedBasis::Materialization(scoped_basis) => {
            ScopedQueryBasisContext::Materialization(ScopedMaterializationQueryBasisContext {
                scoped_basis,
                context,
            })
        }
    })
}

pub fn execute_query_basis_context(
    context: &ScopedQueryBasisContext,
) -> Result<QueryContextExecutionArtifact, QueryContextAdmissionError> {
    execute_legacy_query_basis_context(context.context())
}

pub fn attach_query_basis_metadata(
    context: &ScopedQueryBasisContext,
    execution: &QueryContextExecutionArtifact,
) -> Result<QueryBasisMetadata, QueryContextAdmissionError> {
    attach_legacy_query_basis_metadata(context.context(), execution)
}

pub fn build_query_basis_result_bundle(
    context: &ScopedQueryBasisContext,
    execution: QueryContextExecutionArtifact,
) -> Result<QueryBasisResultBundle, QueryContextAdmissionError> {
    build_legacy_query_basis_result_bundle(context, execution)
}

#[cfg(test)]
pub fn execute_and_build_query_basis_result_bundle(
    context: &ScopedQueryBasisContext,
) -> Result<QueryBasisResultBundle, QueryContextAdmissionError> {
    let execution = execute_query_basis_context(context)?;
    build_query_basis_result_bundle(context, execution)
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum QueryContextScopedBasis {
    Observation(ScopedObservationBasis),
    Materialization(ScopedMaterializationBasis),
}

fn scope_query_context_declaration(
    declaration: BasisLifecycleIntentDraft,
) -> Result<QueryContextScopedBasis, ScopedQueryContextAdmissionError> {
    let raw = declaration.into_raw();
    let materialization = matches!(
        raw,
        RawBasisIntent::HistoricalSnapshot { .. } | RawBasisIntent::HistoricalCommit { .. }
    );
    let declaration = BasisLifecycleIntentDraft::new(raw);
    if materialization {
        declaration
            .materialize()
            .map(QueryContextScopedBasis::Materialization)
            .map_err(map_lifecycle_error)
    } else {
        declaration
            .observe()
            .map(QueryContextScopedBasis::Observation)
            .map_err(map_lifecycle_error)
    }
}

fn legacy_request_for_declaration(
    raw: &RawBasisIntent,
) -> Result<QueryBasisContextRequest, ScopedQueryContextAdmissionError> {
    let request = match raw {
        RawBasisIntent::CurrentHead => QueryBasisContextRequest::current_branch_head(),
        RawBasisIntent::BranchHead {
            branch_identity, ..
        } => QueryBasisContextRequest::branch_head(branch_identity),
        RawBasisIntent::HistoricalSnapshot {
            snapshot_identity, ..
        } => QueryBasisContextRequest::historical_snapshot(snapshot_identity),
        RawBasisIntent::HistoricalCommit {
            commit_identity, ..
        } => QueryBasisContextRequest::historical_commit(commit_identity),
        RawBasisIntent::PreviewDerived {
            preview_identity, ..
        } => QueryBasisContextRequest::preview_derived_historical(preview_identity),
        _ => return Err(unsupported_query_context_family()),
    };
    Ok(request)
}

fn map_lifecycle_error(error: BasisLifecycleDeclarationError) -> ScopedQueryContextAdmissionError {
    match error {
        BasisLifecycleDeclarationError::Intent(denial) => {
            ScopedQueryContextAdmissionError::Intent(denial)
        }
        BasisLifecycleDeclarationError::Eligibility(denial) => {
            ScopedQueryContextAdmissionError::Eligibility(denial)
        }
    }
}

fn unsupported_query_context_family() -> ScopedQueryContextAdmissionError {
    ScopedQueryContextAdmissionError::Context(QueryContextAdmissionError::new(
        QueryContextAdmissionFailureClass::UnsupportedQueryContextBasisFamily,
        "query-context execution supports current head, branch head, historical snapshot, historical commit, and preview-derived declarations",
        QueryContextCounters::for_denial(false, false),
    ))
}

fn ensure_scoped_context_coherence(
    scoped_basis: &QueryContextScopedBasis,
    context: &AdmittedQueryBasisContext,
) -> Result<(), QueryContextAdmissionError> {
    let scoped_family = match scoped_basis {
        QueryContextScopedBasis::Observation(scoped) => scoped.family(),
        QueryContextScopedBasis::Materialization(scoped) => scoped.family(),
    };
    if scoped_family == expected_basis_family(context.family()) {
        return Ok(());
    }
    Err(QueryContextAdmissionError::new(
        QueryContextAdmissionFailureClass::BasisSubstitutionForbidden,
        "scoped query-context admission requires lifecycle family parity with its lowered query context",
        QueryContextCounters::for_denial(false, true),
    ))
}

fn expected_basis_family(family: &QueryContextFamily) -> BasisFamily {
    match family {
        QueryContextFamily::CurrentBranchHead => BasisFamily::CurrentHead,
        QueryContextFamily::BranchHead => BasisFamily::BranchHead,
        QueryContextFamily::HistoricalSnapshot => BasisFamily::HistoricalSnapshot,
        QueryContextFamily::HistoricalCommit => BasisFamily::HistoricalCommit,
        QueryContextFamily::PreviewDerivedHistorical => BasisFamily::PreviewDerived,
        QueryContextFamily::DiffComparison => BasisFamily::CurrentHead,
    }
}

#[cfg(test)]
pub(crate) fn admit_and_scope_legacy_query_basis_context_for_test(
    binding: super::QueryBasisContextBinding,
) -> Result<ScopedQueryBasisContext, QueryContextAdmissionError> {
    let declaration = match binding.request().family() {
        QueryContextFamily::CurrentBranchHead => {
            crate::basis_lifecycle::basis_lifecycle().current_head()
        }
        QueryContextFamily::BranchHead => crate::basis_lifecycle::basis_lifecycle()
            .branch_head(binding.request().declared_basis_label(), true),
        QueryContextFamily::HistoricalSnapshot => crate::basis_lifecycle::basis_lifecycle()
            .historical_snapshot(binding.request().declared_basis_label(), true),
        QueryContextFamily::HistoricalCommit => crate::basis_lifecycle::basis_lifecycle()
            .historical_commit(binding.request().declared_basis_label(), true),
        QueryContextFamily::PreviewDerivedHistorical => crate::basis_lifecycle::basis_lifecycle()
            .preview_derived(
                binding.request().declared_basis_label(),
                "legacy-test-source",
            ),
        QueryContextFamily::DiffComparison => {
            return Err(QueryContextAdmissionError::new(
                QueryContextAdmissionFailureClass::UnsupportedQueryContextBasisFamily,
                "diff comparison is not a standalone query basis context",
                QueryContextCounters::for_denial(false, false),
            ))
        }
    };
    let scoped_basis = scope_query_context_declaration(declaration).map_err(|_| {
        QueryContextAdmissionError::new(
            QueryContextAdmissionFailureClass::UnsupportedQueryContextBasisFamily,
            "test-only legacy context could not enter the canonical scoped lifecycle",
            QueryContextCounters::for_denial(false, false),
        )
    })?;
    let context = admit_legacy_query_basis_context(binding)?;
    ensure_scoped_context_coherence(&scoped_basis, &context)?;
    Ok(match scoped_basis {
        QueryContextScopedBasis::Observation(scoped_basis) => {
            ScopedQueryBasisContext::Observation(ScopedObservationQueryBasisContext {
                scoped_basis,
                context,
            })
        }
        QueryContextScopedBasis::Materialization(scoped_basis) => {
            ScopedQueryBasisContext::Materialization(ScopedMaterializationQueryBasisContext {
                scoped_basis,
                context,
            })
        }
    })
}
