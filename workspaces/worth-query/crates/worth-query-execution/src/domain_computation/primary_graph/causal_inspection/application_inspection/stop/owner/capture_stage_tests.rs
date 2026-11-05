//! Each routed common failure retains the actual owner's independently named stage.
use super::super::super::{
    identity::InspectionIdentityStop as Identity, report::InspectionReportDenial as Report,
    target::InspectionTargetDenial as Target,
};
use super::*;
use {crate::domain_computation::vocabulary::primary_graph::retained_stage::RetainedStorageDenial as Storage, crate::domain_computation::primary_graph::application_query::ProjectionComparisonDenial as Comparison};
use worth_runtime_bridge::facade::BridgeQueryObservationReportingStop as Bridge;
use InspectionAdmissionDenial as A;
fn captured<E>(cause: E) -> CaptureStop<E> {
    CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Domain(cause)))
}
fn panicked_capture<E>() -> CaptureStop<E> {
    CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Panic))
}
fn capacity_capture<E>() -> CaptureStop<E> {
    CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::ResultCapacityExceeded))
}
macro_rules! stage_court {
    ($name:ident, $input:expr, $expected:pat) => {
        #[test]
        fn $name() {
            let denied = WorthQueryApplicationInspectionDenial::new(
                InspectionCause::Admission($input),
                None,
            );
            assert!(
                matches!(denied.owner_refusal(), Some($expected)),
                "{:?}",
                denied.owner_refusal()
            );
        }
    };
}
stage_court!(
    diagnostic_admission_panic,
    A::DiagnosticAdmission(panicked_capture()),
    Refusal::KernelPanic(Stage::DiagnosticAdmission)
);
stage_court!(
    comparison_parameters_panic,
    A::ComparisonParameters(panicked_capture()),
    Refusal::KernelPanic(Stage::ComparisonParameters)
);
stage_court!(
    connection_panic,
    A::Connection(panicked_capture()),
    Refusal::KernelPanic(Stage::Connection)
);
stage_court!(
    source_materialization_panic,
    A::RichSource(panicked_capture()),
    Refusal::KernelPanic(Stage::SourceMaterialization)
);
stage_court!(
    query_materialization_panic,
    A::Rich(panicked_capture()),
    Refusal::KernelPanic(Stage::QueryMaterialization)
);
stage_court!(
    comparison_panic,
    A::Comparison(panicked_capture()),
    Refusal::KernelPanic(Stage::Comparison)
);
stage_court!(
    diagnostic_contract_panic,
    A::DiagnosticContract(panicked_capture()),
    Refusal::KernelPanic(Stage::DiagnosticContract)
);
stage_court!(
    diagnostic_preparation_panic,
    A::DiagnosticPreparation(panicked_capture()),
    Refusal::KernelPanic(Stage::DiagnosticPreparation)
);
stage_court!(
    target_panic,
    A::Target(Target::Storage(panicked_capture())),
    Refusal::KernelPanic(Stage::Target)
);
stage_court!(
    storage_panic,
    A::Storage(panicked_capture()),
    Refusal::KernelPanic(Stage::Storage)
);
stage_court!(
    plan_panic,
    A::Plan(panicked_capture()),
    Refusal::KernelPanic(Stage::Plan)
);
stage_court!(
    report_panic,
    A::Report(panicked_capture()),
    Refusal::KernelPanic(Stage::Report)
);
stage_court!(
    diagnostic_admission_capacity,
    A::DiagnosticAdmission(capacity_capture()),
    Refusal::KernelResultCapacityExceeded(Stage::DiagnosticAdmission)
);
stage_court!(
    comparison_parameters_capacity,
    A::ComparisonParameters(capacity_capture()),
    Refusal::KernelResultCapacityExceeded(Stage::ComparisonParameters)
);
stage_court!(
    connection_capacity,
    A::Connection(capacity_capture()),
    Refusal::KernelResultCapacityExceeded(Stage::Connection)
);
stage_court!(
    source_materialization_capacity,
    A::RichSource(capacity_capture()),
    Refusal::KernelResultCapacityExceeded(Stage::SourceMaterialization)
);
stage_court!(
    query_materialization_capacity,
    A::Rich(capacity_capture()),
    Refusal::KernelResultCapacityExceeded(Stage::QueryMaterialization)
);
stage_court!(
    comparison_capacity,
    A::Comparison(capacity_capture()),
    Refusal::KernelResultCapacityExceeded(Stage::Comparison)
);
stage_court!(
    diagnostic_contract_capacity,
    A::DiagnosticContract(capacity_capture()),
    Refusal::KernelResultCapacityExceeded(Stage::DiagnosticContract)
);
stage_court!(
    diagnostic_preparation_capacity,
    A::DiagnosticPreparation(capacity_capture()),
    Refusal::KernelResultCapacityExceeded(Stage::DiagnosticPreparation)
);
stage_court!(
    target_capacity,
    A::Target(Target::Storage(capacity_capture())),
    Refusal::KernelResultCapacityExceeded(Stage::Target)
);
stage_court!(
    storage_capacity,
    A::Storage(capacity_capture()),
    Refusal::KernelResultCapacityExceeded(Stage::Storage)
);
stage_court!(
    plan_capacity,
    A::Plan(capacity_capture()),
    Refusal::KernelResultCapacityExceeded(Stage::Plan)
);
stage_court!(
    report_capacity,
    A::Report(capacity_capture()),
    Refusal::KernelResultCapacityExceeded(Stage::Report)
);
stage_court!(
    storage_overflow,
    A::Storage(captured(Storage::Overflow)),
    Refusal::StorageAccountingOverflow(Stage::Storage)
);
stage_court!(
    target_storage_overflow,
    A::Target(Target::Storage(captured(Storage::Overflow))),
    Refusal::StorageAccountingOverflow(Stage::Target)
);
stage_court!(
    contract_storage_overflow,
    A::DiagnosticContract(captured(DiagnosticContractDenial::Storage(
        Storage::Overflow
    ))),
    Refusal::StorageAccountingOverflow(Stage::DiagnosticContract)
);
stage_court!(
    plan_storage_overflow,
    A::Plan(captured(InspectionPlanDenial::Storage(Storage::Overflow))),
    Refusal::StorageAccountingOverflow(Stage::Plan)
);
stage_court!(
    comparison_storage_overflow,
    A::Comparison(captured(Comparison::Storage(Storage::Overflow))),
    Refusal::StorageAccountingOverflow(Stage::Comparison)
);
stage_court!(
    report_storage_overflow,
    A::Report(captured(Report::Storage(Storage::Overflow))),
    Refusal::StorageAccountingOverflow(Stage::Report)
);
stage_court!(
    identity_resource_storage_overflow,
    A::Report(captured(Report::Identity(Identity::Resource(
        MapKernelFailure::Domain(Storage::Overflow)
    )))),
    Refusal::StorageAccountingOverflow(Stage::ReportIdentity)
);
stage_court!(
    bridge_resource_storage_overflow,
    A::Report(captured(Report::Bridge(Bridge::Resource(
        MapKernelFailure::Domain(Storage::Overflow)
    )))),
    Refusal::StorageAccountingOverflow(Stage::ReportBridge)
);
macro_rules! binding_court {
    ($name:ident, $cause:expr, $expected:pat) => {
        #[test]
        fn $name() {
            let denied =
                WorthQueryApplicationInspectionDenial::new(InspectionCause::Binding($cause), None);
            assert!(
                matches!(denied.owner_refusal(), Some($expected)),
                "{:?}",
                denied.owner_refusal()
            );
        }
    };
}
binding_court!(
    binding_panic,
    panicked_capture(),
    Refusal::KernelPanic(Stage::Binding)
);
binding_court!(
    binding_capacity,
    capacity_capture(),
    Refusal::KernelResultCapacityExceeded(Stage::Binding)
);
binding_court!(
    binding_storage_overflow,
    captured(InspectionBindingDenial::Storage(Storage::Overflow)),
    Refusal::StorageAccountingOverflow(Stage::Binding)
);
fn basis_failure(
) -> worth_foundational::facade::CanonicalBasisPreparationStop<MapKernelFailure<Storage>> {
    use worth_foundational::facade::{
        CanonicalBasisPreparationPhase as Phase, CanonicalBasisPreparationQuote as Quote,
        CanonicalBasisPreparationStop as Basis,
    };
    // Descriptive refusal input, not a canonical-ready proof or admitted execution.
    Basis::Resource {
        cause: MapKernelFailure::Domain(Storage::Overflow),
        quote: Quote {
            phase: Phase::SequenceVisit,
            work: 1,
            bytes: 0,
            entry_count: 0,
            comparison_width: None,
        },
    }
}
use worth_foundational::facade::CanonicalDigestAdmissionStop as Digest;
stage_court!(
    identity_basis_storage_overflow,
    A::Report(captured(Report::Identity(Identity::Basis(basis_failure())))),
    Refusal::StorageAccountingOverflow(Stage::ReportIdentity)
);
stage_court!(
    bridge_basis_storage_overflow,
    A::Report(captured(Report::Bridge(Bridge::Basis(basis_failure())))),
    Refusal::StorageAccountingOverflow(Stage::ReportBridge)
);
stage_court!(
    identity_digest_storage_overflow,
    A::Report(captured(Report::Identity(Identity::Digest(
        Digest::Resource(MapKernelFailure::Domain(Storage::Overflow))
    )))),
    Refusal::StorageAccountingOverflow(Stage::ReportIdentity)
);
stage_court!(
    bridge_digest_storage_overflow,
    A::Report(captured(Report::Bridge(Bridge::Digest(Digest::Resource(
        MapKernelFailure::Domain(Storage::Overflow)
    ))))),
    Refusal::StorageAccountingOverflow(Stage::ReportBridge)
);
