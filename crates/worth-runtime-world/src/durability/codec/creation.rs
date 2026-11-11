//! Creation intent framing retains the complete original source and target plans.
use super::fields::{association, head, number, Emit, Read};
use crate::branch::SignalBranchCreationPlan as Signal;
use crate::durability::artifact::{
    WorldCreationIntentRecord as Intent, WorldRecordedRelationalCreationPlan as Native,
};
use crate::durability::WorldDurableFrameDenial as Frame;
use worth_foundational::facade::DurableProductUnpublishedOwnerEffectsIdentity;
use worth_relational::facade::{
    branch::RelationalBranchTransitionSequence, history::BranchId, publication::PatchStreamPosition,
};

pub(super) fn write(out: &mut Emit<'_>, intent: &Intent) -> Result<(), Frame> {
    intent.validate_description()?;
    association(out, &intent.association)?;
    number(out, intent.custody.ordinal())?;
    head(out, &intent.source)?;
    head(out, &intent.destination)?;
    out(&intent.program_revision)?;
    match &intent.relational {
        Native::ReuseExact => out(&[0])?,
        Native::ForkExact {
            target,
            canonical_before,
            transition_before,
        } => {
            out(&[1])?;
            text(out, &target.0)?;
            number(out, canonical_before.0)?;
            number(out, transition_before.0)?;
        }
    }
    match &intent.signal {
        Signal::ReuseExact => out(&[0]),
        Signal::ForkExact { target } => {
            out(&[1])?;
            text(out, target.as_str())
        }
    }
}

pub(super) fn read(input: &mut Read<'_>) -> Result<Intent, Frame> {
    let association = input.association()?;
    let custody = DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(
        association.attempt().owner(),
        input.number()?,
    );
    let source = input.head()?;
    let destination = input.head()?;
    let program_revision = input.fixed()?;
    let relational = match input.fixed::<1>()? {
        [0] => Native::ReuseExact,
        [1] => Native::ForkExact {
            target: BranchId(read_text(input)?),
            canonical_before: PatchStreamPosition(input.number()?),
            transition_before: RelationalBranchTransitionSequence(input.number()?),
        },
        _ => return Err(Frame::Malformed),
    };
    let signal = match input.fixed::<1>()? {
        [0] => Signal::ReuseExact,
        [1] => Signal::ForkExact {
            target: worth_signal::facade::branch::validate_signal_branch_name(read_text(input)?)
                .map_err(|_| Frame::IdentityMismatch)?,
        },
        _ => return Err(Frame::Malformed),
    };
    let intent = Intent {
        association,
        custody,
        source,
        destination,
        program_revision,
        relational,
        signal,
    };
    intent.validate_description()?;
    Ok(intent)
}

fn text(out: &mut Emit<'_>, value: &str) -> Result<(), Frame> {
    number(
        out,
        u64::try_from(value.len()).map_err(|_| Frame::SizeLimit)?,
    )?;
    out(value.as_bytes())
}

fn read_text(input: &mut Read<'_>) -> Result<String, Frame> {
    let bytes = text_bytes(input)?;
    Ok(std::str::from_utf8(bytes)
        .map_err(|_| Frame::Malformed)?
        .to_owned())
}

pub(super) fn text_bytes<'a>(input: &mut Read<'a>) -> Result<&'a [u8], Frame> {
    let length = usize::try_from(input.number()?).map_err(|_| Frame::SizeLimit)?;
    let bytes = input.take(length)?;
    std::str::from_utf8(bytes).map_err(|_| Frame::Malformed)?;
    Ok(bytes)
}
