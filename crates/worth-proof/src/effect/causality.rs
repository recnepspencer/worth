use std::marker::PhantomData;

use super::{ActionMarker, Performed};
use crate::proof::AuthorityMarker;

/// `Self` undoes `Original`.
///
/// Declared by the owner of both actions, because only they know that the one
/// reverses the other. Nothing here can check it; what the gate below buys is
/// that the *claim is made once, in the open*, instead of being implied by a
/// caller passing a boolean.
pub trait InverseOf<Original>
where
    Original: ActionMarker,
{
}

/// Evidence that a predecessor action was performed.
///
/// Zero-sized, and mintable only from a [`Performed`] value. A function taking
/// `DerivedFrom<Ingest, A>` cannot be called before ingest has run — the
/// ordering is a type constraint rather than a field the caller fills in.
#[must_use]
pub struct DerivedFrom<Predecessor, Authority>
where
    Predecessor: ActionMarker,
    Authority: AuthorityMarker,
{
    predecessor: PhantomData<Predecessor>,
    authority: PhantomData<Authority>,
}

// Hand-written rather than derived. A derive puts the bounds on the type
// parameters, so comparing two zero-sized proofs would demand
// `Predecessor: PartialEq` — forcing every action marker to carry derives it
// has no other use for. The value holds nothing, so nothing needs comparing.
crate::type_level_traits!(copy DerivedFrom<Predecessor: ActionMarker, Authority: AuthorityMarker>);

/// Evidence that `Original` was reversed.
///
/// Milestone 9.16's `WorthQueryProvedUndo` is this type, hand-rolled — and it
/// shipped as a public five-raw-field constructor, so a caller could assert an
/// undo that never happened (Q8.23). Here the only door is
/// [`prove_inversion`], and it needs a `Performed` value that only the acting
/// module can mint.
#[must_use]
pub struct Inverts<Original, Authority>
where
    Original: ActionMarker,
    Authority: AuthorityMarker,
{
    original: PhantomData<Original>,
    authority: PhantomData<Authority>,
}

// Kept move-only. One proved reversal may be carried into one owner-defined
// progression; owners that admit fan-out must make that policy explicit rather
// than receiving duplication from a zero-sized derive.
crate::type_level_traits!(Inverts<Original: ActionMarker, Authority: AuthorityMarker>);

/// Mint derivation evidence from a performed predecessor.
///
/// Borrows rather than consumes: one completed action can legitimately license
/// several successors, and forcing the caller to keep re-performing it to get
/// a second token would push them back toward a hand-rolled flag.
pub fn prove_derivation<Predecessor, Authority, Outcome>(
    _predecessor: &Performed<Predecessor, Authority, Outcome>,
) -> DerivedFrom<Predecessor, Authority>
where
    Predecessor: ActionMarker,
    Authority: AuthorityMarker,
{
    DerivedFrom {
        predecessor: PhantomData,
        authority: PhantomData,
    }
}

/// Mint inversion evidence from a performed undo.
///
/// The `Undo: InverseOf<Original>` bound is what makes this specific: evidence
/// that *some* action ran does not license a redo of `Original`.
pub fn prove_inversion<Undo, Original, Authority, Outcome>(
    _performed_undo: &Performed<Undo, Authority, Outcome>,
) -> Inverts<Original, Authority>
where
    Undo: ActionMarker + InverseOf<Original>,
    Original: ActionMarker,
    Authority: AuthorityMarker,
{
    Inverts {
        original: PhantomData,
        authority: PhantomData,
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::{prove_derivation, prove_inversion, ActionMarker, DerivedFrom, InverseOf, Inverts};
    use crate::effect::Performed;
    use crate::proof::AuthorityWitness;

    crate::authority_marker!(pub LedgerAuthority);

    pub struct ApplyEntry;
    impl ActionMarker for ApplyEntry {}

    pub struct ReverseEntry;
    impl ActionMarker for ReverseEntry {}
    impl InverseOf<ApplyEntry> for ReverseEntry {}

    fn witness() -> AuthorityWitness<LedgerAuthority> {
        LedgerAuthority::witness()
    }

    /// A redo is legal only against a proved undo. The gate is the parameter;
    /// there is no `already_undone: bool` for a caller to get wrong.
    fn redo(_proof: Inverts<ApplyEntry, LedgerAuthority>) -> &'static str {
        "redone"
    }

    const _: () = assert!(size_of::<DerivedFrom<ApplyEntry, LedgerAuthority>>() == 0);
    const _: () = assert!(size_of::<Inverts<ApplyEntry, LedgerAuthority>>() == 0);

    #[test]
    fn a_redo_is_reachable_only_through_a_performed_undo() {
        let undo: Performed<ReverseEntry, LedgerAuthority> = Performed::record(&witness(), ());

        assert_eq!(redo(prove_inversion(&undo)), "redone");
    }

    #[test]
    fn one_performed_action_can_license_several_successors() {
        let applied: Performed<ApplyEntry, LedgerAuthority, u8> = Performed::record(&witness(), 3);

        let first = prove_derivation(&applied);
        let second = prove_derivation(&applied);

        assert_eq!(first, second);
        assert_eq!(applied.outcome(), &3);
    }
}
