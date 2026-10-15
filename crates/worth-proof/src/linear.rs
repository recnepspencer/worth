//! Resource values represented once and consumed exactly once.
//!
//! Milestone 9.16 Phase 8's recovery handle is a general pattern wearing a
//! domain name: represented under an authority, reaching **exactly one** terminal
//! state, never usable afterwards. Store, UI, and Signal all have this shape,
//! and each hand-rolled it with a `live: bool` guarded at run time.
//!
//! The law is expressible without any of that. [`LinearResource::terminate`]
//! takes `self`, so a second transition is not a check that fails — it is a
//! program that does not compile.
//!
//! **What is deliberately absent.** No registry, no enumeration of live
//! resources, no `Drop` leak detection, no `assert_no_live()`. Those need a
//! live table and a clock, which this crate does not have and does not want;
//! they belong to the runtime that owns the resources. What that runtime
//! should not have to re-derive is the law itself.
//!
//! **Honest limit.** Rust has no linear types: nothing forces a caller to
//! terminate rather than drop. This module makes the *second* termination
//! unrepresentable (rung 1) and marks the resource `#[must_use]` so an ignored
//! one is a warning. Detecting a resource that was dropped without terminating
//! is a `Drop` impl, and therefore the owner's job.

use std::fmt::{self, Debug};
use std::marker::PhantomData;

use crate::proof::{AuthorityMarker, AuthorityWitness};

/// The closed set of ways a resource can end.
///
/// The owner defines the type — typically an enum, one variant per way the
/// resource can finish. `label` is required rather than derived so that adding
/// a variant produces a non-exhaustive-match error at the one place that has
/// to know about it, instead of a silently wrong string.
pub trait TerminalState: 'static {
    fn label(&self) -> &'static str;
}

/// A resource value that exists once, ends once, and cannot be used after.
///
/// `Id` is whatever the owner needs to identify it — an opaque counter value,
/// a name, a handle. This crate does not mint it: a process-unique identity
/// needs a counter, which belongs to the runtime.
#[must_use = "a LinearResource must reach a terminal state; dropping it silently is the leak this type exists to make visible"]
pub struct LinearResource<Id, Terminal, Authority>
where
    Terminal: TerminalState,
    Authority: AuthorityMarker,
{
    id: Id,
    terminal: PhantomData<Terminal>,
    authority: PhantomData<Authority>,
}

impl<Id, Terminal, Authority> LinearResource<Id, Terminal, Authority>
where
    Terminal: TerminalState,
    Authority: AuthorityMarker,
{
    /// Represent an owner-issued resource under an authority.
    ///
    /// The witness is the seal: only code that can produce an
    /// `AuthorityWitness<Authority>` — that is, the module that declared the
    /// marker — can bring one of these into existence. A caller cannot mint a
    /// resource and hand it in as though the runtime had issued it. Identity
    /// uniqueness remains the owner runtime's job: this type has no counter or
    /// live table.
    pub fn mint(id: Id, _authority: &AuthorityWitness<Authority>) -> Self {
        Self {
            id,
            terminal: PhantomData,
            authority: PhantomData,
        }
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    /// End the resource, naming which terminal it reached.
    ///
    /// Takes `self`. There is no path that terminates twice, so there is no
    /// `already_terminated` error to define, test, or forget to check.
    pub fn terminate(self, terminal: Terminal) -> TerminalReceipt<Id, Terminal, Authority> {
        TerminalReceipt {
            id: self.id,
            terminal,
            authority: PhantomData,
        }
    }

    /// Attempt to end the resource, handing it back if the attempt is refused.
    ///
    /// Without this, a fallible termination forces the caller to choose
    /// between consuming the resource before knowing the outcome and keeping a
    /// second reachable copy. Returning it inside `Err` keeps exactly one
    /// live resource on every path — the retry-safe shape.
    #[allow(clippy::result_large_err)]
    pub fn try_terminate<E>(
        self,
        decide: impl FnOnce(&Id) -> Result<Terminal, E>,
    ) -> Result<TerminalReceipt<Id, Terminal, Authority>, (Self, E)> {
        match decide(&self.id) {
            Ok(terminal) => Ok(self.terminate(terminal)),
            Err(error) => Err((self, error)),
        }
    }
}

/// Evidence that a resource reached one specific terminal state.
///
/// Produced only by [`LinearResource::terminate`], so a function that requires
/// a receipt cannot be reached by a caller who merely intended to finish.
#[must_use]
pub struct TerminalReceipt<Id, Terminal, Authority>
where
    Terminal: TerminalState,
    Authority: AuthorityMarker,
{
    id: Id,
    terminal: Terminal,
    authority: PhantomData<Authority>,
}

impl<Id, Terminal, Authority> TerminalReceipt<Id, Terminal, Authority>
where
    Terminal: TerminalState,
    Authority: AuthorityMarker,
{
    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn terminal(&self) -> &Terminal {
        &self.terminal
    }

    pub fn label(&self) -> &'static str {
        self.terminal.label()
    }

    pub fn into_parts(self) -> (Id, Terminal) {
        (self.id, self.terminal)
    }
}

