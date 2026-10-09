use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;
use crate::application_schema::canonical_decision_read_identity::append_decision_read_target;
use crate::application_schema::canonical_operation_identity::append_operation_target;
use crate::application_schema::ApplicationSchemaMember;

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
