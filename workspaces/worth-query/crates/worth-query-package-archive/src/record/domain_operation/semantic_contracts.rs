use worth_query_installation::facade::*;

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::decode_budget::RecordDecodeAttempt;
use crate::record::sequence::{decode_sequence, write_sequence};

use super::artifact_reference::{decode_reference, write_reference};
use super::input_contracts::{decode_native_projection, write_native_projection};

mod lifecycle;
mod vocabulary;
pub(super) use lifecycle::{
    decode_lifecycle, decode_replay, decode_support_lowering, decode_terminal_cost,
    write_lifecycle, write_replay, write_support_lowering, write_terminal_cost,
};
use vocabulary::{effect, effect_tag};
mod decision_facts;
mod invariant_execution;
#[cfg(test)]
mod optional_tests;
pub(super) use decision_facts::{decode_decision_facts, write_decision_facts};
pub(super) use invariant_execution::{decode_invariant_execution, write_invariant_execution};

pub(super) fn write_evidence(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryDomainEvidenceContract,
) -> Result<(), Denial> {
    match value {
        WorthQueryDomainEvidenceContract::NotRequired => output.u16(1),
        WorthQueryDomainEvidenceContract::InstalledArtifact(reference) => {
            output.u16(2)?;
            write_reference(output, reference)
        }
    }
}

pub(super) fn decode_evidence(
    input: &mut BinaryInput<'_>,
) -> Result<WorthQueryDomainEvidenceContract, Denial> {
    match input.u16()? {
        1 => Ok(WorthQueryDomainEvidenceContract::NotRequired),
        2 => Ok(WorthQueryDomainEvidenceContract::InstalledArtifact(
            decode_reference(input)?,
        )),
        _ => unsupported(),
    }
}

pub(super) fn write_graph_reads(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryOperationGraphReadContract,
) -> Result<(), Denial> {
    match value {
        WorthQueryOperationGraphReadContract::NotRequired => output.u16(1),
        WorthQueryOperationGraphReadContract::DeclaredDomain { roles } => {
            output.u16(2)?;
            write_sequence(output, roles, |output, role| {
                output.text(&role.role)?;
                write_participation(output, &role.participation)?;
                output.u16(match role.access {
                    WorthQueryOperationGraphAccess::Observe => 1,
                    WorthQueryOperationGraphAccess::Project => 2,
                })?;
                write_sequence(output, &role.semantic_reads, write_native_projection)
            })
        }
        WorthQueryOperationGraphReadContract::Declared { .. } => invalid(),
    }
}

