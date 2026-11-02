//! Additive record: old invariant wire forms keep their original bytes.
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::{binary_encoding::BinaryEncodingSink, binary_input::BinaryInput};
use worth_foundational::facade::{
    ManagedIndexColdSourcePosture as Cold, ManagedSourceInterpretationCommitment as Interpretation,
};
use worth_query_declaration::facade::application_query::ApplicationInvariantManagedIndexSource as Requirement;
use worth_query_declaration::facade::application_schema::{
    ApplicationInvariantExecutionPoint as Point, ApplicationSchemaMember as Member,
};

pub(super) fn write(output: &mut dyn BinaryEncodingSink, member: &Member) -> Result<(), Denial> {
    let Member::InvariantManagedIndexSource {
        invariant,
        execution_point,
        requirement,
    } = member
    else {
        unreachable!("invariant Source archive dispatch")
    };
    output.u16(requirement.interpretation().revision())?;
    output.text(invariant)?;
    output.u16(match execution_point {
        Point::CommitBoundary => 1,
        Point::MutationSensitive => 2,
        Point::SnapshotPublication => 3,
    })?;
    super::managed_index::write(output, requirement.definition())?;
    output.u16(match requirement.cold() {
        Cold::Refuse => 1,
        Cold::AdmitRebuild => 2,
    })
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut super::RecordDecodeAttempt,
) -> Result<Member, Denial> {
    Interpretation::from_revision(input.u16()?)
        .ok_or_else(|| Denial::new(Kind::UnsupportedRecordVariant))?;
    let invariant = input.text()?.to_owned();
    let execution_point = match input.u16()? {
        1 => Point::CommitBoundary,
        2 => Point::MutationSensitive,
        3 => Point::SnapshotPublication,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let Member::ManagedIndex { definition } = super::managed_index::decode(input, budget)? else {
        unreachable!("managed index decoder returns its own record")
    };
    let cold = match input.u16()? {
        1 => Cold::Refuse,
        2 => Cold::AdmitRebuild,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(Member::InvariantManagedIndexSource {
        invariant,
        execution_point,
        requirement: Requirement::from_untrusted_parts(definition, cold),
    })
}
