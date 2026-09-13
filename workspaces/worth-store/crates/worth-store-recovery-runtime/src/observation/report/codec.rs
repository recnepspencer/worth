use sha2::{Digest, Sha256};

use super::super::{protocol, RecoveryReportCounters, RecoveryReportDecodeDenial};
use super::model::{
    RecoveryReportBlockCause, RecoveryReportDenialCause, RecoveryReportEnvelope,
    RecoveryReportOutcome, RecoveryReportRefusalCause,
};

impl RecoveryReportEnvelope {
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(96);
        protocol::encode_header(&mut bytes);
        bytes.push(outcome_byte(self.outcome));
        encode_optional(&mut bytes, self.store.as_ref());
        match self.root_generation {
            Some(generation) => {
                bytes.push(1);
                bytes.extend_from_slice(&generation.to_le_bytes());
            }
            None => bytes.push(0),
        }
        bytes.extend_from_slice(&self.counters.recovery_effects().to_le_bytes());
        bytes.extend_from_slice(&self.counters.cleanup_performed().to_le_bytes());
        bytes.extend_from_slice(&self.counters.cleanup_deferred().to_le_bytes());
        bytes.extend_from_slice(&self.counters.peak_recovery_bytes().to_le_bytes());
        if let Some(cause) = self.denial_cause {
            bytes.push(denial_cause_byte(cause));
        }
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        bytes.extend_from_slice(&digest);
        bytes
    }

    pub fn decode(encoded: &[u8]) -> Result<Self, RecoveryReportDecodeDenial> {
        if encoded.len() < 32 {
            return Err(RecoveryReportDecodeDenial::Malformed);
        }
        let (payload, digest) = encoded.split_at(encoded.len() - 32);
        if <[u8; 32]>::from(Sha256::digest(payload)) != digest {
            return Err(RecoveryReportDecodeDenial::DigestMismatch);
        }
        let mut bytes = payload;
        protocol::admit_header(&mut bytes)?;
        let outcome = decode_outcome(protocol::byte(&mut bytes)?)?;
        let store = match protocol::byte(&mut bytes)? {
            0 => None,
            1 => Some(protocol::array::<16>(&mut bytes)?),
            _ => return Err(RecoveryReportDecodeDenial::Malformed),
        };
        let root_generation = match protocol::byte(&mut bytes)? {
            0 => None,
            1 => Some(protocol::u64_value(&mut bytes)?),
            _ => return Err(RecoveryReportDecodeDenial::Malformed),
        };
        let counters = RecoveryReportCounters::new(
            protocol::u64_value(&mut bytes)?,
            protocol::u64_value(&mut bytes)?,
            protocol::u64_value(&mut bytes)?,
            protocol::u64_value(&mut bytes)?,
        );
        let denial_cause = (!bytes.is_empty())
            .then(|| decode_denial_cause(protocol::byte(&mut bytes)?))
            .transpose()?;
        if !bytes.is_empty() {
            return Err(RecoveryReportDecodeDenial::Malformed);
        }
        if !semantically_valid(outcome, store, root_generation, counters, denial_cause) {
            return Err(RecoveryReportDecodeDenial::Malformed);
        }
        Ok(Self {
            outcome,
            store,
            root_generation,
            counters,
            denial_cause,
        })
    }
}

fn semantically_valid(
    outcome: RecoveryReportOutcome,
    store: Option<[u8; 16]>,
    root_generation: Option<u64>,
    counters: RecoveryReportCounters,
    denial_cause: Option<RecoveryReportDenialCause>,
) -> bool {
    match outcome {
        RecoveryReportOutcome::Recovered => {
            store.is_some() && root_generation.is_some() && denial_cause.is_none()
        }
        RecoveryReportOutcome::Refused => {
            store.is_none()
                && root_generation.is_none()
                && counters.cleanup_performed() == 0
                && counters.cleanup_deferred() == 0
                && counters.peak_recovery_bytes() == 0
                && matches!(denial_cause, Some(RecoveryReportDenialCause::Refused(_)))
        }
        RecoveryReportOutcome::Blocked => {
            store.is_some()
                && counters.cleanup_performed() == 0
                && counters.cleanup_deferred() == 0
                && matches!(denial_cause, Some(RecoveryReportDenialCause::Blocked(_)))
        }
        RecoveryReportOutcome::PublicationIndeterminate => {
            store.is_some()
                && root_generation.is_none()
                && counters.cleanup_performed() == 0
                && counters.cleanup_deferred() == 0
                && counters.peak_recovery_bytes() == 0
                && matches!(
                    denial_cause,
                    Some(RecoveryReportDenialCause::PublicationSettlementIndeterminate)
                )
        }
    }
}

