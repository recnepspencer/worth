use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;
use crate::application_schema::canonical_decision_read_identity::append_decision_read_target;
use crate::application_schema::canonical_operation_identity::append_operation_target;
use crate::application_schema::ApplicationSchemaMember;
mod creation;

pub(super) fn append_body(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    operation: &str,
    body: &crate::application_schema::ApplicationOperationBodyProgram,
) {
    use crate::application_schema::ApplicationOperationBodyScopeField;
    append_operation_header(
        basis,
        prefix,
        if body.succession().is_some() {
            "operation-body-v13"
        } else if body.creation().is_some_and(|creation| creation.fields().iter().any(|step| step.related_source().is_some())) {
            "operation-body-v12"
        } else if body.creation().is_some_and(|creation| creation.links().iter().any(|link|
            link.posture == crate::application_schema::ApplicationOperationBodyCreationLinkPosture::ReplaceSingle)) {
            "operation-body-v11"
        } else if body
            .creation()
            .is_some_and(|creation| creation.enrollment().is_some())
        {
            "operation-body-v10"
        } else if body
            .creation()
            .is_some_and(|creation| !creation.links().is_empty())
        {
            "operation-body-v9"
        } else if !body.outputs().is_empty() {
            "operation-body-v8"
        } else if body.has_optional_inputs() {
            "operation-body-v7"
        } else if body.explicit_scope_entity().is_some() || body.is_creation_only() {
            "operation-body-v6"
        } else if !body.requirements().is_empty() {
            "operation-body-v5"
        } else if body.scope_fields().iter().any(|step| {
            matches!(
                step,
                ApplicationOperationBodyScopeField::RequirePresent { .. }
            )
        }) {
            "operation-body-v4"
        } else if body.scope_fields().iter().any(|step| {
            matches!(
                step,
                ApplicationOperationBodyScopeField::SetOptionalInput { .. }
                    | ApplicationOperationBodyScopeField::ClearOptional { .. }
            )
        }) {
            "operation-body-v3"
        } else if body.creation().is_some() {
            "operation-body-v2"
        } else {
            "operation-body-v1"
        },
        operation,
    );
    if !body.outputs().is_empty() {
        basis.usize(format!("{prefix}.outputs"), body.outputs().len());
        for (index, output) in body.outputs().iter().enumerate() {
            basis.text(
                format!("{prefix}.output[{index}]"),
                &output.canonical_record(),
            );
        }
    }
    if body.explicit_scope_entity().is_some() || body.is_creation_only() {
        basis.text(
            format!("{prefix}.scope-entity"),
            body.scope_entity().unwrap_or_default(),
        );
    }
    if !body.requirements().is_empty() {
        basis.usize(format!("{prefix}.requirements"), body.requirements().len());
        for (index, requirement) in body.requirements().iter().enumerate() {
            basis.text(
                format!("{prefix}.requirement[{index}]"),
                &requirement.canonical_record(),
            );
        }
    }
    basis.usize(format!("{prefix}.steps"), body.scope_fields().len());
    for (index, step) in body.scope_fields().iter().enumerate() {
        let prefix = format!("{prefix}.step[{index}]");
        basis.text(
            format!("{prefix}.kind"),
            match step {
                ApplicationOperationBodyScopeField::Expect { .. } => "expect",
                ApplicationOperationBodyScopeField::Assign { .. } => "assign",
                ApplicationOperationBodyScopeField::AssignInput { .. } => "assign-input",
                ApplicationOperationBodyScopeField::SetOptionalInput { .. } => "set-optional-input",
                ApplicationOperationBodyScopeField::AssignOptionalInput { .. } => {
                    "assign-optional-input"
                }
                ApplicationOperationBodyScopeField::ClearOptional { .. } => "clear-optional",
                ApplicationOperationBodyScopeField::RequirePresent { .. } => "require-present",
            },
        );
        basis.text(format!("{prefix}.entity"), step.field().entity());
        basis.text(format!("{prefix}.aspect"), step.field().aspect());
        basis.text(format!("{prefix}.field"), step.field().field());
        basis.text(format!("{prefix}.binding"), step.binding().as_str());
        if let Some(value) = step.literal_value() {
            basis.value(
                format!("{prefix}.value"),
                worth_foundational::facade::canonical_basis_value_for_aspect_value(value),
            );
        } else if let Some(member) = step.input_member() {
            basis.text(format!("{prefix}.input-member"), member);
            basis.text(
                format!("{prefix}.input-family"),
                step.scalar_family().canonical_name(),
            );
        } else if matches!(
            step,
            ApplicationOperationBodyScopeField::RequirePresent { .. }
        ) {
            basis.text(
                format!("{prefix}.require-family"),
                step.scalar_family().canonical_name(),
            );
        } else {
            basis.text(
                format!("{prefix}.clear-family"),
                step.scalar_family().canonical_name(),
            );
        }
    }
    if let Some(creation) = body.creation() {
        creation::append(basis, prefix, creation);
    }
    if let Some(succession) = body.succession() {
        basis.text(
            format!("{prefix}.succession"),
            &succession.canonical_record(),
        );
    }
}

use super::aftermath::append_declared_aftermath;

