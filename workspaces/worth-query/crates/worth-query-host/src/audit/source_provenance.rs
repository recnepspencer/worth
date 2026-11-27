//! "Which operation and inputs produced this field?" in one session call:
//! materialized source-effect inspection of an authored operation's original
//! commit receipt, with both inspection capabilities admitted for the
//! session principal immediately before the inspection consumes them.
//!
//! This is `request.inspect_source::<Disclosure>(receipt).why_performed()
//! .materialize(source_effects()).execute_governed(metadata, diagnostic)`;
//! the disclosure's metadata and diagnostic contracts decide what is shown.
use worth_execution::ExecutionAllocationPolicy;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::ApplicationSourceDisclosureBinding,
    application_query::{ApplicationQueryBinding, ApplicationQueryScopeBinding},
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationSchema, ApplicationStructuredValueBinding,
    },
};
use worth_query_execution::facade::{
    application_inspection::{
        WorthQueryApplicationInspectionDenial, WorthQueryApplicationMaterializationRequest,
        WorthQueryMaterializedFamilyOutcome, WorthQueryMaterializedPerformedSourceInspection,
        WorthQueryMaterializedSourceField,
    },
    primary_graph::WorthQueryApplicationCommitReceipt,
};
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationRequestQueryDenial, WorthQueryGovernedReadAccess,
};
use worth_relational::facade::history::RelationalCommitReceipt;

use crate::session::WorthQueryHostSession;

type DisclosureScope<S, D> = <<<D as ApplicationSourceDisclosureBinding<S>>::Disclosure as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope;

/// The two capabilities a source provenance read admits: the disclosure's
/// metadata contract and its diagnostic (field value) contract.
pub struct WorthQuerySourceProvenanceRequest<Schema, MC, MO, MI, DC, DO, DI> {
    pub metadata: WorthQueryGovernedReadAccess<Schema, MC, MO, MI>,
    pub diagnostic: WorthQueryGovernedReadAccess<Schema, DC, DO, DI>,
}

/// Why a source provenance read refused.
#[derive(Debug)]
pub enum WorthQuerySourceProvenanceDenial {
    /// The receipt was committed on another product branch than the
    /// session's; an active-branch session never explains a preview's write.
    ForeignBranch,
    /// A capability or the principal was refused at admission.
    Admission(WorthQueryApplicationRequestQueryDenial),
    /// The inspection itself refused (stale incarnation, governance, request,
    /// or a disclosure shape field diagnostics do not support: see
    /// [`Self::is_unsupported_field_shape`]).
    Inspection(WorthQueryApplicationInspectionDenial),
}

impl WorthQuerySourceProvenanceDenial {
    /// Whether field-level diagnostics refused the source disclosure's query
    /// shape, rather than a grant, the principal or a resource limit.
    ///
    /// Field diagnostics admit only a disclosure query rooted at its scoped
    /// record: no root path, orbit or component selection, continuation or
    /// live mode. Every other shape answers `true` here and renders an
    /// `Unsupported` summary.
    ///
    /// The common case is a creation scoped by its parent (a Site-scoped
    /// operation creating a Dossier): a source disclosure is scoped like its
    /// operation, so it can disclose the child only through a root path,
    /// which selects its records by traversal at the original commit. The
    /// governed receipt stays readable through `reference_only()` inspection,
    /// and the child's changed aspects through `record_history`.
    pub fn is_unsupported_field_shape(&self) -> bool {
        use worth_query_execution::facade::application_inspection::{
            DiagnosticComparisonDenial as Comparison,
            WorthQueryApplicationInspectionOwnerRefusal as Owner,
        };
        let Self::Inspection(cause) = self else {
            return false;
        };
        matches!(
            cause.owner_refusal(),
            Some(
                Owner::DiagnosticContractUnsupportedShape
                    | Owner::DiagnosticAdmission(Comparison::UnsupportedShape)
            )
        )
    }
}

/// The original source effects of one authored operation.
pub struct WorthQuerySourceProvenance {
    inspection: WorthQueryMaterializedPerformedSourceInspection,
}

impl std::fmt::Debug for WorthQuerySourceProvenance {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQuerySourceProvenance")
            .field("report", &self.inspection.report_identity())
            .field("source_effects", &self.source_effects_outcome())
            .finish_non_exhaustive()
    }
}

