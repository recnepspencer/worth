use std::fmt::{self, Debug};
use std::marker::PhantomData;

use crate::proof::{AuthorityMarker, AuthorityWitness};

/// A named action a runtime can perform.
///
/// A marker, never a value — the action itself is code, and what travels is
/// evidence that the code ran.
pub trait ActionMarker: 'static {}

/// Evidence that an action was performed, carrying what it produced.
///
/// Minted only by surrendering an [`AuthorityWitness`], so only the module
/// that declared the authority marker can say the action occurred. A caller
/// cannot construct one and present it as though the effect had escaped —
/// which is precisely what a caller-supplied "redispatch record" would let
/// them do.
///
/// ```
/// use worth_proof::{ActionMarker, Performed};
///
/// mod dispatcher {
///     worth_proof::authority_marker!(pub DispatchAuthority);
///
///     pub struct SendToOutbox;
///     impl worth_proof::ActionMarker for SendToOutbox {}
///
///     /// The only door: performing the effect is what produces the evidence.
///     pub fn send(payload: &str) -> worth_proof::Performed<SendToOutbox, DispatchAuthority, usize> {
///         let written = payload.len();
///         worth_proof::Performed::record(&DispatchAuthority::witness(), written)
///     }
/// }
///
/// let performed = dispatcher::send("entry");
/// assert_eq!(performed.outcome(), &5);
/// ```
/// Deliberately **not** `Clone`. Cloning evidence that an effect occurred
/// would let a retry consume the same evidence twice, which is the shape
/// R8.66 ("consume the proof, not the permission") exists to prevent.
#[must_use]
pub struct Performed<Action, Authority, Outcome = ()>
where
    Action: ActionMarker,
    Authority: AuthorityMarker,
{
    outcome: Outcome,
    action: PhantomData<Action>,
    authority: PhantomData<Authority>,
}

// Bounds land on `Outcome` alone. A derive would demand them on the action and
// authority markers too, which hold nothing.
impl<Action, Authority, Outcome> Debug for Performed<Action, Authority, Outcome>
where
    Action: ActionMarker,
    Authority: AuthorityMarker,
    Outcome: Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Performed")
            .field("outcome", &self.outcome)
            .finish()
    }
}

impl<Action, Authority, Outcome> PartialEq for Performed<Action, Authority, Outcome>
where
    Action: ActionMarker,
    Authority: AuthorityMarker,
    Outcome: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.outcome == other.outcome
    }
}

impl<Action, Authority, Outcome> Eq for Performed<Action, Authority, Outcome>
where
    Action: ActionMarker,
    Authority: AuthorityMarker,
    Outcome: Eq,
{
}

impl<Action, Authority, Outcome> Performed<Action, Authority, Outcome>
where
    Action: ActionMarker,
    Authority: AuthorityMarker,
{
    /// Record that the action was performed.
    ///
    /// Call this **after** the effect, never before. The witness proves who
    /// may record; only placement proves the effect actually happened, and no
    /// type system reaches that far. Keep the call in the same function as the
    /// effect so a reader can see both at once.
    pub fn record(_authority: &AuthorityWitness<Authority>, outcome: Outcome) -> Self {
        Self {
            outcome,
            action: PhantomData,
            authority: PhantomData,
        }
    }

    pub fn outcome(&self) -> &Outcome {
        &self.outcome
    }

    pub fn into_outcome(self) -> Outcome {
        self.outcome
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::{ActionMarker, Performed};

    crate::authority_marker!(pub DispatchAuthority);

    pub struct SendToOutbox;
    impl ActionMarker for SendToOutbox {}

    pub(super) fn send(payload: &str) -> Performed<SendToOutbox, DispatchAuthority, usize> {
        Performed::record(&DispatchAuthority::witness(), payload.len())
    }

    const _: () = assert!(
        size_of::<Performed<SendToOutbox, DispatchAuthority, usize>>() == size_of::<usize>()
    );
    const _: () = assert!(size_of::<Performed<SendToOutbox, DispatchAuthority>>() == 0);

    #[test]
    fn performing_the_effect_is_what_produces_the_evidence() {
        let performed = send("entry");

        assert_eq!(performed.outcome(), &5);
        assert_eq!(performed.into_outcome(), 5);
    }
}
