//! Graph continuation of the exact permission prepared before a Clean probe.

use std::fmt::Write;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::*;
use crate::domain_computation::primary_graph::application_query::{
    admission::permission::PreparedApplicationQueryPermission,
    basis::ensure_selected_read_indexes_admitted, disclosure::WorthQueryApplicationQueryGovernance,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::provider_session::WorthQueryAdmittedQuerySessionStartStop;
use crate::domain_computation::provider_session::WorthQueryPreparedQuerySessionIdentity;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Consume exactly the same fresh principal, scope, selected Product and
    /// reserved session that the Clean probe used. Only a genuine disclosure
    /// need reaches this graph/index preparation.
    pub(in crate::domain_computation::primary_graph) fn finish_prepared_application_query_permission<
        'a,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &'a self,
        permission: PreparedApplicationQueryPermission<
            'a,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        WorthQueryAdmittedApplicationQueryPlan<
            'a,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        let PreparedApplicationQueryPermission {
            issuer,
            query,
            access,
            parameters,
            controls,
            basis,
            security_product,
            session,
            disclosure,
            authorization,
            authorization_work,
            selected_access,
            shape,
        } = permission;
        admission.charge_external_work(1).map_err(|_| {
            denial(
                WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded,
                String::new(),
            )
        })?;
        if !std::ptr::eq(self, issuer) {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis,
                query.name(),
            ));
        }
        let graph = self.runtime.primary_graph().ok_or_else(|| {
            denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StaleScope,
                query.name(),
            )
        })?;
        let prepared = self.prepare_application_query_graph_work_admitted(
            query,
            &parameters,
            &controls,
            shape,
            graph,
            disclosure,
            admission,
        )?;
        let indexes = ensure_selected_read_indexes_admitted(
            graph,
            query,
            basis.selected_product().relational_basis(),
            admission,
        )?;
        let index_posture = WorthQueryQueryIndexPosture::SelectedInstalled(indexes);
        validate_admission_request(controls.request_scope(), query.name())?;
        let graph_work = start_prepared_query_session(
            self,
            session,
            prepared.plan,
            &prepared.obligation_identity,
            query,
            &access,
            &basis,
            &security_product,
            graph,
            admission,
        )?;
        if !authorization.belongs_to_session(graph_work.identity()) {
            return Err(graph_work_denial(query.name()));
        }
        let mut graph_work = graph_work;
        graph_work.set_retained_decision_facts(authorization.exact_fact_count());
        Ok(WorthQueryAdmittedApplicationQueryPlan {
            runtime_authority: self.runtime.authority_identity(),
            graph_authority_identity: self
                .primary_graph_authority
                .authority_identity()
                .to_string(),
            provider_identity: self.primary_graph_authority.provider_identity().to_string(),
            query,
            principal: access.principal(),
            scope: access.scope(),
            parameters,
            controls,
            canonical_work: prepared.canonical_work,
            continuation_index_id: prepared.continuation_index_id,
            continuation_state: None,
            basis,
            index_posture,
            security_product,
            graph_work,
            authorization,
            authorization_work,
            selected_access: Some(selected_access),
            governance: WorthQueryApplicationQueryGovernance::Public,
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn start_prepared_query_session<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    reserved: WorthQueryPreparedQuerySessionIdentity,
    plan: WorthQueryAdmittedGraphWorkPlan,
    obligation: &WorthQueryInstalledGraphObligationSetIdentity,
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    access: &WorthQueryApplicationQueryAccessContext<
        '_,
        Schema,
        Principal,
        PrincipalIdentity,
        Scope,
    >,
    basis: &WorthQueryApplicationQueryBasisCustody,
    security_product: &crate::basis::WorthQueryProductObservationLease,
    graph: &WorthQueryPrimaryGraph,
    admission: &mut InvalidationEditAdmission,
) -> Result<WorthQueryManagedGraphWorkSession, WorthQueryApplicationQueryAdmissionDenial>
where
    Schema: ApplicationSchema,
{
    // The returned read plan owns these two identity Strings. A failed
    // session may instead own one formatted denial; fund that maximum before
    // session construction, including the physical denial backing.
    const MAX_START_DENIAL_LABEL: usize = 27;
    admission
        .charge_external_work(3)
        .map_err(prepared_session_resource_denial)?;
    let authority_len = application
        .primary_graph_authority
        .authority_identity()
        .len();
    let provider_len = application
        .primary_graph_authority
        .provider_identity()
        .len();
    let denial_len = query
        .name()
        .len()
        .checked_add(2 + MAX_START_DENIAL_LABEL)
        .ok_or_else(prepared_session_bytes_overflow)?;
    let copy_bytes = authority_len
        .checked_add(provider_len)
        .and_then(|value| value.checked_add(denial_len))
        .ok_or_else(prepared_session_bytes_overflow)?;
    let copy_work = copy_bytes
        .checked_add(3)
        .ok_or_else(prepared_session_work_overflow)?;
    admission
        .charge_external_work(
            u64::try_from(copy_work).map_err(|_| prepared_session_work_overflow())?,
        )
        .map_err(prepared_session_resource_denial)?;
    admission
        .admit_read_scratch(
            u64::try_from(copy_bytes).map_err(|_| prepared_session_bytes_overflow())?,
        )
        .map_err(prepared_session_resource_denial)?;

    WorthQueryManagedGraphWorkSession::start_query_with_reserved_identity_admitted(
        reserved,
        plan,
        application.runtime.authority_identity(),
        query.binding_identity(),
        obligation,
        query.authority_identity(),
        access.principal().principal_entity_id(),
        WorthQueryGraphWorkAccessContextAffinity::entity(access.scope().entity_id()),
        basis.identity(),
        basis.selected_product(),
        security_product,
        application.graph_work_provider_identity(),
        graph,
        |work, bytes| {
            admission.charge_external_work(work)?;
            admission.admit_read_scratch(
                u64::try_from(bytes)
                    .map_err(|_| CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
            )?;
            Ok(())
        },
    )
    .map_err(|stop| match stop {
        WorthQueryAdmittedQuerySessionStartStop::Admission(cause) => {
            prepared_session_resource_denial(cause)
        }
        WorthQueryAdmittedQuerySessionStartStop::WorkCounterOverflow => {
            prepared_session_work_overflow()
        }
        WorthQueryAdmittedQuerySessionStartStop::PreparationBytesCounterOverflow => denial(
            WorthQueryApplicationQueryAdmissionDenialKind::ReadmissionPreparationMemoryExhausted,
            String::new(),
        ),
        WorthQueryAdmittedQuerySessionStartStop::Session(cause) => {
            let mut subject = String::with_capacity(denial_len);
            subject.push_str(query.name());
            subject.push_str(": ");
            // The four fixed StartDenial Debug labels fit the preclaimed
            // maximum; this write cannot grow the owned String.
            let _ = write!(&mut subject, "{cause:?}");
            graph_work_denial(subject)
        }
    })
}

fn prepared_session_work_overflow() -> WorthQueryApplicationQueryAdmissionDenial {
    denial(
        WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded,
        String::new(),
    )
}

fn prepared_session_bytes_overflow() -> WorthQueryApplicationQueryAdmissionDenial {
    denial(
        WorthQueryApplicationQueryAdmissionDenialKind::ReadmissionPreparationMemoryExhausted,
        String::new(),
    )
}

fn prepared_session_resource_denial(
    stop: CompanionPreflightStop,
) -> WorthQueryApplicationQueryAdmissionDenial {
    let kind = match stop {
        CompanionPreflightStop::PreparationMemoryExhausted { .. }
        | CompanionPreflightStop::PreparationMemoryCounterOverflow => {
            WorthQueryApplicationQueryAdmissionDenialKind::ReadmissionPreparationMemoryExhausted
        }
        _ => WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded,
    };
    denial(kind, String::new())
}
