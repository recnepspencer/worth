//! Schema-3 certificate roster observation. These are bounded format claims,
//! not proof of the selected descriptor route or durable WAL fate.

mod release;
mod tier;

use release::{Accumulator, Batch, Certificate, NoRelease};

use super::source::CheckpointSourceFacts;
use crate::integrity_observation::{
    record_walk::damage, sha256::Sha256, OfflineIntegrityObservationCounters,
    OfflineIntegrityOutcome as Outcome, OfflinePhysicalBlastRadius as Blast,
    OfflinePhysicalDamageCause as Cause,
};

#[derive(Clone)]
pub(crate) struct ObservedCheckpointReleaseClaim {
    kind: ReleaseClaimKind,
}

#[derive(Clone)]
enum ReleaseClaimKind {
    Released {
        #[cfg(test)]
        batches: Vec<Batch>,
        accumulator: Accumulator,
    },
    NoRelease(NoRelease),
}

impl ObservedCheckpointReleaseClaim {
    pub(crate) fn source_root_sha(&self) -> [u8; 32] {
        match &self.kind {
            ReleaseClaimKind::Released { accumulator, .. } => accumulator.root_sha,
            ReleaseClaimKind::NoRelease(marker) => marker.root_sha,
        }
    }
}

pub(super) struct CertificateState {
    digest: Sha256,
    count: u64,
    bytes: u64,
    tier_seen: bool,
    batches: Vec<Batch>,
    accumulator: Option<Accumulator>,
    no_release: Option<NoRelease>,
}

impl CertificateState {
    pub(super) fn new() -> Self {
        Self {
            digest: Sha256::new(),
            count: 0,
            bytes: 0,
            tier_seen: false,
            batches: Vec::new(),
            accumulator: None,
            no_release: None,
        }
    }

    pub(super) fn include(
        &mut self,
        kind: u8,
        payload: &[u8],
        full_record: &[u8],
        source: CheckpointSourceFacts,
    ) -> Result<(), Outcome> {
        if self.count >= 64
            || self
                .bytes
                .checked_add(full_record.len() as u64)
                .is_none_or(|n| n > 65_536)
        {
            return Err(damage(Cause::MalformedPayload, None, Blast::Artifact));
        }
        match kind {
            6 => {
                if !tier::valid(
                    payload,
                    source.identity[..16].try_into().unwrap(),
                    source.sequence,
                    source.root_generation,
                ) {
                    return Err(damage(Cause::MalformedPayload, None, Blast::Frame));
                }
                if self.tier_seen || self.count != 0 {
                    return Err(damage(Cause::DuplicateIdentity, None, Blast::Frame));
                }
                self.tier_seen = true;
            }
            7 => {
                let certificate = release::decode(payload)
                    .ok_or_else(|| damage(Cause::MalformedPayload, None, Blast::Frame))?;
                match certificate {
                    Certificate::Batch(batch) => {
                        if self.accumulator.is_some()
                            || self.no_release.is_some()
                            || batch.root_generation != source.root_generation
                            || !matches_source(
                                u64::from_le_bytes(batch.checkpoint[16..24].try_into().unwrap()),
                                batch.checkpoint[..16].try_into().unwrap(),
                                source,
                            )
                        {
                            return Err(damage(Cause::ScopeMismatch, None, Blast::Frame));
                        }
                        self.batches.push(batch);
                    }
                    Certificate::Accumulator(accumulator) => {
                        if self.accumulator.is_some()
                            || self.no_release.is_some()
                            || accumulator.root_generation != source.root_generation
                            || !matches_source(
                                u64::from_le_bytes(
                                    accumulator.checkpoint[16..24].try_into().unwrap(),
                                ),
                                accumulator.checkpoint[..16].try_into().unwrap(),
                                source,
                            )
                        {
                            return Err(damage(Cause::DuplicateIdentity, None, Blast::Frame));
                        }
                        self.accumulator = Some(accumulator);
                    }
                    Certificate::NoRelease(marker) => {
                        if self.no_release.is_some() {
                            return Err(damage(Cause::DuplicateIdentity, None, Blast::Frame));
                        }
                        if !self.batches.is_empty() || self.accumulator.is_some() {
                            return Err(damage(Cause::ScopeMismatch, None, Blast::Frame));
                        }
                        if marker.checkpoint != source.identity
                            || marker.root_generation != source.root_generation
                        {
                            return Err(damage(Cause::ScopeMismatch, None, Blast::Frame));
                        }
                        self.no_release = Some(marker);
                    }
                }
            }
            _ => return Err(damage(Cause::Framing, None, Blast::Frame)),
        }
        self.digest.update(full_record);
        self.count += 1;
        self.bytes += full_record.len() as u64;
        Ok(())
    }

    pub(super) fn finish(
        self,
        footer: &[u8],
        counters: &mut OfflineIntegrityObservationCounters,
    ) -> Result<Option<ObservedCheckpointReleaseClaim>, Outcome> {
        let read_u64 = |offset| u64::from_le_bytes(footer[offset..offset + 8].try_into().unwrap());
        if read_u64(136) != self.count || read_u64(144) != self.bytes {
            return Err(damage(Cause::ScopeMismatch, None, Blast::Artifact));
        }
        counters.checksum_calculations += 1;
        if self.digest.finish() != footer[152..184] {
            return Err(damage(Cause::ChecksumMismatch, None, Blast::Artifact));
        }
        if let Some(marker) = self.no_release {
            return Ok(Some(ObservedCheckpointReleaseClaim {
                kind: ReleaseClaimKind::NoRelease(marker),
            }));
        }
        match (self.batches.is_empty(), self.accumulator) {
            (true, None) => Ok(None),
            (false, None) => Err(damage(Cause::MissingArtifact, None, Blast::Artifact)),
            (_, Some(accumulator)) => {
                if self.batches.len() != usize::from(accumulator.batch_count) {
                    return Err(damage(Cause::ScopeMismatch, None, Blast::Artifact));
                }
                if let Some(last) = self.batches.last() {
                    let digest = release::batch_digest(&self.batches)
                        .ok_or_else(|| damage(Cause::DuplicateIdentity, None, Blast::Artifact))?;
                    if digest != accumulator.batch_digest
                        || last.tip != accumulator.tip
                        || last.cumulative != accumulator.cumulative
                        || last.cumulative_digest != accumulator.cumulative_digest
                        || last.terminal != accumulator.terminal
                        || last.root_generation != accumulator.root_generation
                        || last.root_sha != accumulator.root_sha
                    {
                        return Err(damage(Cause::ScopeMismatch, None, Blast::Artifact));
                    }
                }
                Ok(Some(ObservedCheckpointReleaseClaim {
                    kind: ReleaseClaimKind::Released {
                        #[cfg(test)]
                        batches: self.batches,
                        accumulator,
                    },
                }))
            }
        }
    }
}

fn matches_source(sequence: u64, store: [u8; 16], source: CheckpointSourceFacts) -> bool {
    sequence == source.sequence && store.as_slice() == &source.identity[..16]
}

#[cfg(test)]
#[path = "certificates/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "certificates/no_release_tests.rs"]
mod no_release_tests;

#[cfg(test)]
#[path = "certificates/head_roster_tests.rs"]
mod head_roster_tests;
