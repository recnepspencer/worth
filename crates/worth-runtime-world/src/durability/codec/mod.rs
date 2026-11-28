pub(in crate::durability) mod decoded;
pub(in crate::durability) mod fields;
pub(crate) use decoded::DecodedWorldRecord;
mod created_branch;
mod creation;
mod creation_name_extent;
mod initial_components;
#[cfg(test)]
pub(in crate::durability) use initial_components::bridge_decode_denial;
pub(in crate::durability) mod occurrence_backing;
pub(in crate::durability) mod owner_evidence;
pub(in crate::durability) mod owner_value;
mod planned_leg;
use super::artifact::WORLD_ARTIFACT_VERSION;
use super::{WorldDurableFrameDenial as Denial, WorldDurableMediumDenial, WorldRecord};
use fields::*;
pub(super) use fields::{association as write_association, Read};
use worth_execution::{
    decode_framed_envelope, encode_framed_array, inspect_framed_envelope, ExecutionArray,
    ExecutionRequest, FramedEnvelopeSpec, FRAMED_ENVELOPE_HEADER_BYTES,
};
use worth_foundational::facade::{HomeIdentity, HomeIncarnation};

const MAGIC: &[u8; 8] = b"WWRLDART";
const MAXIMUM: usize = 64 << 10;
const fn spec(magic: &[u8; 8]) -> FramedEnvelopeSpec {
    FramedEnvelopeSpec {
        magic: *magic,
        supported_versions: const { &[WORLD_ARTIFACT_VERSION.get()] },
        max_payload_bytes: (MAXIMUM - FRAMED_ENVELOPE_HEADER_BYTES) as u64,
    }
}

