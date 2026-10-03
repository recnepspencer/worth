use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationQuery, WorthQueryInstalledApplicationQueryIdentity,
};

use super::super::{
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
    WorthQueryApplicationQueryControls, WorthQueryObservedSource,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

/// The original selector and parameters have passed fresh installed-query,
/// principal, scope, request and parent-meter admission. The selected basis
/// remains owned here until Query read permission and graph work are admitted.
pub(in crate::domain_computation::primary_graph) struct PreparedApplicationQuerySourceReadmission<
    'a,
    Schema,
> {
    query_identity: WorthQueryInstalledApplicationQueryIdentity,
    parameters: WorthQueryAdmittedApplicationQueryParameters,
    controls: WorthQueryApplicationQueryControls<'a, Schema>,
}

impl<'a, Schema> PreparedApplicationQuerySourceReadmission<'a, Schema> {
    pub(in crate::domain_computation::primary_graph) fn query_identity(
        &self,
    ) -> &WorthQueryInstalledApplicationQueryIdentity {
        &self.query_identity
    }

    pub(in crate::domain_computation::primary_graph) fn parameter_identity(
        &self,
    ) -> &worth_foundational::facade::CanonicalDigestId {
        self.parameters.identity()
    }

    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (
        WorthQueryAdmittedApplicationQueryParameters,
        WorthQueryApplicationQueryControls<'a, Schema>,
    ) {
        (self.parameters, self.controls)
    }

    pub(in crate::domain_computation::primary_graph) fn selected_basis(
        &self,
    ) -> Option<(
        &crate::basis::WorthQueryProductObservationLease,
        &worth_relational::facade::snapshots::SnapshotHandle,
    )> {
        self.controls.selected_basis()
    }

    pub(super) fn selected_permission_basis(
        &self,
    ) -> Option<(
        &crate::basis::WorthQueryProductObservationLease,
        &super::super::resource_lifecycle::WorthQueryApplicationBasisLease,
    )> {
        self.controls.selected_permission_basis()
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Reissue the retained source's parameters only after a fresh principal,
    /// scope, installed query, request, and basis have been selected.
    pub(in crate::domain_computation::primary_graph) fn readmit_application_query_from_observed<
        'a,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &'a self,
        query: &'a WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: &WorthQueryApplicationQueryAccessContext<
            'a,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        observed: &WorthQueryObservedSource<Query>,
        controls: WorthQueryApplicationQueryControls<'a, Schema>,
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
        let prepared = self.prepare_application_query_from_observed(
            query, access, observed, controls, admission,
        )?;
        let (parameters, controls) = prepared.into_parts();
        self.finish_application_query_readmission(query, access, parameters, controls, admission)
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_application_query_from_observed<
        'a,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &'a self,
        query: &'a WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: &WorthQueryApplicationQueryAccessContext<
            'a,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        observed: &WorthQueryObservedSource<Query>,
        controls: WorthQueryApplicationQueryControls<'a, Schema>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        PreparedApplicationQuerySourceReadmission<'a, Schema>,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        let selector = observed.retained_scope_selector();
        let selector_visits = selector
            .locator()
            .field_path()
            .fields()
            .len()
            .checked_add(4)
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(source_work_denial)?;
        admission
            .charge_external_work(selector_visits)
            .map_err(|_| source_work_denial())?;
        let selector_work = selector
            .initialized_copy_work()
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(source_work_denial)?;
        admission
            .charge_external_work(selector_work)
            .map_err(|_| source_work_denial())?;
        let name_bytes = u64::try_from(query.name().len()).map_err(|_| source_work_denial())?;
        admission
            .charge_external_work(name_bytes)
            .map_err(|_| source_work_denial())?;
        admission
            .admit_read_scratch(name_bytes)
            .map_err(source_resource_denial)?;
        if observed.runtime_authority != self.runtime.authority_identity().as_u64()
            || observed.query_identity != *query.identity()
            || observed.schema_binding != *query.binding_identity()
        {
            return Err(WorthQueryApplicationQueryAdmissionDenial::new(
                WorthQueryApplicationQueryAdmissionDenialKind::InstalledQuery(
                    worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenialKind::StaleGeneration,
                ),
                query.name(),
            ));
        }
        if selector.locator() != access.scope().identity_locator()
            || selector.value() != access.scope().identity_value()
        {
            return Err(WorthQueryApplicationQueryAdmissionDenial::new(
                WorthQueryApplicationQueryAdmissionDenialKind::StaleScope,
                query.name(),
            ));
        }
        let (parameters, controls) = self.prepare_application_query_source_readmission(
            query,
            access,
            &observed.parameters,
            controls,
            admission,
        )?;
        Ok(PreparedApplicationQuerySourceReadmission {
            query_identity: query.identity().clone(),
            parameters,
            controls,
        })
    }
}

fn source_work_denial() -> WorthQueryApplicationQueryAdmissionDenial {
    WorthQueryApplicationQueryAdmissionDenial::new(
        WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded,
        String::new(),
    )
}

fn source_resource_denial(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> WorthQueryApplicationQueryAdmissionDenial {
    use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
    let kind = match stop {
        Stop::PreparationMemoryExhausted { .. } | Stop::PreparationMemoryCounterOverflow => {
            WorthQueryApplicationQueryAdmissionDenialKind::ReadmissionPreparationMemoryExhausted
        }
        _ => WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded,
    };
    WorthQueryApplicationQueryAdmissionDenial::new(kind, String::new())
}
