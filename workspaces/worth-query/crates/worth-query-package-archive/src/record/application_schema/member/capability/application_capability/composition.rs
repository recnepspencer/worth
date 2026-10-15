use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityAcceptedValues, ApplicationCapabilityActorComposition,
    ApplicationCapabilityAllowRule, ApplicationCapabilityComposition,
    ApplicationCapabilityConflictRule, ApplicationCapabilityDecisionComposition,
    ApplicationCapabilityDelegationDepth, ApplicationCapabilityDelegationRule,
    ApplicationCapabilityDenyRule, ApplicationCapabilityDisclosureRule,
    ApplicationCapabilityDistinctActorRule, ApplicationCapabilityGraphClause,
    ApplicationCapabilityGraphRequirement, ApplicationCapabilityGraphRule,
    ApplicationCapabilityPropagationComposition, ApplicationCapabilityScopeGuard,
    ApplicationCapabilitySeparationOfDutyRule,
    WorthQueryPortableApplicationCapabilityAcceptedValuesParts,
    WorthQueryPortableApplicationCapabilityGraphClauseParts,
    WorthQueryPortableApplicationCapabilityGraphRequirementParts,
    WorthQueryPortableApplicationCapabilityGraphRuleParts,
    WorthQueryPortableApplicationCapabilityScopeGuardParts,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use super::super::super::super::super::decode_budget::RecordDecodeAttempt;
use super::super::super::super::super::foundational_value;
use super::super::super::super::super::sequence::{decode_sequence, write_sequence};
use super::bindings;

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityComposition,
) -> Result<(), Denial> {
    write_graph_rule(output, value.decision().allow().graph())?;
    write_optional_graph(output, value.decision().deny().graph())?;
    write_optional_graph(output, value.decision().conflict().graph())?;
    write_optional_graph(output, value.actors().separation_of_duty().graph())?;
    write_optional_graph(output, value.actors().distinct_actor().graph())?;
    output.text(
        value
            .actors()
            .recorded_actor()
            .requirement()
            .map_or("not-applicable", |requirement| {
                requirement.persistence_identity()
            }),
    )?;
    write_delegation_rule(output, value.propagation().delegation())?;
    write_disclosure(output, value.propagation().disclosure())
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationCapabilityComposition, Denial> {
    let allow = ApplicationCapabilityAllowRule::new(decode_graph_rule(input, budget)?);
    let deny = match decode_optional_graph(input, budget)? {
        None => ApplicationCapabilityDenyRule::NotApplicable,
        Some(value) => ApplicationCapabilityDenyRule::When(value),
    };
    let conflict = match decode_optional_graph(input, budget)? {
        None => ApplicationCapabilityConflictRule::NotApplicable,
        Some(value) => ApplicationCapabilityConflictRule::When(value),
    };
    let separation = match decode_optional_graph(input, budget)? {
        None => ApplicationCapabilitySeparationOfDutyRule::NotApplicable,
        Some(value) => ApplicationCapabilitySeparationOfDutyRule::When(value),
    };
    let distinct = match decode_optional_graph(input, budget)? {
        None => ApplicationCapabilityDistinctActorRule::NotApplicable,
        Some(value) => ApplicationCapabilityDistinctActorRule::When(value),
    };
    use worth_query_declaration::facade::application_capability::{
        ApplicationCapabilityRecordedActorRequirement, ApplicationCapabilityRecordedActorRule,
    };
    let recorded_actor = match input.text()? {
        "not-applicable" => ApplicationCapabilityRecordedActorRule::not_applicable(),
        identity => ApplicationCapabilityRecordedActorRule::Required(
            ApplicationCapabilityRecordedActorRequirement::from_persistence_identity(identity)
                .ok_or_else(|| Denial::new(Kind::UnsupportedRecordVariant))?,
        ),
    };
    let delegation = decode_delegation_rule(input)?;
    let disclosure = decode_disclosure(input, budget)?;
    Ok(ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(allow, deny, conflict),
        ApplicationCapabilityActorComposition::new(separation, distinct, recorded_actor),
        ApplicationCapabilityPropagationComposition::new(delegation, disclosure),
    ))
}