impl WorthQuerySourceProvenance {
    /// The authored operation's original commit.
    pub fn source_commit(&self) -> &RelationalCommitReceipt {
        self.inspection.source_commit()
    }
    /// `Provided` when the field effects below were materialized; otherwise
    /// why they were not (`UnsupportedSourceProjectionShape` when the root
    /// disclosed record's projection reaches beyond its own fields). A
    /// root-path disclosure refuses instead: see
    /// [`WorthQuerySourceProvenanceDenial::is_unsupported_field_shape`].
    pub fn source_effects_outcome(&self) -> WorthQueryMaterializedFamilyOutcome {
        self.inspection.materialization().source_outcome()
    }
    /// Every materialized field: its result path, effect and written value.
    pub fn fields(&self) -> impl Iterator<Item = &WorthQueryMaterializedSourceField> {
        self.inspection
            .materialization()
            .source_effects()
            .into_iter()
            .flat_map(|effects| effects.fields())
    }
    /// The field at `result_path`, if the operation's projection has one.
    pub fn field(&self, result_path: &str) -> Option<&WorthQueryMaterializedSourceField> {
        self.fields()
            .find(|field| field.result_path() == result_path)
    }
    /// The full materialized inspection, with evidence references.
    pub fn inspection(&self) -> &WorthQueryMaterializedPerformedSourceInspection {
        &self.inspection
    }
    pub fn into_inspection(self) -> WorthQueryMaterializedPerformedSourceInspection {
        self.inspection
    }
}

/// A request that admits both inspection capabilities of `Disclosure`.
pub trait WorthQuerySourceProvenanceAccess<Schema: ApplicationSchema, Disclosure> {
    #[doc(hidden)]
    fn inspect_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
        source: &WorthQueryApplicationCommitReceipt,
    ) -> Result<WorthQuerySourceProvenance, WorthQuerySourceProvenanceDenial>;
}

impl<Schema, Disclosure, MC, MO, MI, DC, DO, DI>
    WorthQuerySourceProvenanceAccess<Schema, Disclosure>
    for WorthQuerySourceProvenanceRequest<Schema, MC, MO, MI, DC, DO, DI>
where
    Schema: ApplicationSchema,
    Disclosure: ApplicationSourceDisclosureBinding<Schema>,
    MO: ApplicationOperationMarkerIdentity<Schema> + 'static,
    MO::InputBinding: ApplicationStructuredValueBinding<Value = MI>,
    MI: ApplicationCapabilityRequest<Schema, MC, Scope = DisclosureScope<Schema, Disclosure>>
        + Clone
        + 'static,
    DO: ApplicationOperationMarkerIdentity<Schema> + 'static,
    DO::InputBinding: ApplicationStructuredValueBinding<Value = DI>,
    DI: ApplicationCapabilityRequest<Schema, DC, Scope = DisclosureScope<Schema, Disclosure>>
        + Clone
        + 'static,
{
    fn inspect_in_session<Program>(
        self,
        session: &WorthQueryHostSession<'_, Schema, Program>,
        source: &WorthQueryApplicationCommitReceipt,
    ) -> Result<WorthQuerySourceProvenance, WorthQuerySourceProvenanceDenial> {
        use WorthQuerySourceProvenanceDenial as Denial;
        if source.product_branch() != session.branch() {
            return Err(Denial::ForeignBranch);
        }
        let scope = session.fresh_scope();
        let principal = session.call_principal();
        let request = session.request_as(&principal, &scope);
        let (metadata, diagnostic) = session
            .with_funded_advancement(&scope, |phase| {
                let metadata = request
                    .admit_read_access_in_advancement::<Disclosure::Disclosure, _, _, _>(
                        &phase,
                        &self.metadata,
                    )?;
                let diagnostic = request
                    .admit_read_access_in_advancement::<Disclosure::Disclosure, _, _, _>(
                        &phase,
                        &self.diagnostic,
                    )?;
                Ok((metadata, diagnostic))
            })
            .map_err(|cause| {
                Denial::Admission(WorthQueryApplicationRequestQueryDenial::ExecutionRequest(
                    cause,
                ))
            })?
            .map_err(Denial::Admission)?;
        let lease = session.open_lease(&scope).map_err(|cause| {
            Denial::Admission(WorthQueryApplicationRequestQueryDenial::ExecutionRequest(
                cause,
            ))
        })?;
        request
            .inspect_source::<Disclosure>(source)
            .why_performed()
            .materialize(WorthQueryApplicationMaterializationRequest::source_effects())
            .with_allocation_policy(ExecutionAllocationPolicy::Execution(&lease))
            .execute_governed(metadata, diagnostic)
            .map(|inspection| WorthQuerySourceProvenance { inspection })
            .map_err(Denial::Inspection)
    }
}

impl<Schema: ApplicationSchema, Program> WorthQueryHostSession<'_, Schema, Program> {
    /// Materializes what the authored operation behind `source` wrote, field
    /// by field: `session.source_provenance::<Disclosure>(&receipt, request)`.
    /// The receipt must belong to the session's branch (`ForeignBranch`
    /// otherwise). Both capabilities are admitted for the session principal at
    /// that branch's head and consumed by this one inspection, which runs under
    /// the session's execution policy.
    pub fn source_provenance<Disclosure>(
        &self,
        source: &WorthQueryApplicationCommitReceipt,
        request: impl WorthQuerySourceProvenanceAccess<Schema, Disclosure>,
    ) -> Result<WorthQuerySourceProvenance, WorthQuerySourceProvenanceDenial> {
        request.inspect_in_session(self, source)
    }
}
