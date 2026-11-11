//! Fold acknowledged attempts and observations; custody is derived membership.
use super::*;

/// The latest recorded attempt for one co-committed external effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueryDispatchCustody {
    attempt: DispatchAttemptIdentity,
    observation: Option<QueryDispatchRecordedObservation>,
}
impl QueryDispatchCustody {
    pub const fn attempt(&self) -> DispatchAttemptIdentity {
        self.attempt
    }
    /// `None` only while the original attempt's contact is still executing.
    pub const fn observation(&self) -> Option<QueryDispatchRecordedObservation> {
        self.observation
    }
}

/// Memory is O(recorded effects + controlled sources): latest custody and monotone source facts. Folding
/// is O(frames). Reclamation of settled history belongs to retention work.
#[derive(Clone, Debug)]
pub(super) struct QueryDispatchLedger {
    home: HomeIdentity,
    position: u64,
    digest: [u8; 32],
    payload_digest: [u8; 32],
    publication_generation: u64,
    effects: BTreeMap<CanonicalDigestId, QueryDispatchCustody>,
    source_controls: BTreeMap<CanonicalDigestId, InboundSourceControl>,
    awaiting_observation: BTreeMap<DispatchAttemptIdentity, CanonicalDigestId>,
    pub(super) inbound: super::inbound::QueryInboundMessageLedger,
}
impl QueryDispatchLedger {
    pub(super) fn empty(home: HomeIdentity) -> Self {
        Self {
            home,
            position: 0,
            digest: [0; 32],
            payload_digest: [0; 32],
            publication_generation: 0,
            effects: BTreeMap::new(),
            source_controls: BTreeMap::new(),
            awaiting_observation: BTreeMap::new(),
            inbound: Default::default(),
        }
    }
    pub(super) const fn position(&self) -> u64 {
        self.position
    }
    pub(super) const fn digest(&self) -> [u8; 32] {
        self.digest
    }
    pub(super) const fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
    pub(super) const fn publication_generation(&self) -> u64 {
        self.publication_generation
    }
    pub(super) const fn home(&self) -> HomeIdentity {
        self.home
    }
    pub(super) fn custody(&self, correlation: &CanonicalDigestId) -> Option<QueryDispatchCustody> {
        self.effects.get(correlation).copied()
    }
    pub(super) fn source_control(
        &self,
        source: &CanonicalDigestId,
    ) -> Option<InboundSourceControl> {
        self.source_controls.get(source).copied()
    }
    /// The earliest attempt recorded before contact whose one observation
    /// is not yet durable, with the effect it contacts.
    pub(super) fn first_unobserved(&self) -> Option<QueryDispatchJournalDenial> {
        self.awaiting_observation
            .iter()
            .next()
            .map(
                |(attempt, effect)| QueryDispatchJournalDenial::UnobservedAttempt {
                    attempt: *attempt,
                    effect: *effect,
                },
            )
    }
    pub(super) fn awaits_observation(&self, attempt: &DispatchAttemptIdentity) -> bool {
        self.awaiting_observation.contains_key(attempt)
    }
    /// Preflight returns descriptive bytes only. It changes no admitted state.
    pub(super) fn prepare(
        &self,
        intent: QueryDispatchRecordIntent,
        generations: &mut QueryDispatchPublicationGenerations,
    ) -> Result<QueryDispatchFrame, QueryDispatchJournalDenial> {
        use QueryDispatchJournalDenial as Denial;
        let position = self
            .position
            .checked_add(1)
            .ok_or(Denial::IdentityExhausted)?;
        let record = match intent {
            QueryDispatchRecordIntent::Inbound(value) => QueryDispatchRecord::Inbound(value),
            QueryDispatchRecordIntent::SourceControl { source, posture } => {
                QueryDispatchRecord::SourceControl { source, posture }
            }
            QueryDispatchRecordIntent::Attempt { correlation } => {
                QueryDispatchRecord::Attempt(QueryDispatchAttempt {
                    identity: DispatchAttemptIdentity::from_recorded(
                        self.home,
                        std::num::NonZeroU64::new(position).expect("checked positive successor"),
                    ),
                    correlation,
                })
            }
            QueryDispatchRecordIntent::Observation {
                attempt,
                observation,
            } => QueryDispatchRecord::Observation {
                attempt,
                observation,
            },
        };
        self.validate(&record, position)?;
        let generation = generations.allocate(self)?;
        Ok(super::codec::encode(
            self.home,
            position,
            self.digest,
            generation,
            record,
        ))
    }
    /// Readmission never treats matching content as live publication authority.
    pub(super) fn readmit(
        &mut self,
        frame: QueryDispatchFrame,
    ) -> Result<(), QueryDispatchJournalDenial> {
        use QueryDispatchJournalDenial as Denial;
        if frame.home != self.home {
            return Err(Denial::ForeignHome);
        }
        if self.position.checked_add(1) != Some(frame.position) || self.digest != frame.predecessor
        {
            return Err(Denial::PrefixMismatch);
        }
        if frame.generation.get() <= self.publication_generation {
            return Err(Denial::PublicationGenerationMismatch);
        }
        self.validate(&frame.record, frame.position)?;
        let payload_digest = frame.payload_digest();
        match frame.record {
            QueryDispatchRecord::Inbound(value) => self.inbound.fold(value, frame.position),
            QueryDispatchRecord::SourceControl { source, posture } => {
                self.source_controls.insert(source, posture);
            }
            QueryDispatchRecord::Attempt(attempt) => {
                self.effects.insert(
                    attempt.correlation,
                    QueryDispatchCustody {
                        attempt: attempt.identity,
                        observation: None,
                    },
                );
                self.awaiting_observation
                    .insert(attempt.identity, attempt.correlation);
            }
            QueryDispatchRecord::Observation {
                attempt,
                observation,
            } => {
                let correlation = self
                    .awaiting_observation
                    .remove(&attempt)
                    .expect("validated attempt awaiting its observation");
                self.effects
                    .get_mut(&correlation)
                    .expect("an awaiting attempt is its effect's latest custody")
                    .observation = Some(observation);
            }
        }
        self.position = frame.position;
        self.digest = frame.digest;
        self.payload_digest = payload_digest;
        self.publication_generation = frame.generation.get();
        Ok(())
    }
    fn validate(
        &self,
        record: &QueryDispatchRecord,
        position: u64,
    ) -> Result<(), QueryDispatchJournalDenial> {
        use QueryDispatchJournalDenial as Denial;
        match record {
            QueryDispatchRecord::Inbound(value) => self.inbound.validate(value),
            QueryDispatchRecord::SourceControl { source, posture } => {
                if self
                    .source_control(source)
                    .is_some_and(|old| old > *posture)
                {
                    Err(Denial::MalformedFrame)
                } else {
                    Ok(())
                }
            }
            QueryDispatchRecord::Attempt(attempt) => {
                if attempt.identity.home() != self.home
                    || attempt.identity.attempt_position().get() != position
                {
                    return Err(Denial::AttemptIdentityMismatch);
                }
                match self
                    .effects
                    .get(&attempt.correlation)
                    .map(|custody| custody.observation)
                {
                    None => Ok(()),
                    Some(None) => Err(Denial::AttemptAwaitingObservation),
                    Some(Some(QueryDispatchRecordedObservation::Completed)) => {
                        Err(Denial::EffectAlreadyCompleted)
                    }
                    Some(Some(
                        QueryDispatchRecordedObservation::Acknowledged
                        | QueryDispatchRecordedObservation::ProvenNoEffect(_)
                        | QueryDispatchRecordedObservation::Unresolved(_),
                    )) => Ok(()),
                }
            }
            QueryDispatchRecord::Observation { attempt, .. } => {
                if attempt.home() != self.home {
                    return Err(Denial::ForeignHome);
                }
                if self.awaiting_observation.contains_key(attempt) {
                    return Ok(());
                }
                if self
                    .effects
                    .values()
                    .any(|custody| custody.attempt == *attempt)
                {
                    return Err(Denial::AttemptAlreadyObserved);
                }
                Err(Denial::UnknownAttempt)
            }
        }
    }
}
