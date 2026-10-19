//! Context at graph-preparation callbacks and their existing refusal mappings.
use super::super::*;
use crate::domain_computation::primary_graph::application_query::denial::{
    QueryPreparationContext, QueryPreparationOperation,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use worth_query_admission::facade::graph_read_access::WorthQueryGraphIndexInventoryAdmissionStop;
use worth_query_admission::integration::{
    WorthQueryCanonicalIdentityPreparationContext, WorthQueryCanonicalIdentityStop,
};
use worth_relational::facade::mvcc::CompanionPreflightStop;

pub(super) struct PreparationChargeStop {
    cause: CompanionPreflightStop,
    context: QueryPreparationContext,
}

impl PreparationChargeStop {
    fn with_canonical(mut self, context: WorthQueryCanonicalIdentityPreparationContext) -> Self {
        self.context.canonical.get_or_insert(context);
        self
    }
}

pub(super) fn claim(
    admission: &mut InvalidationEditAdmission,
    operation: QueryPreparationOperation,
    work: u64,
    bytes: u64,
) -> Result<(), PreparationChargeStop> {
    let stopped = |cause| PreparationChargeStop {
        cause,
        context: QueryPreparationContext::new(operation, work, bytes),
    };
    admission.charge_external_work(work).map_err(stopped)?;
    admission.admit_read_scratch(bytes).map_err(stopped)
}

pub(super) fn resource_denial(
    stop: PreparationChargeStop,
) -> WorthQueryApplicationQueryAdmissionDenial {
    let denial = match stop.cause {
        CompanionPreflightStop::PreparationMemoryExhausted { .. }
        | CompanionPreflightStop::PreparationMemoryCounterOverflow => memory_unavailable(),
        _ => work_overflow(),
    };
    denial
        .with_preparation_stop(stop.cause)
        .with_preparation_context(stop.context)
}

pub(super) fn canonical_stop(
    stop: WorthQueryCanonicalIdentityStop<PreparationChargeStop>,
) -> WorthQueryApplicationQueryAdmissionDenial {
    use WorthQueryCanonicalIdentityStop::*;
    match stop {
        Admission(stop) => resource_denial(stop),
        Preparation { cause, context } => resource_denial(cause.with_canonical(context)),
        AccountingOverflow => work_overflow(),
        AllocationUnavailable => memory_unavailable(),
        Derivation(_) | Construction(_) | UnsupportedSortImplementation => denial(
            WorthQueryApplicationQueryAdmissionDenialKind::CanonicalWorkDenied,
            String::new(),
        ),
    }
}

pub(super) fn inventory_stop(
    stop: WorthQueryGraphIndexInventoryAdmissionStop<PreparationChargeStop>,
) -> WorthQueryApplicationQueryAdmissionDenial {
    use WorthQueryGraphIndexInventoryAdmissionStop::*;
    match stop {
        Admission(stop) => resource_denial(stop),
        Preparation { cause, context } => resource_denial(cause.with_canonical(context)),
        AccountingOverflow => work_overflow(),
        AllocationUnavailable => memory_unavailable(),
        UnsupportedPreparation => denial(
            WorthQueryApplicationQueryAdmissionDenialKind::RuntimeSupportUnavailable,
            String::new(),
        ),
    }
}

pub(super) fn work_overflow() -> WorthQueryApplicationQueryAdmissionDenial {
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
