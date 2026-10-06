//! Fresh installed Query permission before any graph plan or source read.

use worth_query_declaration::facade::application_query::ApplicationQueryDisclosurePosture;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryAuthorization;

use super::*;
use crate::domain_computation::authorization::{
    AdmittedQueryAuthorizationStop, WorthQueryRetainedAuthorizationDecisionFacts,
};
use crate::domain_computation::provider_session::WorthQueryPreparedQuerySessionIdentity;

mod principal_dependency;

/// The same reserved session and admitted Ability decision enter either a
/// Clean exact-row proof or the ordinary graph continuation. This is not a
/// graph-read session and does not itself authorize source disclosure.
pub(super) struct PreparedRetainedQueryPermission<'a, Schema> {
    source: crate::domain_computation::primary_graph::application_query::PreparedApplicationQuerySourceReadmission<'a, Schema>,
    session: WorthQueryPreparedQuerySessionIdentity,
    authorization: WorthQueryRetainedAuthorizationDecisionFacts,
}

impl<'a, Schema> PreparedRetainedQueryPermission<'a, Schema> {
    pub(super) fn into_parts(
        self,
    ) -> (
        crate::domain_computation::primary_graph::application_query::PreparedApplicationQuerySourceReadmission<'a, Schema>,
        WorthQueryPreparedQuerySessionIdentity,
        WorthQueryRetainedAuthorizationDecisionFacts,
    ){
        (self.source, self.session, self.authorization)
    }
}

