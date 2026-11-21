//! Cold release of process-local custody whose Home incarnation ended.
//!
//! The only proof that a row's process-local resources are gone is the Home
//! incarnation fence (D7): every row the cold fold reads before service was
//! retained by an earlier incarnation that ended and is fenced out. The fold
//! that read them mints the one witness of that fact; a row retained after the
//! fold, a fresh empty receiver or a release from another fold proves nothing.
use super::{QueryObligationDenial as Denial, QueryObligationFrame, QueryObligationLedger};
use worth_foundational::facade::{
    HomeIdentity, ManagedCleanupObligationIdentity, ManagedRunObligationIdentity,
    OwnedAsyncObligationIdentity,
};
pub(super) mod codec;
mod fold;

/// The R7 terminal a cold release records. It has no `Completed` arm: losing
/// a process never completes work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryColdReleaseTerminal {
    /// Admitted or running work, or a resource request without a recorded
    /// result: R7's abandonment route, which ends as `Failed`.
    Abandoned,
    /// A recorded terminal, cleanup failure or result whose release was
    /// outstanding. The recorded terminal and failure stand unchanged; only
    /// the process-local resources ended with the incarnation.
    ReleaseOutstanding,
}

/// One pending row of a cold-releasable family.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum QueryColdReleaseSubject {
    ManagedRun(ManagedRunObligationIdentity),
    ManagedCleanup(ManagedCleanupObligationIdentity),
    /// An application resource. Its request is an opted-in process-local
    /// owned-async request; a mandatory owned-async request is never one.
    Resource(OwnedAsyncObligationIdentity),
}
impl QueryColdReleaseSubject {
    pub(super) fn home(self) -> HomeIdentity {
        match self {
            Self::ManagedRun(identity) => identity.home(),
            Self::ManagedCleanup(identity) => identity.home(),
            Self::Resource(identity) => identity.home(),
        }
    }
    /// The journal position whose frame retained this row.
    pub(super) fn acquire_position(self) -> u64 {
        match self {
            Self::ManagedRun(identity) => identity.acquire_position().get(),
            Self::ManagedCleanup(identity) => identity.acquire_position().get(),
            Self::Resource(identity) => identity.acquire_position().get(),
        }
    }
}

/// DATA: the row named by `subject`, retained at or before `fence`, ended with
/// its incarnation. Built only from a [`QueryEndedIncarnation`] or decoded from
/// its own durable frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryColdRelease {
    subject: QueryColdReleaseSubject,
    fence: u64,
    terminal: QueryColdReleaseTerminal,
}
impl QueryColdRelease {
    pub const fn subject(&self) -> QueryColdReleaseSubject {
        self.subject
    }
    /// The last journal position the ended incarnation retained.
    pub const fn fence(&self) -> u64 {
        self.fence
    }
    pub const fn terminal(&self) -> QueryColdReleaseTerminal {
        self.terminal
    }
}

/// The before-service fold of a reopened Home's obligation journal.
#[derive(Debug)]
pub struct QueryColdObligationFold {
    ledger: QueryObligationLedger,
}
impl QueryColdObligationFold {
    pub fn new(home: HomeIdentity) -> Self {
        Self {
            ledger: QueryObligationLedger::empty(home),
        }
    }
    pub const fn ledger(&self) -> &QueryObligationLedger {
        &self.ledger
    }
    pub fn readmit(&mut self, frame: QueryObligationFrame) -> Result<(), Denial> {
        self.ledger.readmit(frame)
    }
    /// Ends the fold. Every row folded so far belongs to the ended
    /// incarnation; the returned ledger accepts cold releases only from the
    /// returned witness.
    pub fn finish(mut self) -> (QueryObligationLedger, QueryEndedIncarnation) {
        let fence = self.ledger.position();
        self.ledger.cold_fence = Some(fence);
        let ended = QueryEndedIncarnation {
            home: self.ledger.home(),
            fence,
        };
        (self.ledger, ended)
    }
}

/// Witness that the incarnation which retained every row up to `fence` has
/// ended and is fenced. Minted only by [`QueryColdObligationFold::finish`].
#[derive(Debug)]
#[must_use = "an ended incarnation's pending rows are released through its witness"]
pub struct QueryEndedIncarnation {
    home: HomeIdentity,
    fence: u64,
}
impl QueryEndedIncarnation {
    pub const fn fence(&self) -> u64 {
        self.fence
    }
    /// The cold release of one row, refused unless it is a pending row of a
    /// cold-releasable family that the ended incarnation retained.
    pub fn cold_release(
        &self,
        ledger: &QueryObligationLedger,
        subject: QueryColdReleaseSubject,
    ) -> Result<QueryColdRelease, Denial> {
        if ledger.home() != self.home {
            return Err(Denial::ForeignHome);
        }
        if ledger.cold_fence != Some(self.fence) {
            return Err(Denial::IncarnationNotEnded);
        }
        let release = QueryColdRelease {
            subject,
            fence: self.fence,
            terminal: ledger.cold_release_terminal(subject)?,
        };
        let position = ledger
            .position()
            .checked_add(1)
            .ok_or(Denial::IdentityExhausted)?;
        ledger.validate_cold_release(&release, position)?;
        Ok(release)
    }
    /// Consumes the witness: one cold release for every pending row the ended
    /// incarnation retained. Rows retained after the fence, released rows,
    /// cleanup rows with inconsistent evidence and owner-held rows outside
    /// these families are not candidates and stay as they are.
    pub fn into_cold_releases(
        self,
        ledger: &QueryObligationLedger,
    ) -> Result<Vec<QueryColdRelease>, Denial> {
        ledger
            .cold_release_candidates()
            .into_iter()
            .filter(|subject| subject.acquire_position() <= self.fence)
            .map(|subject| self.cold_release(ledger, subject))
            .collect()
    }
}
