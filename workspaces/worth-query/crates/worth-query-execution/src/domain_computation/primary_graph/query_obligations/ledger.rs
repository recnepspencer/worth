//! Fold acknowledged identities and statuses; counts are derived membership.
use super::*;
#[derive(Clone, Debug)]
pub struct QueryObligationLedger {
    home: HomeIdentity,
    position: u64,
    digest: [u8; 32],
    payload_digest: [u8; 32],
    publication_generation: u64,
    pub(super) cleanup: managed_cleanup::fold::ManagedCleanupFold,
    pub(super) adoption: adoption::fold::AdoptionFold,
    pub(super) output: output_custody::fold::OutputCustodyFold,
    statuses: BTreeMap<CanonicalDigestId, ProgramSupportStatus>,
    obligations: BTreeMap<ProgramSupportObligationIdentity, QueryProgramSupportObligation>,
}
impl QueryObligationLedger {
    pub fn empty(home: HomeIdentity) -> Self {
        Self {
            home,
            position: 0,
            digest: [0; 32],
            payload_digest: [0; 32],
            publication_generation: 0,
            cleanup: managed_cleanup::fold::ManagedCleanupFold::default(),
            adoption: adoption::fold::AdoptionFold::default(),
            output: output_custody::fold::OutputCustodyFold::default(),
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
        let (interpretations, custody) = self
            .obligations
            .values()
            .filter(|row| row.source == *revision || row.target == *revision)
            .fold((0, 0), |(interpretations, custody), row| match row.kind {
                ProgramSupportObligationKind::OriginalInterpretation => {
                    (interpretations + 1, custody)
                }
                ProgramSupportObligationKind::BranchAdoption => (interpretations, custody + 1),
            });
        (
            interpretations,
            custody + self.output.count(revision) + self.adoption.count(revision),
        )
    }
    /// Descriptive membership of a prepared change, never publication authority.
    pub fn record_mentions_revision(
        &self,
        record: &QueryObligationRecord,
        revision: &CanonicalDigestId,
    ) -> bool {
        match record {
            QueryObligationRecord::Adoption(record) => {
                return self.adoption.mentions(record, revision)
            }
            QueryObligationRecord::AcceptedOutput(_) => return false,
            QueryObligationRecord::OutputSource(QueryOutputSourceRecord::Retain(row)) => {
                return row.revision == *revision
            }
            QueryObligationRecord::OutputSource(
                QueryOutputSourceRecord::Discharge { commit }
                | QueryOutputSourceRecord::Bind { commit, .. },
            ) => {
                return self
                    .output
                    .sources
                    .get(commit)
                    .is_some_and(|row| row.revision == *revision)
            }
            QueryObligationRecord::OutputDemand(QueryOutputDemandRecord::Retain(row)) => {
                return row.revision == *revision
            }
            QueryObligationRecord::OutputDemand(
                QueryOutputDemandRecord::Discharge { acquire_position }
                | QueryOutputDemandRecord::DomainDenied {
                    acquire_position, ..
                }
                | QueryOutputDemandRecord::Links {
                    acquire_position, ..
                },
            ) => {
                return self
                    .output
                    .demands
                    .get(acquire_position)
                    .is_some_and(|row| row.revision == *revision)
            }
            QueryObligationRecord::ProgramSupport(_) | QueryObligationRecord::ManagedCleanup(_) => {
            }
        }
        let QueryObligationRecord::ProgramSupport(record) = record else {
            return false;
        };
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
        record: impl Into<QueryObligationRecord>,
        generations: &mut QueryObligationPublicationGenerations,
    ) -> Result<QueryObligationFrame, QueryObligationDenial> {
        self.prepare_record(record.into(), generations)
    }
    fn prepare_record(
        &self,
        record: QueryObligationRecord,
        generations: &mut QueryObligationPublicationGenerations,
    ) -> Result<QueryObligationFrame, QueryObligationDenial> {
        self.validate(
            &record,
            self.position
                .checked_add(1)
                .ok_or(QueryObligationDenial::IdentityExhausted)?,
        )?;
        let generation = generations.allocate(self)?;
        let frame = super::codec::encode(
            self.home,
            self.position + 1,
            self.digest,
            generation,
            record,
        );
        if frame.bytes().len() as u64 > QUERY_OBLIGATION_FRAME_MAX_BYTES {
            return Err(QueryObligationDenial::MalformedFrame);
        }
        Ok(frame)
    }
    /// Readmission never treats matching content as live publication authority.
    pub fn readmit(&mut self, frame: QueryObligationFrame) -> Result<(), QueryObligationDenial> {
        use QueryObligationDenial as Denial;
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
            QueryObligationRecord::Adoption(record) => self.adoption.apply(record),
            QueryObligationRecord::AcceptedOutput(row) => {
                self.output.accepted.insert(
                    (
                        row.branch.clone(),
                        row.producer.clone(),
                        row.scope,
                        row.partition,
                    ),
                    row,
                );
            }
            QueryObligationRecord::ProgramSupport(record) => self.apply_support(record),
            QueryObligationRecord::ManagedCleanup(record) => self.cleanup.apply(record),
            QueryObligationRecord::OutputSource(record) => self.output.apply_source(record),
            QueryObligationRecord::OutputDemand(record) => self.output.apply_demand(record),
        }
        self.position = frame.position;
        self.digest = frame.digest;
        self.payload_digest = payload_digest;
        self.publication_generation = frame.generation.get();
        Ok(())
    }
    fn validate(
        &self,
        record: &QueryObligationRecord,
        position: u64,
    ) -> Result<(), QueryObligationDenial> {
        match record {
            QueryObligationRecord::Adoption(record) => {
                self.adoption.validate(record, self.home)?;
                if let QueryAdoptionRecord::RetainBranch(row) = record {
                    for revision in [&row.source, &row.target] {
                        if self.status(revision) != Some(ProgramSupportStatus::Active) {
                            return Err(QueryObligationDenial::SupportUnavailable);
                        }
                    }
                }
                if let QueryAdoptionRecord::RetainStopped(row) = record {
                    for revision in std::iter::once(&row.target)
                        .chain(row.remaining.iter().map(|(_, _, source)| source))
                    {
                        if self.status(revision) != Some(ProgramSupportStatus::Active) {
                            return Err(QueryObligationDenial::SupportUnavailable);
                        }
                    }
                }
                Ok(())
            }
            QueryObligationRecord::AcceptedOutput(row) => {
                if row.commit.owner().home() != self.home {
                    return Err(QueryObligationDenial::ForeignHome);
                }
                if row.branch.is_empty() || row.producer.is_empty() || row.payload.is_empty() {
                    return Err(QueryObligationDenial::MalformedFrame);
                }
                Ok(())
            }
            QueryObligationRecord::ProgramSupport(record) => {
                self.validate_support(record, position)
            }
            QueryObligationRecord::ManagedCleanup(record) => {
                self.cleanup.validate(record, self.home, position)
            }
            QueryObligationRecord::OutputSource(record) => {
                self.output.validate_source(record, self.home)?;
                if let QueryOutputSourceRecord::Retain(row) = record
                    && self.status(&row.revision) != Some(ProgramSupportStatus::Active)
                {
                    return Err(QueryObligationDenial::SupportUnavailable);
                }
                Ok(())
            }
            QueryObligationRecord::OutputDemand(record) => {
                self.output.validate_demand(record, self.home, position)?;
                if let QueryOutputDemandRecord::Retain(row) = record {
                    match self.status(&row.revision) {
                        Some(ProgramSupportStatus::Active) => {}
                        Some(ProgramSupportStatus::Retiring)
                            if !row.performed_sources.is_empty() => {}
                        _ => return Err(QueryObligationDenial::SupportUnavailable),
                    }
                }
                Ok(())
            }
        }
    }
}

mod program_support;
