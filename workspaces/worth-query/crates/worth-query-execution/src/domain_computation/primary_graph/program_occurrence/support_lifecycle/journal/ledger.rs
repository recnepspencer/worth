//! Fold acknowledged identities and statuses; counts are derived membership.
use super::*;
#[derive(Clone, Debug)]
pub struct QueryProgramSupportLedger {
    home: HomeIdentity,
    position: u64,
    digest: [u8; 32],
    payload_digest: [u8; 32],
    publication_generation: u64,
    statuses: BTreeMap<CanonicalDigestId, ProgramSupportStatus>,
    obligations: BTreeMap<ProgramSupportObligationIdentity, QueryProgramSupportObligation>,
}
impl QueryProgramSupportLedger {
    pub fn empty(home: HomeIdentity) -> Self {
        Self {
            home,
            position: 0,
            digest: [0; 32],
            payload_digest: [0; 32],
            publication_generation: 0,
            statuses: BTreeMap::new(),
            obligations: BTreeMap::new(),
        }
    }
    pub const fn position(&self) -> u64 {
        self.position
    }
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
    pub const fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
    pub const fn publication_generation(&self) -> u64 {
        self.publication_generation
    }
    pub const fn home(&self) -> HomeIdentity {
        self.home
    }
    pub fn status(&self, revision: &CanonicalDigestId) -> Option<ProgramSupportStatus> {
        self.statuses.get(revision).copied()
    }
    pub fn statuses(&self) -> impl Iterator<Item = (&CanonicalDigestId, &ProgramSupportStatus)> {
        self.statuses.iter()
    }
    pub fn obligations(&self) -> impl Iterator<Item = &QueryProgramSupportObligation> {
        self.obligations.values()
    }
    pub fn counts(&self, revision: &CanonicalDigestId) -> (usize, usize) {
        self.obligations
            .values()
            .filter(|row| row.source == *revision || row.target == *revision)
            .fold((0, 0), |(interpretations, custody), row| match row.kind {
                ProgramSupportObligationKind::OriginalInterpretation => {
                    (interpretations + 1, custody)
                }
                ProgramSupportObligationKind::BranchAdoption => (interpretations, custody + 1),
            })
    }
    /// Descriptive membership of a prepared change, never publication authority.
    pub fn record_mentions_revision(
        &self,
        record: &QueryProgramSupportRecord,
        revision: &CanonicalDigestId,
    ) -> bool {
        match record {
            QueryProgramSupportRecord::Install { revision: affected }
            | QueryProgramSupportRecord::Status {
                revision: affected, ..
            } => affected == revision,
            QueryProgramSupportRecord::Acquire(row) => {
                row.source == *revision || row.target == *revision
            }
            QueryProgramSupportRecord::Release { identity } => self
                .obligations
                .get(identity)
                .is_some_and(|row| row.source == *revision || row.target == *revision),
        }
    }
    /// Preflight returns descriptive bytes only. It changes no admitted state.
    pub fn prepare(
        &self,
        record: QueryProgramSupportRecord,
        generations: &mut QueryProgramSupportPublicationGenerations,
    ) -> Result<QueryProgramSupportFrame, QueryProgramSupportDenial> {
        self.validate(
            &record,
            self.position
                .checked_add(1)
                .ok_or(QueryProgramSupportDenial::IdentityExhausted)?,
        )?;
        let generation = generations.allocate(self)?;
        Ok(super::codec::encode(
            self.home,
            self.position + 1,
            self.digest,
            generation,
            record,
        ))
    }
    /// Readmission never treats matching content as live publication authority.
    pub fn readmit(
        &mut self,
        frame: QueryProgramSupportFrame,
    ) -> Result<(), QueryProgramSupportDenial> {
        use QueryProgramSupportDenial as Denial;
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
            QueryProgramSupportRecord::Install { revision } => {
                self.statuses.insert(revision, ProgramSupportStatus::Active);
            }
            QueryProgramSupportRecord::Status {
                revision, after, ..
            } => {
                self.statuses.insert(revision, after);
            }
            QueryProgramSupportRecord::Acquire(row) => {
                self.obligations.insert(row.identity, row);
            }
            QueryProgramSupportRecord::Release { identity } => {
                self.obligations.remove(&identity);
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
        record: &QueryProgramSupportRecord,
        position: u64,
    ) -> Result<(), QueryProgramSupportDenial> {
        use QueryProgramSupportDenial as Denial;
        match record {
            QueryProgramSupportRecord::Install { revision } => {
                if self.statuses.contains_key(revision) {
                    return Err(Denial::DuplicateProgram);
                }
            }
            QueryProgramSupportRecord::Status {
                revision,
                before,
                after,
            } => {
                if self.status(revision) != Some(*before) {
                    return Err(Denial::StatusMismatch);
                }
                match (before, after) {
                    (ProgramSupportStatus::Active, ProgramSupportStatus::Retiring)
                    | (ProgramSupportStatus::Retiring, ProgramSupportStatus::Active) => {}
                    (ProgramSupportStatus::Retiring, ProgramSupportStatus::Retired) => {
                        if self.counts(revision) != (0, 0) {
                            return Err(Denial::SupportUnavailable);
                        }
                    }
                    (
                        ProgramSupportStatus::Active,
                        ProgramSupportStatus::Active | ProgramSupportStatus::Retired,
                    )
                    | (ProgramSupportStatus::Retiring, ProgramSupportStatus::Retiring)
                    | (
                        ProgramSupportStatus::Retired,
                        ProgramSupportStatus::Active
                        | ProgramSupportStatus::Retiring
                        | ProgramSupportStatus::Retired,
                    ) => return Err(Denial::IllegalStatusTransition),
                }
            }
            QueryProgramSupportRecord::Acquire(row) => {
                if row.identity.home() != self.home
                    || row.identity.acquire_position().get() != position
                {
                    return Err(Denial::ObligationIdentityMismatch);
                }
                if self.obligations.contains_key(&row.identity) {
                    return Err(Denial::DuplicateObligation);
                }
                if row.original.owner().home() != self.home {
                    return Err(Denial::ForeignHome);
                }
                if let Some(existing) = self
                    .obligations
                    .values()
                    .find(|known| known.kind == row.kind && known.original == row.original)
                {
                    return Err(
                        if existing.source == row.source && existing.target == row.target {
                            Denial::DuplicateObligation
                        } else {
                            Denial::ObligationSubjectMismatch
                        },
                    );
                }
                match row.kind {
                    ProgramSupportObligationKind::OriginalInterpretation => {
                        if row.source != row.target {
                            return Err(Denial::ObligationSubjectMismatch);
                        }
                    }
                    ProgramSupportObligationKind::BranchAdoption => {}
                }
                for revision in [&row.source, &row.target] {
                    if self.status(revision) != Some(ProgramSupportStatus::Active) {
                        return Err(Denial::SupportUnavailable);
                    }
                }
            }
            QueryProgramSupportRecord::Release { identity } => {
                if identity.home() != self.home {
                    return Err(Denial::ForeignHome);
                }
                if !self.obligations.contains_key(identity) {
                    return Err(Denial::UnknownObligation);
                }
            }
        }
        Ok(())
    }
}