// `Debug` and equality are bounded on the payload only. A derive would put
// them on `Authority` as well, forcing every authority marker to carry derives
// it has no use for. Neither type is `Clone`: a resource that exists once
// cannot be duplicated, and a receipt is evidence of one specific ending.
impl<Id, Terminal, Authority> Debug for LinearResource<Id, Terminal, Authority>
where
    Id: Debug,
    Terminal: TerminalState,
    Authority: AuthorityMarker,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LinearResource")
            .field("id", &self.id)
            .finish()
    }
}

impl<Id, Terminal, Authority> Debug for TerminalReceipt<Id, Terminal, Authority>
where
    Id: Debug,
    Terminal: TerminalState + Debug,
    Authority: AuthorityMarker,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TerminalReceipt")
            .field("id", &self.id)
            .field("terminal", &self.terminal)
            .finish()
    }
}

impl<Id, Terminal, Authority> PartialEq for TerminalReceipt<Id, Terminal, Authority>
where
    Id: PartialEq,
    Terminal: TerminalState + PartialEq,
    Authority: AuthorityMarker,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.terminal == other.terminal
    }
}

impl<Id, Terminal, Authority> Eq for TerminalReceipt<Id, Terminal, Authority>
where
    Id: Eq,
    Terminal: TerminalState + Eq,
    Authority: AuthorityMarker,
{
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::{LinearResource, TerminalReceipt, TerminalState};
    crate::authority_marker!(pub RecoveryAuthority);

    #[derive(Debug, PartialEq, Eq)]
    enum RecoveryEnd {
        Completed,
        Cancelled,
        Expired,
    }

    impl TerminalState for RecoveryEnd {
        fn label(&self) -> &'static str {
            match self {
                Self::Completed => "completed",
                Self::Cancelled => "cancelled",
                Self::Expired => "expired",
            }
        }
    }

    type Recovery = LinearResource<u64, RecoveryEnd, RecoveryAuthority>;

    fn mint(id: u64) -> Recovery {
        LinearResource::mint(id, &RecoveryAuthority::witness())
    }

    const _: () = assert!(size_of::<Recovery>() == size_of::<u64>());
    const _: () = assert!(
        size_of::<TerminalReceipt<u64, RecoveryEnd, RecoveryAuthority>>()
            == size_of::<(u64, RecoveryEnd)>()
    );

    #[test]
    fn terminating_names_the_terminal_it_reached() {
        let receipt = mint(7).terminate(RecoveryEnd::Cancelled);

        assert_eq!(receipt.id(), &7);
        assert_eq!(receipt.terminal(), &RecoveryEnd::Cancelled);
        assert_eq!(receipt.label(), "cancelled");
    }

    #[test]
    fn a_refused_termination_returns_the_resource_rather_than_losing_it() {
        let resource = mint(11);

        let Err((returned, reason)) = resource.try_terminate(|id| {
            assert_eq!(id, &11);
            Err::<RecoveryEnd, &str>("not yet quiesced")
        }) else {
            panic!("termination should have been refused");
        };

        assert_eq!(reason, "not yet quiesced");
        // Exactly one resource is live on this path, and it is still usable.
        let receipt = returned.terminate(RecoveryEnd::Completed);
        assert_eq!(receipt.label(), "completed");
    }

    #[test]
    fn an_accepted_termination_consumes_the_resource() {
        let receipt = mint(13)
            .try_terminate(|_| Ok::<_, ()>(RecoveryEnd::Expired))
            .expect("termination was accepted");

        let (id, terminal) = receipt.into_parts();
        assert_eq!(id, 13);
        assert_eq!(terminal, RecoveryEnd::Expired);
        // A second `terminate` on the same resource does not typecheck; see
        // `tests/ui/contracts/compile_fail/linear_resource_cannot_terminate_twice.rs`.
    }
}
