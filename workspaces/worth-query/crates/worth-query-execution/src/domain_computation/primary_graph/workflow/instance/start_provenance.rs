//! Immutable original revision: native funded storage, never a permission or hash issuer.
use crate::domain_computation::primary_graph::{
    application_attempt::{
        observe_field_value_borrowed, WorthQueryApplicationAttemptDenial as Attempt,
        WorthQueryApplicationAttemptDenialKind as Kind,
    },
    program_occurrence::WorthQueryInstalledProgramSupport,
    workflow::schema::{
        version::WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION, WorthQueryWorkflowLayout,
    },
};
use worth_execution::{ExecutionByteBuffer, ExecutionRequest};
use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_relational::facade::{
    identity::EntityId,
    runtime::{RelationalPinnedReadAdmissionDenial, RelationalRuntime},
    snapshots::SnapshotHandle,
    symbols::{RelationalFieldTextAdmissionDenial, Symbol},
    transactions::RelationalExecutionDenialCause,
};

pub(in crate::domain_computation::primary_graph) fn admit(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    revision: &ApplicationProgramRevision,
) -> Result<AspectValue, Attempt> {
    use std::io::Write;
    execution.check_live().map_err(|cause| {
        Attempt::relational_read_stopped(cause.into(), "workflow start provenance")
    })?;
    let mut text = std::io::Cursor::new([0u8; 64]);
    write!(&mut text, "{revision}").expect("canonical revision is exactly 64 ASCII bytes");
    assert_eq!(text.position(), 64);
    let mut bytes = ExecutionByteBuffer::allocate(64, execution.allocation_policy())
        .map_err(|cause| super::super::actor::storage_denial(cause, "workflow start provenance"))?;
    bytes
        .extend_from_slice(text.get_ref())
        .map_err(|cause| super::super::actor::storage_denial(cause, "workflow start provenance"))?;
    let bytes = bytes
        .seal()
        .map_err(|cause| super::super::actor::storage_denial(cause, "workflow start provenance"))?;
    runtime
        .admit_field_text(execution, basis, &bytes)
        .and_then(|text| text.field_value_for(basis))
        .map_err(|cause| match cause {
            RelationalFieldTextAdmissionDenial::Allocation(cause) => {
                super::super::actor::storage_denial(cause, "workflow start provenance")
            }
            RelationalFieldTextAdmissionDenial::Stopped(cause) => {
                Attempt::relational_read_stopped(cause.into(), "workflow start provenance")
            }
            _ => Attempt::new(Kind::InvalidAuthoritativeValue, "workflow start provenance"),
        })
}

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum StartProvenanceReadDenial {
    Corrupt,
    UnsupportedProtocol(u64),
    Execution(RelationalExecutionDenialCause),
}
/// Caller first admits scope/subject/Spec. Reads ONLY the supplied retained root.
/// The result is copied from installed meaning, never constructed from recorded bytes.
pub(in crate::domain_computation::primary_graph) fn read<Schema>(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    instance: EntityId,
    support: &WorthQueryInstalledProgramSupport<Schema>,
) -> Result<ApplicationProgramRevision, StartProvenanceReadDenial> {
    use StartProvenanceReadDenial as Denial;
    let version = observe_field_value_borrowed(
        runtime,
        execution,
        snapshot,
        instance,
        layout.instance.entity_kind,
        &layout.instance.protocol_version,
        |value| match value {
            Some(AspectValue::UInt64(version)) => Some(*version),
            _ => None,
        },
    )
    .map_err(Denial::Execution)?
    .flatten()
    .ok_or(Denial::Corrupt)?;
    if version != WORKFLOW_INSTANCE_FACT_PROTOCOL_VERSION {
        return Err(Denial::UnsupportedProtocol(version));
    }
    let symbol = observe_field_value_borrowed(
        runtime,
        execution,
        snapshot,
        instance,
        layout.instance.entity_kind,
        &layout.instance.started_program_revision,
        |value| match value {
            Some(AspectValue::String(InternedString::Symbol(symbol))) => Some(Symbol(symbol.0)),
            _ => None,
        },
    )
    .map_err(Denial::Execution)?
    .flatten()
    .ok_or(Denial::Corrupt)?;
    let pin = runtime
        .read_truth()
        .pin_read_capability(snapshot, &[], execution)
        .map_err(|cause| match cause {
            RelationalPinnedReadAdmissionDenial::Request(stop) => Denial::Execution(stop.into()),
            _ => Denial::Corrupt,
        })?;
    let text = pin
        .symbols()
        .retained_text(symbol)
        .filter(|text| text.is_managed())
        .ok_or(Denial::Corrupt)?;
    support
        .rostered_for_revision_text(text.as_str())
        .map(|entry| *entry.revision())
        .ok_or(Denial::Corrupt)
}