pub(super) fn append_operation_member(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    member: &ApplicationSchemaMember,
) {
    match member {
        ApplicationSchemaMember::Operation {
            operation,
            input_type,
        } => {
            append_operation_header(basis, prefix, "operation", operation);
            basis.text(format!("{prefix}.input-type"), input_type.as_str());
        }
        ApplicationSchemaMember::OperationProgram { operation, target } => {
            append_operation_header(basis, prefix, "operation-program", operation);
            append_operation_target(basis, &format!("{prefix}.target"), target);
        }
        ApplicationSchemaMember::OperationDecisionRead { operation, target } => {
            append_operation_header(basis, prefix, "operation-decision-read", operation);
            append_decision_read_target(basis, &format!("{prefix}.target"), target);
        }
        ApplicationSchemaMember::OperationMutationPrecondition { operation, target } => {
            append_mutation_precondition(basis, prefix, operation, target)
        }
        ApplicationSchemaMember::OperationProjectionWorkBudget {
            operation,
            maximum_work_units,
        } => append_projection_work_budget(basis, prefix, operation, *maximum_work_units),
        ApplicationSchemaMember::OperationExternalEffect {
            operation,
            effect,
            rust_payload_type,
            protocol,
            maximum_payload_bytes,
            correlation_family,
        } => append_external_effect(
            basis,
            prefix,
            ExternalEffectEncoding {
                operation,
                effect,
                rust_payload_type: rust_payload_type.as_str(),
                protocol,
                maximum_payload_bytes: *maximum_payload_bytes,
                correlation_family: correlation_family.as_str(),
            },
        ),
        ApplicationSchemaMember::OperationAftermath {
            operation,
            contract,
        } => append_operation_aftermath(basis, prefix, operation, contract),
        ApplicationSchemaMember::OperationInboundOccurrence {
            operation,
            effect,
            protocol,
            source_identity,
            limits,
        } => {
            append_operation_header(basis, prefix, "operation-inbound-occurrence", operation);
            basis.text(format!("{prefix}.effect"), effect);
            basis.text(
                format!("{prefix}.protocol-identity"),
                protocol.identity().as_str(),
            );
            basis.u32(
                format!("{prefix}.protocol-version"),
                protocol.version().get(),
            );
            basis.text(format!("{prefix}.source-identity"), source_identity);
            basis.u64(
                format!("{prefix}.maximum-envelope-bytes"),
                limits.maximum_envelope_bytes.get(),
            );
            basis.u64(
                format!("{prefix}.maximum-verifier-work"),
                limits.maximum_verifier_work.get(),
            );
            basis.u64(
                format!("{prefix}.maximum-payload-bytes"),
                limits.maximum_payload_bytes.get(),
            );
            basis.u64(
                format!("{prefix}.maximum-outstanding-dispatch-provenance"),
                limits.maximum_outstanding_dispatch_provenance.get(),
            );
            basis.u64(
                format!("{prefix}.maximum-accepted-occurrences"),
                limits.maximum_accepted_occurrences.get(),
            );
            basis.u64(
                format!("{prefix}.maximum-accepted-bytes"),
                limits.maximum_accepted_bytes.get(),
            );
            basis.u64(
                format!("{prefix}.maximum-concurrent-publications"),
                limits.maximum_concurrent_publications.get(),
            );
            basis.u64(
                format!("{prefix}.maximum-discovery-work"),
                limits.maximum_discovery_work.get(),
            );
            basis.u64(
                format!("{prefix}.replay-window-milliseconds"),
                limits.replay_window_milliseconds.get(),
            );
            basis.u64(
                format!("{prefix}.maximum-cleanup-work"),
                limits.maximum_cleanup_work.get(),
            );
        }
        _ => unreachable!("operation member router supplied another member family"),
    }
}

fn append_operation_aftermath(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    operation: &str,
    contract: &crate::application_aftermath::PortableApplicationAftermathContract,
) {
    append_operation_header(basis, prefix, "operation-aftermath", operation);
    append_declared_aftermath(basis, &format!("{prefix}.contract"), contract);
}

fn append_mutation_precondition(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    operation: &str,
    target: &crate::application_schema::ApplicationMutationPreconditionTarget,
) {
    append_operation_header(basis, prefix, "operation-mutation-precondition", operation);
    basis.text(format!("{prefix}.family"), target.family().canonical_name());
    basis.text(format!("{prefix}.entity"), target.entity());
    basis.text(format!("{prefix}.aspect"), target.aspect());
    basis.text(format!("{prefix}.field"), target.field_name());
}

fn append_projection_work_budget(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    operation: &str,
    maximum_work_units: usize,
) {
    append_operation_header(basis, prefix, "operation-projection-work-budget", operation);
    basis.usize(format!("{prefix}.maximum-work-units"), maximum_work_units);
}

fn append_operation_header(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    kind: &str,
    operation: &str,
) {
    basis.text(format!("{prefix}.kind"), kind);
    basis.text(format!("{prefix}.operation"), operation);
}

struct ExternalEffectEncoding<'a> {
    operation: &'a str,
    effect: &'a str,
    rust_payload_type: &'a str,
    protocol: &'a crate::application_schema::ApplicationExternalEffectProtocol,
    maximum_payload_bytes: u64,
    correlation_family: &'a str,
}

fn append_external_effect(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    encoding: ExternalEffectEncoding<'_>,
) {
    basis.text(format!("{prefix}.kind"), "operation-external-effect");
    basis.text(format!("{prefix}.operation"), encoding.operation);
    basis.text(format!("{prefix}.effect"), encoding.effect);
    basis.text(
        format!("{prefix}.rust-payload-type"),
        encoding.rust_payload_type,
    );
    basis.text(
        format!("{prefix}.protocol-identity"),
        encoding.protocol.identity().as_str(),
    );
    basis.u64(
        format!("{prefix}.protocol-version"),
        u64::from(encoding.protocol.version().get()),
    );
    basis.u64(
        format!("{prefix}.maximum-payload-bytes"),
        encoding.maximum_payload_bytes,
    );
    basis.text(
        format!("{prefix}.correlation-family"),
        encoding.correlation_family,
    );
}