pub(super) fn decode_graph_reads(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryOperationGraphReadContract, Denial> {
    match input.u16()? {
        1 => Ok(WorthQueryOperationGraphReadContract::NotRequired),
        2 => Ok(WorthQueryOperationGraphReadContract::DeclaredDomain {
            roles: decode_sequence(input, budget, 10, |input, budget| {
                Ok(WorthQueryDomainOperationGraphReadRole {
                    role: input.text()?.to_owned(),
                    participation: decode_participation(input)?,
                    access: match input.u16()? {
                        1 => WorthQueryOperationGraphAccess::Observe,
                        2 => WorthQueryOperationGraphAccess::Project,
                        _ => return unsupported(),
                    },
                    semantic_reads: decode_sequence(input, budget, 20, decode_native_projection)?,
                })
            })?,
        }),
        _ => unsupported(),
    }
}

pub(super) fn write_touches(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryOperationTouchContract,
) -> Result<(), Denial> {
    match value {
        WorthQueryOperationTouchContract::NotRequired => output.u16(1),
        WorthQueryOperationTouchContract::Declared {
            graph_roles,
            scopes,
        } => {
            output.u16(2)?;
            write_sequence(output, graph_roles, |output, role| output.text(role))?;
            write_sequence(output, scopes, |output, scope| match scope {
                WorthQueryOperationTouchScope::DeclaredDomain(identity) => {
                    output.u16(1)?;
                    output.text(identity.as_str())
                }
                _ => invalid(),
            })
        }
    }
}

pub(super) fn decode_touches(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryOperationTouchContract, Denial> {
    match input.u16()? {
        1 => Ok(WorthQueryOperationTouchContract::NotRequired),
        2 => {
            let graph_roles =
                decode_sequence(input, budget, 4, |input, _| Ok(input.text()?.to_owned()))?;
            let scopes = decode_sequence(input, budget, 6, |input, _| match input.u16()? {
                1 => Ok(WorthQueryOperationTouchScope::DeclaredDomain(
                    WorthQueryDeclaredDomainTouchScopeIdentity::new(input.text()?.to_owned())
                        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?,
                )),
                _ => unsupported(),
            })?;
            Ok(WorthQueryOperationTouchContract::Declared {
                graph_roles,
                scopes,
            })
        }
        _ => unsupported(),
    }
}

pub(super) fn write_effects(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryOperationEffectContract,
) -> Result<(), Denial> {
    match value {
        WorthQueryOperationEffectContract::NotRequired => output.u16(1),
        WorthQueryOperationEffectContract::Declared { effect_families } => {
            output.u16(2)?;
            write_sequence(output, effect_families, |output, family| {
                output.u16(effect_tag(*family))
            })
        }
    }
}

pub(super) fn decode_effects(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryOperationEffectContract, Denial> {
    match input.u16()? {
        1 => Ok(WorthQueryOperationEffectContract::NotRequired),
        2 => Ok(WorthQueryOperationEffectContract::Declared {
            effect_families: decode_sequence(input, budget, 2, |input, _| effect(input.u16()?))?,
        }),
        _ => unsupported(),
    }
}

pub(super) fn write_invariants(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryOperationInvariantContract,
) -> Result<(), Denial> {
    match value {
        WorthQueryOperationInvariantContract::NotRequired => output.u16(1),
        WorthQueryOperationInvariantContract::Declared { invariant_slots } => {
            output.u16(2)?;
            write_sequence(output, invariant_slots, |output, slot| output.text(slot))
        }
    }
}

pub(super) fn decode_invariants(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryOperationInvariantContract, Denial> {
    match input.u16()? {
        1 => Ok(WorthQueryOperationInvariantContract::NotRequired),
        2 => Ok(WorthQueryOperationInvariantContract::Declared {
            invariant_slots: decode_sequence(input, budget, 4, |input, _| {
                Ok(input.text()?.to_owned())
            })?,
        }),
        _ => unsupported(),
    }
}

fn write_participation(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryOperationGraphParticipation,
) -> Result<(), Denial> {
    match value {
        WorthQueryOperationGraphParticipation::PrimaryLogicalGraph => output.u16(1),
        WorthQueryOperationGraphParticipation::SeparateAuthority { role } => {
            output.u16(2)?;
            output.text(role)
        }
    }
}

fn decode_participation(
    input: &mut BinaryInput<'_>,
) -> Result<WorthQueryOperationGraphParticipation, Denial> {
    match input.u16()? {
        1 => Ok(WorthQueryOperationGraphParticipation::PrimaryLogicalGraph),
        2 => Ok(WorthQueryOperationGraphParticipation::SeparateAuthority {
            role: input.text()?.to_owned(),
        }),
        _ => unsupported(),
    }
}

fn write_usize(output: &mut dyn BinaryEncodingSink, value: usize) -> Result<(), Denial> {
    output.u64(u64::try_from(value).map_err(|_| Denial::new(Kind::NumericWidthExceeded))?)
}
fn read_usize(input: &mut BinaryInput<'_>) -> Result<usize, Denial> {
    usize::try_from(input.u64()?).map_err(|_| Denial::new(Kind::NumericWidthExceeded))
}
fn unsupported<T>() -> Result<T, Denial> {
    Err(Denial::new(Kind::UnsupportedRecordVariant))
}
fn invalid<T>() -> Result<T, Denial> {
    Err(Denial::new(Kind::InvalidRecordShape))
}
