use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
mod condition;
mod dependency;
mod vocabulary;

use worth_query_installation::facade::*;

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::record::decode_budget::RecordDecodeAttempt;
use crate::record::foundational_aspect::{decode_aspect_contract, write_aspect_contract};
use crate::record::sequence::{decode_sequence, write_sequence};

use super::workflow_value::{decode_value, write_value};
use condition::{decode_condition, write_condition};
use dependency::{decode_dependency, write_dependency};
use vocabulary::*;

pub(super) fn write_nodes(
    output: &mut dyn BinaryEncodingSink,
    nodes: &[WorthQueryPortableConditionalNodeDeclaration],
) -> Result<(), Denial> {
    write_sequence(output, nodes, write_node)
}

pub(super) fn decode_nodes(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<Vec<WorthQueryPortableConditionalNodeDeclaration>, Denial> {
    decode_sequence(input, budget, 40, decode_node)
}

fn write_node(
    output: &mut dyn BinaryEncodingSink,
    node: &WorthQueryPortableConditionalNodeDeclaration,
) -> Result<(), Denial> {
    output.text(node.identity())?;
    output.u16(role_tag(node.role()))?;
    write_sequence(output, node.dependencies(), write_dependency)?;
    write_sequence(output, node.outputs(), write_output)?;
    write_sequence(output, node.required_context(), |output, value| {
        output.u16(context_tag(*value))
    })?;
    write_condition(output, node.condition())?;
    write_trigger(output, node.trigger())?;
    write_comparator(output, node.dependency_comparator())?;
    write_output_equivalence(output, node.output_equivalence())?;
    write_artifact_equivalence(output, node.artifact_reuse_equivalence())?;
    output.u16(maintenance_tag(node.maintenance()))?;
    output.u16(artifact_tag(node.artifact()))?;
    output.u16(relationship_tag(node.output_relationship()))
}

fn decode_node(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryPortableConditionalNodeDeclaration, Denial> {
    Ok(
        WorthQueryPortableConditionalNodeDeclaration::from_untrusted_parts(
            ARCHIVE_DECODE,
            WorthQueryPortableConditionalNodeParts {
                identity: input.text()?.to_owned(),
                role: role(input.u16()?)?,
                dependencies: decode_sequence(input, budget, 20, decode_dependency)?,
                outputs: decode_sequence(input, budget, 4, decode_output)?,
                required_context: decode_sequence(input, budget, 2, |input, _| {
                    context(input.u16()?)
                })?,
                condition: decode_condition(input, budget)?,
                trigger: decode_trigger(input)?,
                dependency_comparator: decode_comparator(input)?,
                output_equivalence: decode_output_equivalence(input)?,
                artifact_reuse_equivalence: decode_artifact_equivalence(input)?,
                maintenance: maintenance(input.u16()?)?,
                artifact: artifact(input.u16()?)?,
                output_relationship: relationship(input.u16()?)?,
            },
        ),
    )
}

fn write_output(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryConditionalNodeOutput,
) -> Result<(), Denial> {
    match value {
        WorthQueryConditionalNodeOutput::DerivedAspect {
            contract,
            locality,
            consequences,
        } => {
            output.u16(1)?;
            write_aspect_contract(output, contract)?;
            dependency::write_locality(output, locality)?;
            write_sequence(output, consequences, write_consequence)
        }
        WorthQueryConditionalNodeOutput::OperationOutput { projection_role } => {
            output.u16(2)?;
            output.text(projection_role.as_str())
        }
        WorthQueryConditionalNodeOutput::WorkflowStageOutput { contract } => {
            output.u16(3)?;
            write_value(output, contract)
        }
    }
}

fn decode_output(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryConditionalNodeOutput, Denial> {
    match input.u16()? {
        1 => Ok(WorthQueryConditionalNodeOutput::DerivedAspect {
            contract: decode_aspect_contract(input, budget)?,
            locality: dependency::decode_locality(input)?,
            consequences: decode_sequence(input, budget, 2, decode_consequence)?,
        }),
        2 => Ok(WorthQueryConditionalNodeOutput::OperationOutput {
            projection_role: WorthQueryOperationProjectionRole::new(input.text()?.to_owned())
                .map_err(|_| invalid())?,
        }),
        3 => Ok(WorthQueryConditionalNodeOutput::WorkflowStageOutput {
            contract: decode_value(input)?,
        }),
        _ => unsupported(),
    }
}

fn write_consequence(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryConditionalConsequenceRole,
) -> Result<(), Denial> {
    match value {
        WorthQueryConditionalConsequenceRole::DerivedOnly => output.u16(1),
        WorthQueryConditionalConsequenceRole::Touch(touch) => {
            output.u16(2)?;
            output.text(touch.graph_role())?;
            output.text(touch.scope())
        }
        WorthQueryConditionalConsequenceRole::Effect(effect) => {
            output.u16(3)?;
            output.u16(effect_tag(*effect))
        }
    }
}

fn decode_consequence(
    input: &mut BinaryInput<'_>,
    _budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryConditionalConsequenceRole, Denial> {
    match input.u16()? {
        1 => Ok(WorthQueryConditionalConsequenceRole::DerivedOnly),
        2 => Ok(WorthQueryConditionalConsequenceRole::Touch(
            WorthQueryConditionalTouchRole::new(input.text()?.to_owned(), input.text()?.to_owned())
                .map_err(|_| invalid())?,
        )),
        3 => Ok(WorthQueryConditionalConsequenceRole::Effect(effect(
            input.u16()?,
        )?)),
        _ => unsupported(),
    }
}

fn write_trigger(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryConditionalTrigger,
) -> Result<(), Denial> {
    match value {
        WorthQueryConditionalTrigger::DependencyChange => output.u16(1),
        WorthQueryConditionalTrigger::OnDemand(family) => {
            output.u16(2)?;
            output.text(family.as_str())
        }
        WorthQueryConditionalTrigger::Temporal(wake) => {
            output.u16(3)?;
            output.u16(temporal_wake_tag(*wake))
        }
    }
}

fn decode_trigger(input: &mut BinaryInput<'_>) -> Result<WorthQueryConditionalTrigger, Denial> {
    match input.u16()? {
        1 => Ok(WorthQueryConditionalTrigger::DependencyChange),
        2 => Ok(WorthQueryConditionalTrigger::OnDemand(decode_family(
            input,
        )?)),
        3 => Ok(WorthQueryConditionalTrigger::Temporal(temporal_wake(
            input.u16()?,
        )?)),
        _ => unsupported(),
    }
}

fn write_comparator(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryComparatorRequirement,
) -> Result<(), Denial> {
    match value {
        WorthQueryComparatorRequirement::ExactCanonicalValue => output.u16(1),
        WorthQueryComparatorRequirement::FoundationalContractEquivalence => output.u16(2),
        WorthQueryComparatorRequirement::Registered(family) => {
            output.u16(3)?;
            output.text(family.as_str())
        }
    }
}

fn decode_comparator(
    input: &mut BinaryInput<'_>,
) -> Result<WorthQueryComparatorRequirement, Denial> {
    match input.u16()? {
        1 => Ok(WorthQueryComparatorRequirement::ExactCanonicalValue),
        2 => Ok(WorthQueryComparatorRequirement::FoundationalContractEquivalence),
        3 => Ok(WorthQueryComparatorRequirement::Registered(decode_family(
            input,
        )?)),
        _ => unsupported(),
    }
}

fn write_output_equivalence(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryOutputEquivalenceRequirement,
) -> Result<(), Denial> {
    match value {
        WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue => output.u16(1),
        WorthQueryOutputEquivalenceRequirement::FoundationalContractEquivalence => output.u16(2),
        WorthQueryOutputEquivalenceRequirement::OutputIdentity => output.u16(3),
        WorthQueryOutputEquivalenceRequirement::Registered(family) => {
            output.u16(4)?;
            output.text(family.as_str())
        }
    }
}

fn decode_output_equivalence(
    input: &mut BinaryInput<'_>,
) -> Result<WorthQueryOutputEquivalenceRequirement, Denial> {
    match input.u16()? {
        1 => Ok(WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue),
        2 => Ok(WorthQueryOutputEquivalenceRequirement::FoundationalContractEquivalence),
        3 => Ok(WorthQueryOutputEquivalenceRequirement::OutputIdentity),
        4 => Ok(WorthQueryOutputEquivalenceRequirement::Registered(
            decode_family(input)?,
        )),
        _ => unsupported(),
    }
}

fn write_artifact_equivalence(
    output: &mut dyn BinaryEncodingSink,
    value: &WorthQueryArtifactReuseEquivalence,
) -> Result<(), Denial> {
    match value {
        WorthQueryArtifactReuseEquivalence::NotReusable => output.u16(1),
        WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent => output.u16(2),
        WorthQueryArtifactReuseEquivalence::OutputEquivalent => output.u16(3),
        WorthQueryArtifactReuseEquivalence::Registered(family) => {
            output.u16(4)?;
            output.text(family.as_str())
        }
    }
}

fn decode_artifact_equivalence(
    input: &mut BinaryInput<'_>,
) -> Result<WorthQueryArtifactReuseEquivalence, Denial> {
    match input.u16()? {
        1 => Ok(WorthQueryArtifactReuseEquivalence::NotReusable),
        2 => Ok(WorthQueryArtifactReuseEquivalence::DependencyAndOutputEquivalent),
        3 => Ok(WorthQueryArtifactReuseEquivalence::OutputEquivalent),
        4 => Ok(WorthQueryArtifactReuseEquivalence::Registered(
            decode_family(input)?,
        )),
        _ => unsupported(),
    }
}

fn decode_family(input: &mut BinaryInput<'_>) -> Result<WorthQueryTypedFamilyIdentity, Denial> {
    WorthQueryTypedFamilyIdentity::from_untrusted_portable_identity(
        ARCHIVE_DECODE,
        input.text()?.to_owned(),
    )
    .map_err(|_| invalid())
}

fn unsupported<T>() -> Result<T, Denial> {
    Err(Denial::new(Kind::UnsupportedRecordVariant))
}
fn invalid() -> Denial {
    Denial::new(Kind::InvalidRecordShape)
}
