use super::super::super::decode_budget::RecordDecodeAttempt;
use super::super::super::sequence::{decode_sequence, write_sequence};
use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use std::num::NonZeroU64;
use worth_query_declaration::facade::application_schema::{
    ApplicationInvariantCostPosture as Cost, ApplicationInvariantEnforcement as Enforcement,
    ApplicationInvariantExecutionPoint as Point, ApplicationInvariantGroup as Group,
    ApplicationInvariantScopeTarget as Target, ApplicationSchemaMember,
};

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    member: &ApplicationSchemaMember,
) -> Result<(), Denial> {
    let ApplicationSchemaMember::ApplicationInvariant {
        invariant,
        major,
        minor,
        execution_point,
        maximum_work_units,
        enforcement,
        required_groups,
        read_closure,
        applicability,
        provider,
        cost_posture,
    } = member
    else {
        unreachable!("invariant member dispatch is exhaustive")
    };
    output.text(invariant)?;
    output.u16(*major)?;
    output.u16(*minor)?;
    output.u16(point_tag(*execution_point))?;
    output.u64(maximum_work_units.get())?;
    output.u16(enforcement_tag(*enforcement))?;
    write_sequence(output, required_groups, |output, group| {
        output.u16(group_tag(*group))
    })?;
    write_sequence(output, read_closure, write_target)?;
    write_sequence(output, applicability, write_target)?;
    output.text(provider)?;
    output.u16(cost_tag(*cost_posture))
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<ApplicationSchemaMember, Denial> {
    budget.require_nesting_depth(2)?;
    Ok(ApplicationSchemaMember::ApplicationInvariant {
        invariant: input.text()?.to_owned(),
        major: input.u16()?,
        minor: input.u16()?,
        execution_point: decode_point(input.u16()?)?,
        maximum_work_units: NonZeroU64::new(input.u64()?)
            .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))?,
        enforcement: decode_enforcement(input.u16()?)?,
        required_groups: decode_sequence(input, budget, 2, |input, _| decode_group(input.u16()?))?,
        read_closure: decode_sequence(input, budget, 6, |input, _| decode_target(input))?,
        applicability: decode_sequence(input, budget, 6, |input, _| decode_target(input))?,
        provider: input.text()?.to_owned(),
        cost_posture: decode_cost(input.u16()?)?,
    })
}

fn write_target(output: &mut dyn BinaryEncodingSink, target: &Target) -> Result<(), Denial> {
    match target {
        Target::Entity(name) => {
            output.u16(1)?;
            output.text(name)
        }
        Target::Relation(name) => {
            output.u16(2)?;
            output.text(name)
        }
    }
}
fn decode_target(input: &mut BinaryInput<'_>) -> Result<Target, Denial> {
    match input.u16()? {
        1 => Ok(Target::Entity(input.text()?.to_owned())),
        2 => Ok(Target::Relation(input.text()?.to_owned())),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

macro_rules! wire_enum {
    ($encode:ident, $decode:ident, $ty:ident, {$($variant:ident => $tag:literal),+ $(,)?}) => {
        fn $encode(value: $ty) -> u16 { match value { $($ty::$variant => $tag),+ } }
        fn $decode(tag: u16) -> Result<$ty, Denial> { match tag {
            $($tag => Ok($ty::$variant)),+, _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
        } }
    };
}
wire_enum!(point_tag, decode_point, Point, {CommitBoundary => 1, MutationSensitive => 2, SnapshotPublication => 3});
wire_enum!(enforcement_tag, decode_enforcement, Enforcement, {BlockCommit => 1, BlockPublication => 2});
wire_enum!(cost_tag, decode_cost, Cost, {Touched => 1, Partition => 2, Global => 3});
wire_enum!(group_tag, decode_group, Group, {
    StorageCoherence => 1, VersionVisibility => 2, AdjacencyIntegrity => 3,
    IdentityCoherence => 4, SchemaCompliance => 5, LineageIntegrity => 6,
    PublicationCoherence => 7, RelationIntegrity => 8,
});
