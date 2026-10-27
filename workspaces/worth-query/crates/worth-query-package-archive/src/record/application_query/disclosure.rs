use worth_foundational::facade::{AspectMask, CanonicalFieldPath, FieldKey};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryDisclosureContract, ApplicationQueryDisclosurePosture,
    ApplicationQueryDisclosureRule, ApplicationQueryDisclosureSelector,
    ApplicationQueryInfluenceContract, ApplicationQueryObservableInfluence,
    WorthQueryPortableApplicationQueryDisclosureParts,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use crate::record::application_schema::wire_vocabulary::{
    decode_optional, decode_type_identity, write_optional, write_type_identity,
};
use crate::record::decode_budget::RecordDecodeAttempt;
use crate::record::foundational_aspect;
use crate::record::foundational_value;
use crate::record::sequence::{decode_sequence, require_canonical_sequence, write_sequence};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    contract: &ApplicationQueryDisclosureContract,
) -> Result<(), Denial> {
    output.u16(posture_tag(contract.posture()))?;
    output.text(contract.classification())?;
    write_optional(output, contract.capability_name(), |output, value| {
        output.text(value)
    })?;
    let capability_identity = contract.capability_identity();
    write_optional(output, capability_identity.as_ref(), write_type_identity)?;
    write_sequence(output, contract.rules(), write_rule)
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationQueryDisclosureContract, Denial> {
    let posture = posture_from_tag(input.u16()?)?;
    let classification = input.text()?.to_owned();
    let capability_name = decode_optional(input, |input| Ok(input.text()?.to_owned()))?;
    let capability_type = decode_optional(input, decode_type_identity)?;
    let rules = decode_sequence(input, budget, 10, decode_rule)?;
    Ok(ApplicationQueryDisclosureContract::from_untrusted_parts(
        WorthQueryPortableApplicationQueryDisclosureParts {
            posture,
            classification,
            capability_name,
            capability_type,
            rules,
        },
    ))
}

fn write_rule(
    output: &mut dyn BinaryEncodingSink,
    rule: &ApplicationQueryDisclosureRule,
) -> Result<(), Denial> {
    write_selector(output, rule.selector())?;
    foundational_value::write_aspect_value(output, rule.disclosure_value())?;
    write_sequence(
        output,
        &rule.influence().permitted().collect::<Vec<_>>(),
        |output, value| output.u16(influence_tag(*value)),
    )
}

fn decode_rule(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationQueryDisclosureRule, Denial> {
    let selector = decode_selector(input, budget)?;
    let disclosure_value = foundational_value::decode_aspect_value(input)?;
    let influences = decode_sequence(input, budget, 2, |input, _| {
        influence_from_tag(input.u16()?)
    })?;
    require_canonical_sequence(&influences)?;
    Ok(ApplicationQueryDisclosureRule::from_untrusted_fields(
        selector,
        disclosure_value,
        ApplicationQueryInfluenceContract::permit(influences),
    ))
}

fn write_selector(
    output: &mut dyn BinaryEncodingSink,
    selector: &ApplicationQueryDisclosureSelector,
) -> Result<(), Denial> {
    match selector {
        ApplicationQueryDisclosureSelector::Orbit(orbit) => {
            output.u16(4)?;
            super::orbit::write(output, orbit)
        }
        ApplicationQueryDisclosureSelector::InternalField {
            entity,
            aspect,
            field,
            projection_mask,
            diagnostic_mask,
        } => {
            output.u16(1)?;
            output.text(entity)?;
            output.text(aspect)?;
            output.text(field)?;
            write_mask(output, projection_mask)?;
            write_mask(output, diagnostic_mask)
        }
        ApplicationQueryDisclosureSelector::Field {
            query_type,
            slot_type,
            entity,
            aspect,
            field,
            output_name,
            scalar_family,
            value_type,
            presence,
            projection_mask,
            diagnostic_mask,
        } => {
            output.u16(2)?;
            write_type_identity(output, query_type)?;
            write_type_identity(output, slot_type)?;
            output.text(entity)?;
            output.text(aspect)?;
            output.text(field)?;
            output.text(output_name)?;
            foundational_aspect::write_scalar_type(output, *scalar_family)?;
            write_type_identity(output, value_type)?;
            crate::record::application_schema::wire_vocabulary::write_presence(output, *presence)?;
            write_mask(output, projection_mask)?;
            write_mask(output, diagnostic_mask)
        }
        ApplicationQueryDisclosureSelector::Relation {
            query_type,
            slot_type,
            relation,
            from,
            to,
            direction,
            cardinality,
            output_name,
        } => {
            output.u16(3)?;
            write_type_identity(output, query_type)?;
            write_type_identity(output, slot_type)?;
            output.text(relation)?;
            output.text(from)?;
            output.text(to)?;
            super::result_shape::write_traversal_direction(output, *direction)?;
            super::controls::write_cardinality(output, *cardinality)?;
            output.text(output_name)
        }
    }
}

fn decode_selector(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationQueryDisclosureSelector, Denial> {
    Ok(match input.u16()? {
        4 => ApplicationQueryDisclosureSelector::Orbit(super::orbit::decode(input)?),
        1 => ApplicationQueryDisclosureSelector::InternalField {
            entity: input.text()?.to_owned(),
            aspect: input.text()?.to_owned(),
            field: input.text()?.to_owned(),
            projection_mask: decode_mask(input, budget)?,
            diagnostic_mask: decode_mask(input, budget)?,
        },
        2 => ApplicationQueryDisclosureSelector::Field {
            query_type: decode_type_identity(input)?,
            slot_type: decode_type_identity(input)?,
            entity: input.text()?.to_owned(),
            aspect: input.text()?.to_owned(),
            field: input.text()?.to_owned(),
            output_name: input.text()?.to_owned(),
            scalar_family: foundational_aspect::decode_scalar_type(input)?,
            value_type: decode_type_identity(input)?,
            presence: crate::record::application_schema::wire_vocabulary::decode_presence(input)?,
            projection_mask: decode_mask(input, budget)?,
            diagnostic_mask: decode_mask(input, budget)?,
        },
        3 => ApplicationQueryDisclosureSelector::Relation {
            query_type: decode_type_identity(input)?,
            slot_type: decode_type_identity(input)?,
            relation: input.text()?.to_owned(),
            from: input.text()?.to_owned(),
            to: input.text()?.to_owned(),
            direction: super::result_shape::decode_traversal_direction(input)?,
            cardinality: super::controls::decode_cardinality(input)?,
            output_name: input.text()?.to_owned(),
        },
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    })
}

fn write_mask<Mode>(
    output: &mut dyn BinaryEncodingSink,
    mask: &AspectMask<Mode>,
) -> Result<(), Denial> {
    write_sequence(output, mask.paths(), |output, path| {
        write_sequence(output, path.fields(), |output, field| {
            output.text(field.as_str())
        })
    })
}

fn decode_mask<Mode>(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<AspectMask<Mode>, Denial> {
    let paths = decode_sequence(input, budget, 4, |input, budget| {
        let fields = decode_sequence(input, budget, 5, |input, _| {
            FieldKey::new(input.text()?.to_owned())
                .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))
        })?;
        CanonicalFieldPath::new(fields).ok_or_else(|| Denial::new(Kind::InvalidRecordShape))
    })?;
    require_canonical_sequence(&paths)?;
    Ok(if paths.is_empty() {
        AspectMask::whole_aspect()
    } else {
        AspectMask::new(paths)
    })
}

const fn posture_tag(value: ApplicationQueryDisclosurePosture) -> u16 {
    match value {
        ApplicationQueryDisclosurePosture::Public => 1,
        ApplicationQueryDisclosurePosture::InstalledPolicyRequired => 2,
        ApplicationQueryDisclosurePosture::Governed => 3,
    }
}
fn posture_from_tag(tag: u16) -> Result<ApplicationQueryDisclosurePosture, Denial> {
    match tag {
        1 => Ok(ApplicationQueryDisclosurePosture::Public),
        2 => Ok(ApplicationQueryDisclosurePosture::InstalledPolicyRequired),
        3 => Ok(ApplicationQueryDisclosurePosture::Governed),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

const fn influence_tag(value: ApplicationQueryObservableInfluence) -> u16 {
    match value {
        ApplicationQueryObservableInfluence::RowPresence => 1,
        ApplicationQueryObservableInfluence::Ordering => 2,
        ApplicationQueryObservableInfluence::Pagination => 3,
        ApplicationQueryObservableInfluence::Count => 4,
        ApplicationQueryObservableInfluence::Aggregate => 5,
        ApplicationQueryObservableInfluence::Explanation => 6,
        ApplicationQueryObservableInfluence::HistoricalMembership => 7,
        ApplicationQueryObservableInfluence::Preview => 8,
        ApplicationQueryObservableInfluence::LiveMembership => 9,
    }
}
fn influence_from_tag(tag: u16) -> Result<ApplicationQueryObservableInfluence, Denial> {
    match tag {
        1 => Ok(ApplicationQueryObservableInfluence::RowPresence),
        2 => Ok(ApplicationQueryObservableInfluence::Ordering),
        3 => Ok(ApplicationQueryObservableInfluence::Pagination),
        4 => Ok(ApplicationQueryObservableInfluence::Count),
        5 => Ok(ApplicationQueryObservableInfluence::Aggregate),
        6 => Ok(ApplicationQueryObservableInfluence::Explanation),
        7 => Ok(ApplicationQueryObservableInfluence::HistoricalMembership),
        8 => Ok(ApplicationQueryObservableInfluence::Preview),
        9 => Ok(ApplicationQueryObservableInfluence::LiveMembership),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}
