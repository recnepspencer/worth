//! A scalar assignment body and a separate outbound/completion operation.
use super::*;
use std::num::NonZeroU64;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

pub(super) fn extend(members: &mut Vec<ApplicationSchemaMember>) {
    let n = NonZeroU64::new(128).unwrap();
    members.extend([
        ApplicationSchemaMember::Operation {
            operation: "BudgetBody".into(),
            input_type: portable("worth.rust.unit"),
        },
        ApplicationSchemaMember::OperationBody {
            operation: "BudgetBody".into(),
            body: ApplicationOperationBodyProgram::from_untrusted_scope_fields(vec![
                ApplicationOperationBodyScopeField::Assign {
                    field: ApplicationFieldBindingLocus::new("BudgetScalar", "Value", "value"),
                    binding: portable("u64"),
                    value: worth_foundational::facade::AspectValue::UInt64(7),
                },
            ]),
        },
        ApplicationSchemaMember::OperationDecisionRead {
            operation: "BudgetBody".into(),
            target: ApplicationOperationDecisionReadTarget::Field {
                entity: "BudgetScalar".into(),
                aspect: "Value".into(),
                field: "value".into(),
            },
        },
        ApplicationSchemaMember::OperationProgram {
            operation: "BudgetBody".into(),
            target: ApplicationOperationProgramTarget::Write {
                entity: "BudgetScalar".into(),
                aspect: "Value".into(),
                field: "value".into(),
            },
        },
        ApplicationSchemaMember::OperationProjectionWorkBudget {
            operation: "BudgetBody".into(),
            maximum_work_units: 128,
        },
        ApplicationSchemaMember::Operation {
            operation: "BudgetEmit".into(),
            input_type: portable("worth.rust.unit"),
        },
        ApplicationSchemaMember::Effect {
            effect: "BudgetNotification".into(),
            payload_type: portable("worth.rust.unit"),
        },
        ApplicationSchemaMember::OperationProgram {
            operation: "BudgetEmit".into(),
            target: ApplicationOperationProgramTarget::Emit {
                effect: "BudgetNotification".into(),
            },
        },
        ApplicationSchemaMember::OperationProjectionWorkBudget {
            operation: "BudgetEmit".into(),
            maximum_work_units: 128,
        },
        ApplicationSchemaMember::OperationExternalEffect {
            operation: "BudgetEmit".into(),
            effect: "BudgetNotification".into(),
            rust_payload_type: portable("worth.rust.unit"),
            protocol: ApplicationExternalEffectProtocol::new(
                BoundaryProtocolIdentity::new("worth.budget.notification"),
                BoundaryProtocolVersion::new(1),
            ),
            maximum_payload_bytes: 128,
            correlation_family: WorthQueryExternalEffectCorrelationFamily::new(
                "budget-notification",
            )
            .unwrap(),
        },
        ApplicationSchemaMember::OperationInboundOccurrence {
            operation: "BudgetEmit".into(),
            effect: "BudgetNotification".into(),
            protocol: ApplicationInboundOccurrenceProtocol::new(
                BoundaryProtocolIdentity::new("worth.budget.completion"),
                BoundaryProtocolVersion::new(1),
            ),
            source_identity: "budget-notification-source".into(),
            limits: ApplicationInboundOccurrenceLimits {
                maximum_envelope_bytes: n,
                maximum_verifier_work: n,
                maximum_payload_bytes: n,
                maximum_outstanding_dispatch_provenance: n,
                maximum_accepted_occurrences: n,
                maximum_accepted_bytes: n,
                maximum_concurrent_publications: n,
                maximum_discovery_work: n,
                replay_window_milliseconds: n,
                maximum_cleanup_work: n,
            },
        },
    ]);
}
