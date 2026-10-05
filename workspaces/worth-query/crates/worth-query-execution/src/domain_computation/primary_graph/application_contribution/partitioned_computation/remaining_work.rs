//! What is left of a partitioned computation's declared work.
//!
//! The request's admission meters (`ReservedExternalWork`) do not fit: they
//! charge the request's invalidation-edit meter, which the handler's reader
//! does not reach, and they refund to it. The declared work is the
//! computation's own ceiling, so it is spent here, by one checked spend.

use worth_query_declaration::facade::application_operation::CanonicalEncodingCharge;

use super::super::WorthQueryManagedComputationResourceDenial;

/// The declared work left and the work spent from it. Work is only ever
/// spent, and only by `spend`, so the two always sum to the declared work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RemainingWork {
    remaining: u64,
    spent: u64,
}

impl RemainingWork {
    pub(super) const fn declared(work: u64) -> Self {
        Self {
            remaining: work,
            spent: 0,
        }
    }

    /// Spends `work`, or refuses it and spends nothing when it does not fit.
    /// `None` is a count that overflowed, which fits no ceiling.
    pub(super) fn spend(
        &mut self,
        work: Option<u64>,
    ) -> Result<(), WorthQueryManagedComputationResourceDenial> {
        let refused = WorthQueryManagedComputationResourceDenial::WorkExhausted;
        let work = work.ok_or(refused)?;
        *self = Self {
            remaining: self.remaining.checked_sub(work).ok_or(refused)?,
            spent: self.spent.checked_add(work).ok_or(refused)?,
        };
        Ok(())
    }

    /// The work a ceiling may still admit.
    pub(super) const fn remaining(self) -> u64 {
        self.remaining
    }

    /// All the work spent so far.
    pub(super) const fn spent(self) -> u64 {
        self.spent
    }

    /// The work spent since `earlier`, or `None` when this is not a later
    /// state of it.
    pub(super) const fn spent_since(self, earlier: Self) -> Option<u64> {
        self.spent.checked_sub(earlier.spent)
    }
}

/// One canonical encoding's admission: its work spends the declared work and
/// every scratch growth adds to the encoding's running total, which may not
/// pass the computation's per-partition bytes. The meter owns that total and
/// starts it with the encoding, so no growth is ever checked alone.
pub(super) struct EncodingMeter<'work> {
    remaining: &'work mut RemainingWork,
    declared_bytes: u64,
    scratch: u64,
}

impl<'work> EncodingMeter<'work> {
    pub(super) fn new(remaining: &'work mut RemainingWork, declared_bytes: u64) -> Self {
        Self {
            remaining,
            declared_bytes,
            scratch: 0,
        }
    }

    pub(super) fn admit(
        &mut self,
        charge: CanonicalEncodingCharge,
    ) -> Result<(), WorthQueryManagedComputationResourceDenial> {
        match charge {
            CanonicalEncodingCharge::Work(work) => self.remaining.spend(Some(work)),
            CanonicalEncodingCharge::Scratch(bytes) => {
                self.scratch = self
                    .scratch
                    .checked_add(bytes)
                    .filter(|held| *held <= self.declared_bytes)
                    .ok_or(WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted)?;
                Ok(())
            }
        }
    }
}
