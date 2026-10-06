//! Graph preparation consuming the exact permission's early shape proof.

use worth_query_admission::facade::{
    graph_obligation::WorthQueryGraphWorkIntent,
    graph_read_access::WorthQueryGraphIndexInventoryAdmissionStop,
};
use worth_query_admission::integration::{
    admit_application_query_graph_work_admitted,
    derive_graph_read_access_requirements_for_contract_admitted,
    review_application_query_graph_work_admitted,
    select_shared_application_query_graph_obligations_admitted, WorthQueryCanonicalIdentityStop,
    WorthQueryGraphObligationSelectionAdmissionStop, WorthQueryGraphWorkCapacityAdmissionStop,
    WorthQueryGraphWorkReviewAdmissionStop,
};
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::*;
use crate::domain_computation::primary_graph::application_query::{
    execution_shape::PreparedOneShotShape,
    runtime_support::primary_graph_support_inventory_admitted,
    WorthQueryAdmittedApplicationQueryControls,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare_application_query_graph_work_admitted<
        Query,
        Parameters,
        QueryResult,
        Scope,
    >(
        &self,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        parameters: &WorthQueryAdmittedApplicationQueryParameters,
        controls: &WorthQueryAdmittedApplicationQueryControls<'_>,
        shape: PreparedOneShotShape<'_>,
        graph: &WorthQueryPrimaryGraph,
        disclosure: WorthQueryAdmittedApplicationDisclosureContract,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PreparedApplicationQueryGraphWork, WorthQueryApplicationQueryAdmissionDenial> {
        // One possible terminal owns the query name. Resource refusal uses an
        // empty subject and requires no unfunded diagnostic allocation.
        let name_bytes = u64::try_from(query.name().len()).map_err(|_| work_overflow())?;
        claim(admission, name_bytes, name_bytes).map_err(resource_denial)?;
        shape.consume(query, controls, admission)?;
        validate_admission_request(controls.request_scope(), query.name())?;
        let requirements = derive_graph_read_access_requirements_for_contract_admitted(
            query.read_family_binding().planning_contract(),
            controls.lane(),
            controls.maximum_result_count().get(),
            parameters.identity(),
            query.canonical_work_policy().admission_planning(),
            &mut |work, bytes| claim(admission, work, bytes),
        )
        .map_err(canonical_stop)?;
        let inventory = primary_graph_support_inventory_admitted(
            &graph.layout,
            query.continuation(),
            query.live(),
            &requirements,
            &mut |work, bytes| claim(admission, work, bytes),
        )
        .map_err(inventory_stop)?;
        let obligations = query
            .retain_graph_obligations_for_admission_admitted(|work, bytes| {
                claim(admission, work, bytes)
            })
            .map_err(resource_denial)?;
        // This identity has exactly one inline canonical digest, no text or
        // backing to retain. The installed obligation rows remain shared.
        claim(admission, 32, 0).map_err(resource_denial)?;
        let obligation_identity = obligations.installed_set().identity().clone();
        let selected = select_shared_application_query_graph_obligations_admitted(
            obligations,
            WorthQueryGraphWorkIntent::application_query_read(),
            |work, bytes| claim(admission, work, bytes),
        )
        .map_err(|stop| match stop {
            WorthQueryGraphObligationSelectionAdmissionStop::Admission(stop) => {
                resource_denial(stop)
            }
            WorthQueryGraphObligationSelectionAdmissionStop::AccountingOverflow => work_overflow(),
            WorthQueryGraphObligationSelectionAdmissionStop::Selection(_) => {
                graph_work_denial(query.name())
            }
        })?;
        let budget = self
            .runtime
            .application_query_resource_profile()
            .admission_budget_admitted(
                controls.maximum_result_count(),
                controls.maximum_work(),
                &mut |work, bytes| claim(admission, work, bytes),
            )
            .map_err(canonical_stop)?;
        let reviewed = review_application_query_graph_work_admitted(
            selected,
            requirements,
            inventory,
            budget,
            |work, bytes| claim(admission, work, bytes),
        )
        .map_err(|stop| match stop {
            WorthQueryGraphWorkReviewAdmissionStop::Admission(stop) => resource_denial(stop),
            WorthQueryGraphWorkReviewAdmissionStop::AccountingOverflow => work_overflow(),
            WorthQueryGraphWorkReviewAdmissionStop::Denial(_) => graph_work_denial(query.name()),
        })?;
        claim(admission, 10, 0).map_err(resource_denial)?;
        super::super::work_limit::validate_resolved_work_limit(
            reviewed.review(),
            controls.maximum_work(),
            query.name(),
        )?;
        if let Some(rejected) = reviewed.review().denial() {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::GraphReadPlan(rejected.kind()),
                query.name(),
            ));
        }
        let canonical_work = WorthQueryCanonicalWorkPhases::new(
            query.installation_canonical_work(),
            parameters
                .canonical_basis()
                .work()
                .combine(reviewed.review().requirements().canonical_work()),
            worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
            worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
            worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
        );
        let plan = admit_application_query_graph_work_admitted(
            reviewed,
            self.graph_work_resource_support_ref(),
            &mut |work, bytes| claim(admission, work, bytes),
        )
        .map_err(|stop| match stop {
            WorthQueryGraphWorkCapacityAdmissionStop::Admission(stop) => resource_denial(stop),
            WorthQueryGraphWorkCapacityAdmissionStop::AccountingOverflow => work_overflow(),
            WorthQueryGraphWorkCapacityAdmissionStop::Denial(_) => graph_work_denial(query.name()),
        })?;
        Ok(PreparedApplicationQueryGraphWork {
            plan,
            obligation_identity,
            disclosure,
            canonical_work,
            // The consumed predecessor proves this is the OneShot lane.
            continuation_index_id: None,
        })
    }
}

