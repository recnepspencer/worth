//! Fixed-width recovery claim grammar; recorded bytes grant no live capability.
use super::*;
use worth_foundational::facade::{
    CorrectionMaterialPinIdentity, DurableCompositeCommitIdentity,
    DurableCompositePublicationAttemptIdentity, DurableRecoveryClaimIdentity,
    DurableWorldOwnerIdentity, RecoveryClaimDisposition, WorldOwnerSlotIdentity,
};

pub(super) fn encode(output: &mut Vec<u8>, record: &QueryRecoveryRecord) {
    match record {
        QueryRecoveryRecord::Claim(claim) => {
            output.push(7);
            for number in [
                claim.original.owner().slot().get(),
                claim.original.ordinal(),
                claim.attempt.ordinal(),
                claim.expires_at_unix_ms,
                claim.material_pin.acquire_position().get(),
            ] {
                output.extend_from_slice(&number.to_le_bytes());
            }
        }
        QueryRecoveryRecord::Disposition { claim, terminal } => {
            output.push(8);
            output.extend_from_slice(&claim.position().get().to_le_bytes());
            output.push(match terminal {
                RecoveryClaimDisposition::Consumed => 0,
                RecoveryClaimDisposition::Completed => 1,
                RecoveryClaimDisposition::Expired => 2,
                RecoveryClaimDisposition::Disposed => 3,
                RecoveryClaimDisposition::ForceTerminated => 4,
                RecoveryClaimDisposition::Relinquished => 5,
            });
        }
    }
}
pub(super) fn decode(
    tag: u8,
    home: HomeIdentity,
    reader: &mut Reader<'_>,
) -> Result<QueryRecoveryRecord, QueryDispatchJournalDenial> {
    use QueryDispatchJournalDenial::MalformedFrame;
    Ok(match tag {
        7 => {
            let owner = DurableWorldOwnerIdentity::from_recorded(
                home,
                WorldOwnerSlotIdentity::from_recorded(reader.number()?),
            );
            let original = DurableCompositeCommitIdentity::from_recorded(owner, reader.number()?);
            let attempt =
                DurableCompositePublicationAttemptIdentity::from_recorded(owner, reader.number()?);
            let expires_at_unix_ms = reader.number()?;
            let position = std::num::NonZeroU64::new(reader.number()?).ok_or(MalformedFrame)?;
            QueryRecoveryRecord::Claim(QueryRecoveryClaim {
                original,
                attempt,
                expires_at_unix_ms,
                material_pin: CorrectionMaterialPinIdentity::from_recorded(home, position),
            })
        }
        8 => {
            let position = std::num::NonZeroU64::new(reader.number()?).ok_or(MalformedFrame)?;
            let terminal = match reader.take::<1>()?[0] {
                0 => RecoveryClaimDisposition::Consumed,
                1 => RecoveryClaimDisposition::Completed,
                2 => RecoveryClaimDisposition::Expired,
                3 => RecoveryClaimDisposition::Disposed,
                4 => RecoveryClaimDisposition::ForceTerminated,
                5 => RecoveryClaimDisposition::Relinquished,
                _ => return Err(MalformedFrame),
            };
            QueryRecoveryRecord::Disposition {
                claim: DurableRecoveryClaimIdentity::from_recorded(home, position),
                terminal,
            }
        }
        _ => return Err(MalformedFrame),
    })
}
