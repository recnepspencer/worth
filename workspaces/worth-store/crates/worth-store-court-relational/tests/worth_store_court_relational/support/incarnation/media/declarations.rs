//! The open-time incarnation decision under every identified media fault.
pub(crate) use std::num::NonZeroU64;
pub(crate) use worth_store::physical_runtime::certification::MediaOperationRole;
pub(crate) use {
    crate::durable_run::open, crate::incarnation::media, crate::workload, std::path::Path,
    worth_execution::ExecutionRequest,
    worth_store_relational::RelationalStoreHomeOpenRefusal as Refusal,
};
pub(crate) use {crate::media_faults, crate::schema_transition};
pub(crate) use {
    worth_store::physical_runtime::FilesystemAccessPosture,
    worth_store::physical_runtime::FilesystemMediaAdmission,
};
pub(crate) use {
    worth_store_relational::RelationalStoreHome, worth_store_relational::RelationalStoreHomeRequest,
};

pub(crate) fn faulted_open(
    location: &Path,
    faults: &[crate::media_faults::Fault],
    execution: &ExecutionRequest<'_, '_>,
) -> Option<(
    Result<RelationalStoreHome, Refusal>,
    Vec<worth_store::physical_runtime::certification::CertificationMediaFaultActivation>,
)> {
    let media = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
        worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = media.fault_schedule_authority();
    let activations: Vec<_> = faults
        .iter()
        .map(|_| authority.one_shot_activation())
        .collect();
    let rules = faults
        .iter()
        .zip(&activations)
        .map(|((role, nth, directive), activation)| {
            authority
                .rule(*role, 1, directive.clone())
                .for_nth_identified_operation_after_activation(activation.clone(), *nth)
        })
        .collect();
    let schedule = authority.schedule(rules).ok()?;
    let opened =
        worth_store_relational::RelationalStoreHome::certification_open_with_descriptor_observer(
            worth_store_relational::RelationalStoreHomeRequest::new(
                location,
                crate::workload::runtime_builder(crate::schema_transition::SOURCE),
            ),
            media.with_fault_schedule(schedule),
            |_| crate::media_faults::arm(&activations),
            execution,
        );
    Some((opened, activations))
}
