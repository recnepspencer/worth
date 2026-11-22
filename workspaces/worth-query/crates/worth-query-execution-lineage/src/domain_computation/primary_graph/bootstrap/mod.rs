pub mod publication;
pub use publication::WorthQueryPrimaryGraphPublication;

pub fn map_checkpoint_recovery_stop(
    stop: worth_relational::facade::durability::RelationalRecoveryStop,
) -> super::WorthQueryPrimaryGraphInstallationDenial {
    use super::{
        WorthQueryPrimaryGraphInstallationDenial as Denial,
        WorthQueryPrimaryGraphInstallationDenialKind as Kind,
    };
    use worth_relational::facade::durability::RelationalRecoveryStop;
    let denial = match stop.clone() {
        RelationalRecoveryStop::AdmissionImage(cause) => Denial::new(
            Kind::CheckpointRecoveryRejected,
            format!("native checkpoint admission image refused: {cause:?}"),
        ),
        RelationalRecoveryStop::Content(_) => Denial::new(
            Kind::ContentAdmissionRejected,
            "native checkpoint content refused",
        ),
        RelationalRecoveryStop::Durability(error) => Denial::new(
            Kind::CheckpointRecoveryRejected,
            format!("native checkpoint recovery denied: {error:?}"),
        ),
        RelationalRecoveryStop::Allocation(cause) => Denial::new(
            Kind::NativeAllocation {
                kind: cause.classify(),
                requested_payload_bytes: cause.requested_payload_bytes(),
            },
            "native checkpoint recovery allocation refused",
        ),
        RelationalRecoveryStop::Request(cause) => {
            let translated =
                crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_kind(
                    cause, None,
                );
            let kind = match translated {
                Ok(kind) => Kind::ExecutionDenied { kind },
                Err(kind) => Kind::ExecutionControlStopped { kind },
            };
            Denial::new(
                kind,
                format!("native checkpoint recovery request stopped: {cause:?}"),
            )
        }
    };
    denial.with_native_recovery_cause(stop)
}