fn claim(
    admission: &mut InvalidationEditAdmission,
    work: u64,
    bytes: u64,
) -> Result<(), CompanionPreflightStop> {
    admission.charge_external_work(work)?;
    admission.admit_read_scratch(bytes)
}

fn canonical_stop(
    stop: WorthQueryCanonicalIdentityStop<CompanionPreflightStop>,
) -> WorthQueryApplicationQueryAdmissionDenial {
    use WorthQueryCanonicalIdentityStop::*;
    match stop {
        Admission(stop) => resource_denial(stop),
        AccountingOverflow => work_overflow(),
        AllocationUnavailable => memory_unavailable(),
        Derivation(_) | Construction(_) | UnsupportedSortImplementation => denial(
            WorthQueryApplicationQueryAdmissionDenialKind::CanonicalWorkDenied,
            String::new(),
        ),
    }
}

fn inventory_stop(
    stop: WorthQueryGraphIndexInventoryAdmissionStop<CompanionPreflightStop>,
) -> WorthQueryApplicationQueryAdmissionDenial {
    use WorthQueryGraphIndexInventoryAdmissionStop::*;
    match stop {
        Admission(stop) => resource_denial(stop),
        AccountingOverflow => work_overflow(),
        AllocationUnavailable => memory_unavailable(),
        UnsupportedPreparation => denial(
            WorthQueryApplicationQueryAdmissionDenialKind::RuntimeSupportUnavailable,
            String::new(),
        ),
    }
}

fn resource_denial(stop: CompanionPreflightStop) -> WorthQueryApplicationQueryAdmissionDenial {
    match stop {
        CompanionPreflightStop::PreparationMemoryExhausted { .. }
        | CompanionPreflightStop::PreparationMemoryCounterOverflow => memory_unavailable(),
        _ => work_overflow(),
    }
}

fn work_overflow() -> WorthQueryApplicationQueryAdmissionDenial {
    denial(
        WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded,
        String::new(),
    )
}

fn memory_unavailable() -> WorthQueryApplicationQueryAdmissionDenial {
    denial(
        WorthQueryApplicationQueryAdmissionDenialKind::ReadmissionPreparationMemoryExhausted,
        String::new(),
    )
}