fn write_graph_rule(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityGraphRule,
) -> Result<(), Denial> {
    write_sequence(output, value.requirements(), write_requirement)
}
fn decode_graph_rule(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationCapabilityGraphRule, Denial> {
    Ok(ApplicationCapabilityGraphRule::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityGraphRuleParts {
            requirements: decode_sequence(input, budget, 4, decode_requirement)?,
        },
    ))
}
fn write_requirement(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityGraphRequirement,
) -> Result<(), Denial> {
    write_sequence(output, value.clauses(), write_clause)
}
fn decode_requirement(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationCapabilityGraphRequirement, Denial> {
    Ok(ApplicationCapabilityGraphRequirement::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityGraphRequirementParts {
            clauses: decode_sequence(input, budget, 8, decode_clause)?,
        },
    ))
}

fn write_clause(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityGraphClause,
) -> Result<(), Denial> {
    super::super::super::super::authorization_path::write(output, value.path())?;
    write_guard(output, value.guard())?;
    write_sequence(output, value.context_anchors(), bindings::write_anchor)
}
fn decode_clause(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationCapabilityGraphClause, Denial> {
    Ok(ApplicationCapabilityGraphClause::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityGraphClauseParts {
            path: super::super::super::super::authorization_path::decode(input, budget)?,
            guard: decode_guard(input, budget)?,
            context_anchors: decode_sequence(input, budget, 12, |input, _| {
                bindings::decode_anchor(input)
            })?,
        },
    ))
}

fn write_guard(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityScopeGuard,
) -> Result<(), Denial> {
    write_sequence(output, value.requirements(), write_accepted_values)
}
fn decode_guard(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationCapabilityScopeGuard, Denial> {
    Ok(ApplicationCapabilityScopeGuard::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityScopeGuardParts {
            requirements: decode_sequence(input, budget, 10, decode_accepted_values)?,
        },
    ))
}
fn write_accepted_values(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityAcceptedValues,
) -> Result<(), Denial> {
    bindings::write_field(output, value.field())?;
    write_sequence(
        output,
        value.values(),
        foundational_value::write_aspect_value,
    )
}
fn decode_accepted_values(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationCapabilityAcceptedValues, Denial> {
    Ok(ApplicationCapabilityAcceptedValues::from_untrusted_parts(
        WorthQueryPortableApplicationCapabilityAcceptedValuesParts {
            field: bindings::decode_field(input)?,
            values: decode_sequence(input, budget, 2, |input, _| {
                foundational_value::decode_aspect_value(input)
            })?,
        },
    ))
}

fn write_optional_graph(
    output: &mut dyn BinaryEncodingSink,
    value: Option<&ApplicationCapabilityGraphRule>,
) -> Result<(), Denial> {
    match value {
        None => output.u16(0),
        Some(value) => {
            output.u16(1)?;
            write_graph_rule(output, value)
        }
    }
}
fn decode_optional_graph(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<Option<ApplicationCapabilityGraphRule>, Denial> {
    match input.u16()? {
        0 => Ok(None),
        1 => decode_graph_rule(input, budget).map(Some),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

fn write_delegation_rule(
    output: &mut dyn BinaryEncodingSink,
    value: ApplicationCapabilityDelegationRule,
) -> Result<(), Denial> {
    match value {
        ApplicationCapabilityDelegationRule::Forbidden => output.u16(1),
        ApplicationCapabilityDelegationRule::NarrowAllDimensions { maximum_depth } => {
            output.u16(2)?;
            output.u32(maximum_depth.maximum())
        }
    }
}
fn decode_delegation_rule(
    input: &mut BinaryInput<'_>,
) -> Result<ApplicationCapabilityDelegationRule, Denial> {
    match input.u16()? {
        1 => Ok(ApplicationCapabilityDelegationRule::Forbidden),
        2 => ApplicationCapabilityDelegationDepth::new(input.u32()?)
            .map(ApplicationCapabilityDelegationRule::narrow_all_dimensions)
            .ok_or_else(|| Denial::new(Kind::InvalidRecordShape)),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

fn write_disclosure(
    output: &mut dyn BinaryEncodingSink,
    value: &ApplicationCapabilityDisclosureRule,
) -> Result<(), Denial> {
    match value {
        ApplicationCapabilityDisclosureRule::NotApplicable => output.u16(0),
        ApplicationCapabilityDisclosureRule::Permit(guards) => {
            output.u16(1)?;
            write_sequence(output, guards, write_guard)
        }
    }
}
fn decode_disclosure(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationCapabilityDisclosureRule, Denial> {
    match input.u16()? {
        0 => Ok(ApplicationCapabilityDisclosureRule::NotApplicable),
        1 => Ok(ApplicationCapabilityDisclosureRule::Permit(
            decode_sequence(input, budget, 4, decode_guard)?,
        )),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}
