use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectProtocol, ApplicationInboundOccurrenceLimits,
    ApplicationInboundOccurrenceProtocol, ApplicationMutationPreconditionFamily,
    ApplicationMutationPreconditionTarget, ApplicationOperationDecisionReadTarget,
    ApplicationOperationProgramTarget, ApplicationSchemaMember,
    WorthQueryExternalEffectCorrelationFamily,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use super::super::super::decode_budget::RecordDecodeAttempt;
use super::super::wire_vocabulary::{
    decode_type_identity, decode_usize, write_type_identity, write_usize,
};

mod aftermath;

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    member: &ApplicationSchemaMember,
) -> Result<(), Denial> {
    match member {
        ApplicationSchemaMember::Operation {
            operation,
            input_type,
        } => {
            output.text(operation)?;
            write_type_identity(output, input_type)
        }
        ApplicationSchemaMember::OperationProgram { operation, target } => {
            output.text(operation)?;
            write_program_target(output, target)
        }
        ApplicationSchemaMember::OperationDecisionRead { operation, target } => {
            output.text(operation)?;
            write_decision_read_target(output, target)
        }
        ApplicationSchemaMember::OperationMutationPrecondition { operation, target } => {
            output.text(operation)?;
            write_precondition_target(output, target)
        }
        ApplicationSchemaMember::OperationProjectionWorkBudget {
            operation,
            maximum_work_units,
        } => {
            output.text(operation)?;
            write_usize(output, *maximum_work_units)
        }
        ApplicationSchemaMember::OperationExternalEffect {
            operation,
            effect,
            rust_payload_type,
            protocol,
            maximum_payload_bytes,
            correlation_family,
        } => {
            output.text(operation)?;
            output.text(effect)?;
            write_type_identity(output, rust_payload_type)?;
            output.text(protocol.identity().as_str())?;
            output.u32(protocol.version().get())?;
            output.u64(*maximum_payload_bytes)?;
            output.text(correlation_family.as_str())
        }
        ApplicationSchemaMember::OperationAftermath {
            operation,
            contract,
        } => {
            output.text(operation)?;
            aftermath::write(output, contract)
        }
        ApplicationSchemaMember::OperationInboundOccurrence {
            operation,
            effect,
            protocol,
            source_identity,
            limits,
        } => {
            output.text(operation)?;
            output.text(effect)?;
            output.text(protocol.identity().as_str())?;
            output.u32(protocol.version().get())?;
            output.text(source_identity)?;
            output.u64(limits.maximum_envelope_bytes.get())?;
            output.u64(limits.maximum_verifier_work.get())?;
            output.u64(limits.maximum_payload_bytes.get())?;
            output.u64(limits.maximum_outstanding_dispatch_provenance.get())?;
            output.u64(limits.maximum_accepted_occurrences.get())?;
            output.u64(limits.maximum_accepted_bytes.get())?;
            output.u64(limits.maximum_concurrent_publications.get())?;
            output.u64(limits.maximum_discovery_work.get())?;
            output.u64(limits.replay_window_milliseconds.get())?;
            output.u64(limits.maximum_cleanup_work.get())
        }
        _ => unreachable!("operation member dispatch is exhaustive"),
    }
}

pub(super) fn decode(
    tag: u16,
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationSchemaMember, Denial> {
    let operation = input.text()?.to_owned();
    Ok(match tag {
        11 => ApplicationSchemaMember::Operation {
            operation,
            input_type: decode_type_identity(input)?,
        },
        12 => ApplicationSchemaMember::OperationProgram {
            operation,
            target: decode_program_target(input)?,
        },
        13 => ApplicationSchemaMember::OperationDecisionRead {
            operation,
            target: decode_decision_read_target(input)?,
        },
        14 => ApplicationSchemaMember::OperationMutationPrecondition {
            operation,
            target: decode_precondition_target(input)?,
        },
        16 => ApplicationSchemaMember::OperationProjectionWorkBudget {
            operation,
            maximum_work_units: decode_usize(input)?,
        },
        17 => ApplicationSchemaMember::OperationExternalEffect {
            operation,
            effect: input.text()?.to_owned(),
            rust_payload_type: decode_type_identity(input)?,
            protocol: ApplicationExternalEffectProtocol::new(
                BoundaryProtocolIdentity::parse(input.text()?.to_owned())
                    .map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
                BoundaryProtocolVersion::try_new(input.u32()?)
                    .map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
            ),
            maximum_payload_bytes: input.u64()?,
            correlation_family: WorthQueryExternalEffectCorrelationFamily::new(
                input.text()?.to_owned(),
            )
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
        },
        18 => ApplicationSchemaMember::OperationAftermath {
            operation,
            contract: aftermath::decode(input, budget)?,
        },
        29 | 30 => ApplicationSchemaMember::OperationInboundOccurrence {
            operation,
            effect: input.text()?.to_owned(),
            protocol: ApplicationInboundOccurrenceProtocol::new(
                BoundaryProtocolIdentity::parse(input.text()?.to_owned())
                    .map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
                BoundaryProtocolVersion::try_new(input.u32()?)
                    .map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
            ),
            source_identity: input.text()?.to_owned(),
            limits: decode_inbound_limits(input, tag)?,
        },
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    })
}

fn decode_inbound_limits(
    input: &mut BinaryInput<'_>,
    tag: u16,
) -> Result<ApplicationInboundOccurrenceLimits, Denial> {
    let mut nonzero = || {
        std::num::NonZeroU64::new(input.u64()?).ok_or_else(|| Denial::new(Kind::InvalidRecordShape))
    };
    let maximum_envelope_bytes = nonzero()?;
    let maximum_verifier_work = if tag == 29 {
        maximum_envelope_bytes
    } else {
        nonzero()?
    };
    let limits = ApplicationInboundOccurrenceLimits {
        maximum_envelope_bytes,
        maximum_verifier_work,
        maximum_payload_bytes: nonzero()?,
        maximum_outstanding_dispatch_provenance: nonzero()?,
        maximum_accepted_occurrences: nonzero()?,
        maximum_accepted_bytes: nonzero()?,
        maximum_concurrent_publications: nonzero()?,
        maximum_discovery_work: nonzero()?,
        replay_window_milliseconds: nonzero()?,
        maximum_cleanup_work: nonzero()?,
    };
    limits
        .accommodates_payload()
        .then_some(limits)
        .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))
}