fn outcome_byte(outcome: RecoveryReportOutcome) -> u8 {
    match outcome {
        RecoveryReportOutcome::Recovered => 1,
        RecoveryReportOutcome::Refused => 2,
        RecoveryReportOutcome::Blocked => 3,
        RecoveryReportOutcome::PublicationIndeterminate => 4,
    }
}

fn decode_outcome(value: u8) -> Result<RecoveryReportOutcome, RecoveryReportDecodeDenial> {
    match value {
        1 => Ok(RecoveryReportOutcome::Recovered),
        2 => Ok(RecoveryReportOutcome::Refused),
        3 => Ok(RecoveryReportOutcome::Blocked),
        4 => Ok(RecoveryReportOutcome::PublicationIndeterminate),
        _ => Err(RecoveryReportDecodeDenial::Malformed),
    }
}

fn encode_optional(bytes: &mut Vec<u8>, value: Option<&[u8; 16]>) {
    match value {
        Some(value) => {
            bytes.push(1);
            bytes.extend_from_slice(value);
        }
        None => bytes.push(0),
    }
}

fn denial_cause_byte(cause: RecoveryReportDenialCause) -> u8 {
    match cause {
        RecoveryReportDenialCause::Refused(cause) => {
            1 + match cause {
                RecoveryReportRefusalCause::CancelledBeforeDiscovery => 0,
                RecoveryReportRefusalCause::CancelledBeforeReconstruction => 1,
                RecoveryReportRefusalCause::CancelledBeforeExecution => 2,
                RecoveryReportRefusalCause::EntryBindingDrift => 3,
                RecoveryReportRefusalCause::PersistedStoreAdmission => 4,
                RecoveryReportRefusalCause::CoordinationUnavailable => 5,
            }
        }
        RecoveryReportDenialCause::Blocked(cause) => {
            16 + match cause {
                RecoveryReportBlockCause::DiscoveryLimit => 0,
                RecoveryReportBlockCause::MediaObservation => 1,
                RecoveryReportBlockCause::RootProtocol => 2,
                RecoveryReportBlockCause::Checkpoint => 3,
                RecoveryReportBlockCause::WalInventory => 4,
                RecoveryReportBlockCause::SourceSelection => 5,
                RecoveryReportBlockCause::BindingFreshness => 6,
                RecoveryReportBlockCause::PageAdmission => 7,
                RecoveryReportBlockCause::OperationReconciliation => 8,
                RecoveryReportBlockCause::RedoPlanning => 9,
                RecoveryReportBlockCause::Staging => 10,
                RecoveryReportBlockCause::Publication => 11,
            }
        }
        RecoveryReportDenialCause::PublicationSettlementIndeterminate => 32,
    }
}

fn decode_denial_cause(value: u8) -> Result<RecoveryReportDenialCause, RecoveryReportDecodeDenial> {
    match value {
        1..=6 => Ok(RecoveryReportDenialCause::Refused(match value - 1 {
            0 => RecoveryReportRefusalCause::CancelledBeforeDiscovery,
            1 => RecoveryReportRefusalCause::CancelledBeforeReconstruction,
            2 => RecoveryReportRefusalCause::CancelledBeforeExecution,
            3 => RecoveryReportRefusalCause::EntryBindingDrift,
            4 => RecoveryReportRefusalCause::PersistedStoreAdmission,
            5 => RecoveryReportRefusalCause::CoordinationUnavailable,
            _ => unreachable!(),
        })),
        16..=27 => Ok(RecoveryReportDenialCause::Blocked(match value - 16 {
            0 => RecoveryReportBlockCause::DiscoveryLimit,
            1 => RecoveryReportBlockCause::MediaObservation,
            2 => RecoveryReportBlockCause::RootProtocol,
            3 => RecoveryReportBlockCause::Checkpoint,
            4 => RecoveryReportBlockCause::WalInventory,
            5 => RecoveryReportBlockCause::SourceSelection,
            6 => RecoveryReportBlockCause::BindingFreshness,
            7 => RecoveryReportBlockCause::PageAdmission,
            8 => RecoveryReportBlockCause::OperationReconciliation,
            9 => RecoveryReportBlockCause::RedoPlanning,
            10 => RecoveryReportBlockCause::Staging,
            11 => RecoveryReportBlockCause::Publication,
            _ => unreachable!(),
        })),
        32 => Ok(RecoveryReportDenialCause::PublicationSettlementIndeterminate),
        _ => Err(RecoveryReportDecodeDenial::Malformed),
    }
}