pub(crate) fn encode(
    record: &WorldRecord,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ExecutionArray<u8>, WorldDurableMediumDenial> {
    encode_payload(MAGIC, execution, &mut |out| write_record(record, out))
}

pub(super) fn encode_payload(
    magic: &[u8; 8],
    execution: &ExecutionRequest<'_, '_>,
    payload: &mut dyn FnMut(&mut Emit<'_>) -> Result<(), Denial>,
) -> Result<ExecutionArray<u8>, WorldDurableMediumDenial> {
    encode_framed_array(
        spec(magic),
        WORLD_ARTIFACT_VERSION.get(),
        execution,
        |sink| super::framed_denial::forward_payload(payload, sink),
        WorldDurableMediumDenial::Backing,
    )
}

pub(crate) fn decode(
    bytes: &[u8],
    home: HomeIdentity,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<std::sync::Arc<DecodedWorldRecord>, WorldDurableMediumDenial> {
    if bytes.len() > MAXIMUM {
        return Err(Denial::SizeLimit.into());
    }
    let (_, payload) = decode_framed_envelope(spec(MAGIC), bytes, execution)?;
    decoded::admit(payload, home, execution)
}

fn decode_record(
    payload: &[u8],
    home: HomeIdentity,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<WorldRecord, WorldDurableMediumDenial> {
    let mut read = Read::new(payload, home);
    let tag = read.fixed::<1>()?[0];
    let record = match tag {
        1 => WorldRecord::Owner {
            owner: read.owner()?,
        },
        2 => {
            let head = read.head()?;
            let bootstrap =
                worth_foundational::facade::DurableWorldBootstrapAttemptIdentity::from_recorded(
                    head.branch.owner(),
                    read.number()?,
                );
            WorldRecord::Branch {
                head,
                bootstrap,
                program_revision: read.fixed()?,
                components: read.recorded_components(execution)?,
            }
        }
        3 => WorldRecord::OpenAttempt(read.open()?),
        10 => WorldRecord::PlannedSignalLeg(planned_leg::read(&mut read, execution)?),
        7 => WorldRecord::CreationIntent(creation::read(&mut read)?),
        8 => WorldRecord::BranchCreated(created_branch::read(&mut read, execution)?),
        9 => {
            let association = read.association()?;
            let custody = worth_foundational::facade::DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(
                association.attempt().owner(), read.number()?,
            );
            WorldRecord::CreationReleased {
                association,
                custody,
            }
        }
        11 => {
            let head = read.head()?;
            let retiring_incarnation =
                HomeIncarnation::from_recorded(read.number()?).ok_or(Denial::Malformed)?;
            WorldRecord::BranchRetired {
                head,
                retiring_incarnation,
            }
        }
        12 => {
            let owner = read.owner()?;
            let retiring_incarnation =
                HomeIncarnation::from_recorded(read.number()?).ok_or(Denial::Malformed)?;
            let count = usize::try_from(read.number()?).map_err(|_| Denial::SizeLimit)?;
            if count == 0 || count > read.remaining() / 8 {
                return Err(Denial::Malformed.into());
            }
            let mut commits = Vec::new();
            commits
                .try_reserve_exact(count)
                .map_err(|_| WorldDurableMediumDenial::NameAllocation)?;
            for _ in 0..count {
                commits.push(
                    worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
                        owner,
                        read.number()?,
                    ),
                );
            }
            WorldRecord::HistoryRetired {
                owner,
                commits,
                retiring_incarnation,
            }
        }
        4 => {
            let open = read.open()?;
            let owners = owner_evidence::read(&mut read, execution)?;
            let publishing_incarnation =
                HomeIncarnation::from_recorded(read.number()?).ok_or(Denial::Malformed)?;
            WorldRecord::Occurrence {
                open,
                owners,
                publishing_incarnation,
            }
        }
        5 | 6 => {
            let association = read.association()?;
            let custody = worth_foundational::facade::DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(association.attempt().owner(), read.number()?);
            if tag == 5 {
                WorldRecord::Released {
                    association,
                    custody,
                    relational: match read.fixed::<1>()? {
                        [0] => None,
                        [1] => Some(read.release_evidence(execution)?),
                        _ => return Err(Denial::Malformed.into()),
                    },
                }
            } else {
                WorldRecord::Continued {
                    association,
                    custody,
                }
            }
        }
        _ => return Err(Denial::Malformed.into()),
    };
    read.finished()?;
    Ok(record)
}
pub(super) fn association_payload<'a>(
    bytes: &'a [u8],
    magic: &[u8; 8],
) -> Result<&'a [u8], Denial> {
    inspect_framed_envelope(spec(magic), bytes, |_| {
        Ok::<_, worth_execution::FramedEnvelopeError>(())
    })
    .map(|(_, payload)| payload)
    .map_err(|cause| super::framed_denial::wire(cause, WORLD_ARTIFACT_VERSION))
}

pub(in crate::durability) fn write_record(
    record: &WorldRecord,
    out: &mut Emit<'_>,
) -> Result<(), Denial> {
    match record {
        WorldRecord::Owner { owner: value } => {
            out(&[1])?;
            owner(out, *value)
        }
        WorldRecord::Branch {
            head: value,
            bootstrap,
            program_revision,
            components,
        } => {
            out(&[2])?;
            head(out, value)?;
            number(out, bootstrap.ordinal())?;
            out(program_revision)?;
            initial_components::write(out, components)
        }
        WorldRecord::OpenAttempt(value) => {
            out(&[3])?;
            open(out, value)
        }
        WorldRecord::PlannedSignalLeg(planned) => {
            out(&[10])?;
            planned_leg::write(out, planned)
        }
        WorldRecord::CreationIntent(intent) => {
            out(&[7])?;
            creation::write(out, intent)
        }
        WorldRecord::BranchCreated(created) => {
            out(&[8])?;
            created_branch::write(out, created)
        }
        WorldRecord::CreationReleased {
            association: value,
            custody,
        } => {
            out(&[9])?;
            association(out, value)?;
            number(out, custody.ordinal())
        }
        WorldRecord::BranchRetired {
            head: value,
            retiring_incarnation,
        } => {
            out(&[11])?;
            head(out, value)?;
            number(out, retiring_incarnation.get())
        }
        WorldRecord::HistoryRetired {
            owner: value,
            commits,
            retiring_incarnation,
        } => {
            out(&[12])?;
            owner(out, *value)?;
            number(out, retiring_incarnation.get())?;
            number(out, commits.len() as u64)?;
            for commit in commits {
                number(out, commit.ordinal())?;
            }
            Ok(())
        }
        WorldRecord::Occurrence {
            open: value,
            owners,
            publishing_incarnation,
        } => {
            out(&[4])?;
            open(out, value)?;
            owner_evidence::write(out, owners)?;
            number(out, publishing_incarnation.get())
        }
        WorldRecord::Released {
            association: value,
            custody,
            relational,
        } => {
            out(&[5])?;
            association(out, value)?;
            number(out, custody.ordinal())?;
            match relational {
                None => out(&[0]),
                Some(relational) => {
                    out(&[1])?;
                    owner_value::emit(out, relational)
                }
            }
        }
        WorldRecord::Continued {
            association: value,
            custody,
        } => {
            out(&[6])?;
            association(out, value)?;
            number(out, custody.ordinal())
        }
    }
}