fn write_program_target(
    output: &mut dyn BinaryEncodingSink,
    target: &ApplicationOperationProgramTarget,
) -> Result<(), Denial> {
    match target {
        ApplicationOperationProgramTarget::Create { entity } => tagged_text(output, 1, entity),
        ApplicationOperationProgramTarget::Delete { entity } => tagged_text(output, 2, entity),
        ApplicationOperationProgramTarget::Write {
            entity,
            aspect,
            field,
        } => {
            output.u16(3)?;
            write_three_texts(output, entity, aspect, field)
        }
        ApplicationOperationProgramTarget::Link { relation, from, to } => {
            output.u16(4)?;
            write_three_texts(output, relation, from, to)
        }
        ApplicationOperationProgramTarget::Unlink { relation, from, to } => {
            output.u16(5)?;
            write_three_texts(output, relation, from, to)
        }
        ApplicationOperationProgramTarget::Emit { effect } => tagged_text(output, 6, effect),
    }
}

fn decode_program_target(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationOperationProgramTarget, Denial> {
    Ok(match input.u16()? {
        1 => ApplicationOperationProgramTarget::Create {
            entity: input.text()?.to_owned(),
        },
        2 => ApplicationOperationProgramTarget::Delete {
            entity: input.text()?.to_owned(),
        },
        3 => ApplicationOperationProgramTarget::Write {
            entity: input.text()?.to_owned(),
            aspect: input.text()?.to_owned(),
            field: input.text()?.to_owned(),
        },
        4 => ApplicationOperationProgramTarget::Link {
            relation: input.text()?.to_owned(),
            from: input.text()?.to_owned(),
            to: input.text()?.to_owned(),
        },
        5 => ApplicationOperationProgramTarget::Unlink {
            relation: input.text()?.to_owned(),
            from: input.text()?.to_owned(),
            to: input.text()?.to_owned(),
        },
        6 => ApplicationOperationProgramTarget::Emit {
            effect: input.text()?.to_owned(),
        },
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    })
}

fn write_decision_read_target(
    output: &mut dyn BinaryEncodingSink,
    target: &ApplicationOperationDecisionReadTarget,
) -> Result<(), Denial> {
    match target {
        ApplicationOperationDecisionReadTarget::Entity { entity } => tagged_text(output, 1, entity),
        ApplicationOperationDecisionReadTarget::Field {
            entity,
            aspect,
            field,
        } => {
            output.u16(2)?;
            write_three_texts(output, entity, aspect, field)
        }
        ApplicationOperationDecisionReadTarget::Relation { relation, from, to } => {
            output.u16(3)?;
            write_three_texts(output, relation, from, to)
        }
    }
}

fn decode_decision_read_target(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationOperationDecisionReadTarget, Denial> {
    Ok(match input.u16()? {
        1 => ApplicationOperationDecisionReadTarget::Entity {
            entity: input.text()?.to_owned(),
        },
        2 => ApplicationOperationDecisionReadTarget::Field {
            entity: input.text()?.to_owned(),
            aspect: input.text()?.to_owned(),
            field: input.text()?.to_owned(),
        },
        3 => ApplicationOperationDecisionReadTarget::Relation {
            relation: input.text()?.to_owned(),
            from: input.text()?.to_owned(),
            to: input.text()?.to_owned(),
        },
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    })
}

fn write_precondition_target(
    output: &mut dyn BinaryEncodingSink,
    target: &ApplicationMutationPreconditionTarget,
) -> Result<(), Denial> {
    output.u16(match target.family() {
        ApplicationMutationPreconditionFamily::ExpectedVersion => 1,
        ApplicationMutationPreconditionFamily::ExpectedFact => 2,
    })?;
    write_three_texts(
        output,
        target.entity(),
        target.aspect(),
        target.field_name(),
    )
}

fn decode_precondition_target(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationMutationPreconditionTarget, Denial> {
    let family = match input.u16()? {
        1 => ApplicationMutationPreconditionFamily::ExpectedVersion,
        2 => ApplicationMutationPreconditionFamily::ExpectedFact,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(
        ApplicationMutationPreconditionTarget::from_untrusted_fields(
            family,
            input.text()?.to_owned(),
            input.text()?.to_owned(),
            input.text()?.to_owned(),
        ),
    )
}

fn tagged_text(output: &mut dyn BinaryEncodingSink, tag: u16, value: &str) -> Result<(), Denial> {
    output.u16(tag)?;
    output.text(value)
}
fn write_three_texts(
    output: &mut dyn BinaryEncodingSink,
    first: &str,
    second: &str,
    third: &str,
) -> Result<(), Denial> {
    output.text(first)?;
    output.text(second)?;
    output.text(third)
}
