//! Source DATA records the exact old basis; fresh query admission recreates proof.
use super::*;
use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::primary_graph::ReadObservationRuntimeExt;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryApplicationReadObservation,
    WorthQueryOriginalOperationDenial as D, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode, WorthQueryProductQueryControls,
};
use worth_query_declaration::facade::{
    application_query::ApplicationQueryBinding,
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};
use worth_relational::facade::identity::EntityId;
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WorthQueryOriginalSourceData {
    owner: worth_foundational::facade::DurableWorldOwnerIdentity,
    commit: u64,
    branch: String,
    query: [u8; 32],
    root: EntityId,
    parameters: Vec<u8>,
    scope: Vec<u8>,
    commitment: [u8; 32],
    result_set: bool,
}
pub enum WorthQueryReadmittedOriginalSource<Q> {
    Row(WorthQueryObservedSource<Q>),
    ResultSet(WorthQueryObservedResultSet<Q>),
}
pub fn capture_original_operation_source<Q>(
    source: &WorthQueryObservedSource<Q>,
    result_set: bool,
    _: &worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccess,
) -> Result<WorthQueryOriginalSourceData, D> {
    let commit = source
        .selected_product_commit()
        .ok_or(D::OwnershipMaterialUnavailable)?;
    let locator = source.scope_selector.locator();
    let fields = locator
        .field_path()
        .fields()
        .iter()
        .map(|field| field.as_str())
        .collect::<Vec<_>>();
    Ok(WorthQueryOriginalSourceData {
        owner: commit
            .owner_identity()
            .durable()
            .ok_or(D::OwnershipMaterialUnavailable)?,
        commit: commit.ordinal(),
        branch: source.branch.0.as_str().to_owned(),
        query: *source.query_identity.as_bytes(),
        root: source.model_root,
        parameters: rmp_serde::to_vec(source.parameters.bindings())
            .map_err(|_| D::OwnershipMaterialUnavailable)?,
        scope: rmp_serde::to_vec(&(
            locator.aspect().aspect_key().as_str(),
            fields,
            source.scope_selector.value(),
            source.scope_selector.subject(),
        ))
        .map_err(|_| D::OwnershipMaterialUnavailable)?,
        commitment: source.idempotency_identity().bytes(),
        result_set,
    })
}
impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn readmit_original_operation_source<B>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        _: &worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccess,
        external: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        data: &WorthQueryOriginalSourceData,
    ) -> Result<WorthQueryReadmittedOriginalSource<B::Query>, D>
    where
        B: ApplicationQueryBinding<S>,
        <B::ResultBinding as ApplicationStructuredValueBinding>::Value:
            crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                    S,
                    B::Query,
                > + 'static,
    {
        let execution = phase
            .checked_execution_request_for(self.product_runtime())
            .map_err(D::Execution)?;
        let binding = self
            .installed_schema()
            .installed_query_binding::<B>()
            .map_err(unavailable)?;
        let query = binding.query();
        if query.identity().as_bytes() != &data.query {
            return Err(D::OwnershipMaterialUnavailable);
        }
        let commit = worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
            data.owner,
            data.commit,
        );
        let basis = self
            .readmit_output_observation(&execution, commit, &data.branch)
            .map_err(unavailable)?;
        let branch = WorthQueryProductBranch::from_occurrence(basis.lifecycle_incarnation());
        let observation = WorthQueryApplicationReadObservation::from_product(
            self,
            crate::basis::WorthQueryProductObservationLease::new(basis),
        );
        let retained = self
            .select_application_read_observation_with_request(&execution, &observation)
            .map_err(unavailable)?;
        let current = self
            .on_branch(branch)
            .select_with_request(&execution)
            .map_err(unavailable)?;
        let principal = current
            .resolve_authenticated_principal_admitted(
                &execution,
                binding.principal_binding(),
                external,
                request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(D::Principal)?;
        let (aspect, fields, value, subject): (
            String,
            Vec<String>,
            worth_foundational::facade::AspectValue,
            EntityId,
        ) = rmp_serde::from_slice(&data.scope).map_err(unavailable)?;
        let selector = WorthQueryObservedScopeSelector::readmit_recorded(
            &aspect, &fields, value, subject, &execution,
        )
        .map_err(unavailable)?;
        let scope = current
            .resolve_retained_query_scope(&execution, B::scope_field(), &selector, request)
            .map_err(D::Scope)?;
        let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
        let parameters = rmp_serde::from_slice(&data.parameters).map_err(unavailable)?;
        let parameters = worth_query_admission::facade::application_query::readmit_recorded_application_query_parameters(query, parameters).map_err(unavailable)?;
        let maximum = self
            .resolve_application_query_limits(binding.limits())
            .maximum_results();
        let (_, controls) = current
            .retained_query_controls(
                &execution,
                retained,
                query.name(),
                WorthQueryProductQueryControls::new(maximum, request),
            )
            .map_err(unavailable)?;
        let plan = self
            .finish_application_query_readmission(
                &execution, query, &access, parameters, controls, None,
            )
            .map_err(D::QueryAdmission)?;
        let source = self
            .execute_application_query_one_shot_with_request(&execution, plan)
            .map_err(D::QueryExecution)?
            .into_admitted_disclosed()
            .into_output_demand_source();
        let recovered = if data.result_set {
            let set = source
                .result_set_observation()
                .ok_or(D::OwnershipMaterialUnavailable)?
                .clone();
            if set.source().idempotency_identity().bytes() != data.commitment {
                return Err(D::OwnershipMaterialUnavailable);
            }
            WorthQueryReadmittedOriginalSource::ResultSet(set)
        } else {
            let row = source
                .observed_sources()
                .iter()
                .find(|row| row.model_root == data.root)
                .ok_or(D::OwnershipMaterialUnavailable)?
                .clone();
            if row.idempotency_identity().bytes() != data.commitment {
                return Err(D::OwnershipMaterialUnavailable);
            }
            WorthQueryReadmittedOriginalSource::Row(row)
        };
        Ok(recovered)
    }
}

fn unavailable<T>(_: T) -> D {
    D::OwnershipMaterialUnavailable
}
