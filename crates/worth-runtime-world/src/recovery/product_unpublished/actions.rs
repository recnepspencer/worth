use super::{ProductUnpublishedCause, ProductUnpublishedNextAction};
use crate::publication::{
    CompositeAttemptProgress, RelationalAttemptProgressPosture, SignalAttemptProgressPosture,
};

/// The bound is the exact number of distinct actions a retained record can
/// legally offer, so a retained list is never silently truncated.
const RETAINED_NEXT_ACTION_CAPACITY: usize = 5;

#[derive(Debug)]
pub(crate) struct RetainedNextActions {
    actions: [ProductUnpublishedNextAction; RETAINED_NEXT_ACTION_CAPACITY],
    length: u8,
}

impl RetainedNextActions {
    pub(crate) fn from_vec(actions: Vec<ProductUnpublishedNextAction>) -> Self {
        assert!(
            actions.len() <= RETAINED_NEXT_ACTION_CAPACITY,
            "retained recovery actions exceed the fixed contract bound"
        );
        let mut retained = Self {
            actions: [ProductUnpublishedNextAction::Inspect; RETAINED_NEXT_ACTION_CAPACITY],
            length: actions.len() as u8,
        };
        retained.actions[..actions.len()].copy_from_slice(&actions);
        retained
    }

    pub(crate) fn as_slice(&self) -> &[ProductUnpublishedNextAction] {
        &self.actions[..usize::from(self.length)]
    }
}

/// The single authority for what a retained owner-effect record permits next.
/// Every retained record, recovery custody transition, and close report derives
/// its list here, so two records that retain the same owner progress can never
/// advertise different continuations. The derivation is total: every
/// `CompositeAttemptProgress` shape yields a list, and the list never exceeds
/// the fixed contract bound.
pub(crate) fn next_actions_for_progress(
    progress: &CompositeAttemptProgress,
    cause: ProductUnpublishedCause,
) -> Vec<ProductUnpublishedNextAction> {
    let mut actions = Vec::with_capacity(RETAINED_NEXT_ACTION_CAPACITY);
    if settlement_is_owed(progress) {
        actions.push(ProductUnpublishedNextAction::SettleOwnerEffects);
    }
    if publication_evidence_is_settled(progress) {
        actions.push(ProductUnpublishedNextAction::StartFreshCompositePublication);
    }
    actions.push(ProductUnpublishedNextAction::ReleaseObligations);
    actions.push(ProductUnpublishedNextAction::Inspect);
    if owner_closure_is_permitted(cause) {
        actions.push(ProductUnpublishedNextAction::CloseOwner);
    }
    assert!(
        actions.len() <= RETAINED_NEXT_ACTION_CAPACITY,
        "retained recovery actions exceed the fixed contract bound"
    );
    actions
}

/// Closing the owner is a continuation only for a record whose own cause says
/// an owner-issued authority this world depended on is gone; every other cause
/// leaves a live owner the caller can still settle or release against, so
/// offering closure there would advertise an action the record cannot justify.
///
/// `ProductPublicationLost` is deliberately not that record. It names a product
/// reference race whose loser observed a live owner on both sides, and the
/// cause is a bare discriminant carrying no owner-unavailable evidence, so
/// nothing here can derive owner closure from it. A derived continuation never
/// guesses: if that route ever has to permit closure, it must carry the
/// evidence that justifies it, not a wider match here.
fn owner_closure_is_permitted(cause: ProductUnpublishedCause) -> bool {
    matches!(cause, ProductUnpublishedCause::OwnerLost)
}

/// A record owes settlement while its Relational commit is performed but not
/// settled and no Signal movement has been recorded against it.
///
/// Publication settles the Relational leg before the pre-advance gate, and the
/// gate sits between that settlement and the Signal advance, so a publication
/// that still owes settlement provably never reached the Signal owner and is
/// always `Untouched` here.
///
/// Fork finalization is not what the Signal conjunct separates out.
/// `RelationalAttemptProgress::forked` carries no commit evidence, and
/// `requires_settlement()` matches on evidence, so fork-only progress is
/// already excluded by the first conjunct -- as
/// `branch/retirement_tests/fork_finalization_race.rs` states, settlement is
/// structurally false on that route. The Signal conjunct is defence in depth
/// against a shape that pairs a moved Signal with an unsettled Relational
/// commit, not the discriminator between publication and fork evidence.
fn settlement_is_owed(progress: &CompositeAttemptProgress) -> bool {
    progress.relational_requires_settlement()
        && progress.signal_posture() == SignalAttemptProgressPosture::Untouched
}

/// A fresh composite publication may start only once the retained record's
/// publication evidence is settled: the Relational commit named by the record
/// is durable and owes this world nothing further, so a new Query-admitted
/// operation can name it as its basis. `Performed` fork evidence is
/// branch-creation evidence rather than publication evidence and never permits
/// a fresh composite attempt on its own.
fn publication_evidence_is_settled(progress: &CompositeAttemptProgress) -> bool {
    progress.relational_posture() == RelationalAttemptProgressPosture::Settled
}