pub(super) fn prepare_retained_query_permission<'a, Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    installed: &WorthQueryInstalledApplicationQueryBinding<Schema, SourceBinding<Schema, Binding>>,
    principal: &WorthQueryAuthenticatedPrincipal<
        Schema,
        BoundPrincipal<Schema, Binding>,
        BoundPrincipalIdentity<Schema, Binding>,
    >,
    scope: &WorthQueryApplicationEntityIdentity<Schema, BoundScope<Schema, Binding>>,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    source: crate::domain_computation::primary_graph::application_query::PreparedApplicationQuerySourceReadmission<'a, Schema>,
    request: &WorthQueryRequestScope,
    admission: &mut InvalidationEditAdmission,
) -> Result<PreparedRetainedQueryPermission<'a, Schema>, ProducerExecutionStop>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    let query = installed.query();
    let query_name_bytes = u64::try_from(query.name().len()).map_err(|_| {
        denial(
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
            Binding::IDENTITY,
        )
    })?;
    let selected_observation = selected.product().observation();
    let basis_work = u64::try_from(
        worth_runtime_world::facade::CurrentProductHead::comparison_work_bound(
            selected_observation,
        ),
    )
    .map_err(|_| work_denial::<Schema, Binding>())?;
    let native_branch_bytes = u64::try_from(
        selected
            .application_basis()
            .snapshot_handle()
            .branch_id()
            .0
            .len(),
    )
    .map_err(|_| work_denial::<Schema, Binding>())?;
    let entry_work = query_name_bytes
        .checked_add(8)
        .and_then(|n| n.checked_add(basis_work))
        .and_then(|n| n.checked_add(native_branch_bytes))
        .and_then(|n| n.checked_add(32))
        .ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                Binding::IDENTITY,
            )
        })?;
    admission.charge_external_work(entry_work).map_err(|_| {
        denial(
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
            Binding::IDENTITY,
        )
    })?;
    admission
        .admit_read_scratch(query_name_bytes)
        .map_err(|stop| request_admission_denied(Binding::IDENTITY, stop))?;
    let same_query = source.query_identity() == query.identity();
    let same_basis = source.selected_basis().is_some_and(|(product, snapshot)| {
        product.observation() == selected_observation
            && snapshot.branch_id() == selected.application_basis().snapshot_handle().branch_id()
            && snapshot.version_id() == selected.application_basis().snapshot_handle().version_id()
    });
    if !same_query || !same_basis {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
            Binding::IDENTITY,
        )
        .into());
    }
    match request.interruption() {
        Some(worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled) => {
            return Err(denial(WorthQueryOutputDemandDenialKind::Cancelled, Binding::IDENTITY).into());
        }
        Some(worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::DeadlineExceeded) => {
            return Err(denial(WorthQueryOutputDemandDenialKind::TimedOut, Binding::IDENTITY).into());
        }
        None => {}
    }
    if !selected.application_basis().selected_program_inspected() {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
            Binding::IDENTITY,
        )
        .into());
    }
    match query.disclosure().posture() {
        ApplicationQueryDisclosurePosture::Public => {
            let graph = runtime.runtime.primary_graph().ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    Binding::IDENTITY,
                )
            })?;
            crate::domain_computation::authorization::application_disclosure::compile_disclosure_contract(
                query,
                &graph.layout,
            )
            .map_err(|contract| {
                query_admission_denied(
                    Binding::IDENTITY,
                    crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial::new(
                        crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid,
                        contract.subject(),
                    ),
                )
            })?;
        }
        ApplicationQueryDisclosurePosture::Governed => {
            return Err(query_admission_denied(
                Binding::IDENTITY,
                crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial::new(
                    crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenialKind::DisclosureGovernanceRequired,
                    query.name(),
                ),
            ));
        }
        ApplicationQueryDisclosurePosture::InstalledPolicyRequired => {
            return Err(query_admission_denied(
                Binding::IDENTITY,
                crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial::new(
                    crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid,
                    query.name(),
                ),
            ));
        }
    }
    admission.charge_external_work(2).map_err(|_| {
        denial(
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
            Binding::IDENTITY,
        )
    })?;
    let session = WorthQueryPreparedQuerySessionIdentity::reserve()
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let principal_currentness = principal_dependency::capture::<Schema, Binding>(
        runtime,
        principal,
        session.identity(),
        admission,
    )?;
    let authorization = match query.authorization() {
        WorthQueryInstalledApplicationQueryAuthorization::Public => {
            WorthQueryRetainedAuthorizationDecisionFacts::principal(principal_currentness)
        }
        WorthQueryInstalledApplicationQueryAuthorization::Ability(requirement) => {
            let snapshot = selected.application_basis().snapshot_handle();
            let branch_bytes = u64::try_from(snapshot.branch_id().0.len()).map_err(|_| {
                denial(
                    WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                    Binding::IDENTITY,
                )
            })?;
            admission
                .charge_external_work(branch_bytes.saturating_add(1))
                .map_err(|_| {
                    denial(
                        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                        Binding::IDENTITY,
                    )
                })?;
            admission
                .admit_read_scratch(branch_bytes)
                .map_err(|stop| request_admission_denied(Binding::IDENTITY, stop))?;
            let access = WorthQueryApplicationQueryAccessContext::new(principal, scope);
            let decisions = runtime
                .primary_provider
                .graph
                .with_runtime(|relational| {
                    runtime.observe_query_authorization_requirement_admitted(
                        session.identity(),
                        relational,
                        snapshot.clone(),
                        query,
                        &access,
                        requirement,
                        admission,
                    )
                })
                .map_err(|stop| match stop {
                    AdmittedQueryAuthorizationStop::Authorization(error) => {
                        request_admission_denied(Binding::IDENTITY, error)
                    }
                    AdmittedQueryAuthorizationStop::Preparation(error) => {
                        request_admission_denied(Binding::IDENTITY, error)
                    }
                    AdmittedQueryAuthorizationStop::AccountingOverflow => denial(
                        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                        Binding::IDENTITY,
                    )
                    .into(),
                })?;
            WorthQueryRetainedAuthorizationDecisionFacts::abilities(
                principal_currentness,
                decisions,
            )
        }
    };
    Ok(PreparedRetainedQueryPermission {
        source,
        session,
        authorization,
    })
}
