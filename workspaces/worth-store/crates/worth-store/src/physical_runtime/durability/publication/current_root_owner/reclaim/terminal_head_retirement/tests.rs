//! The retirement fence admits exactly its registered member, and its
//! effect mark is the point after which a dropped attempt keeps the fence.
//! The attempt promotes its session's claim, in the owner's registry alone.

use std::num::NonZeroU64;
use std::sync::{Arc, Mutex, Weak};

use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{DurablePhysicalRootManifest, RootPublicationCell};

use super::{
    promoted_attempt, unfenced_source, BlobClaimRegistry, PhysicalBlobSessionClaim,
    PhysicalReclaimAttempt, PhysicalReclaimAttemptId, ReclaimFenceState, ReclaimPhase,
    ReclaimPurpose, TerminalHeadRetirementAdmissionDenial as Denial,
};
use crate::physical_runtime::{
    durability::PhysicalBlobSessionClaimDenial, LifecycleGeneration, PhysicalMutationIdentity,
    PhysicalOperationIdentity, PhysicalWorkGeneration, PhysicalWorkIdentity, RuntimeIdentity,
};

const ATTEMPT: PhysicalReclaimAttemptId = PhysicalReclaimAttemptId([7; 16]);

fn mutation(operation: u64) -> PhysicalMutationIdentity {
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([0x41; 16]).unwrap(),
    )
    .published_identity();
    PhysicalMutationIdentity::from_reserved_operation(PhysicalWorkIdentity::from_instance_owner(
        store,
        RuntimeIdentity::from_reopened(NonZeroU64::MIN),
        PhysicalWorkGeneration::from_lifecycle(LifecycleGeneration::from_reopened(NonZeroU64::MIN)),
        PhysicalOperationIdentity::from_reopened(NonZeroU64::new(operation).unwrap()),
    ))
}

fn root(generation: u64) -> RootPublicationCell {
    DurablePhysicalRootManifest::builder(generation, 1, 2, 1)
        .admit()
        .unwrap()
        .root_cell()
}

/// The fence exactly as the admission installs it, with its one attempt.
fn fenced() -> (
    Arc<Mutex<Option<ReclaimFenceState>>>,
    PhysicalReclaimAttempt,
) {
    let fence = Arc::new(Mutex::new(Some(ReclaimFenceState {
        id: ATTEMPT,
        expected_root: root(1),
        manifest_mutation: None,
        reservation_mutation: None,
        drop_mutation: None,
        retirement_mutation: None,
        drop_records: Vec::new(),
        displaced: Vec::new(),
        phase: ReclaimPhase::BeforeEffect,
        purpose: ReclaimPurpose::TerminalHeadRetirement,
        _capacity: None,
        _recovered_reservations: Vec::new(),
        release_certificate_pending: None,
    })));
    let attempt = PhysicalReclaimAttempt {
        fence: Arc::clone(&fence),
        state: Weak::new(),
        id: ATTEMPT,
    };
    (fence, attempt)
}

fn accepts(fence: &Mutex<Option<ReclaimFenceState>>, mutation: PhysicalMutationIdentity) -> bool {
    fence
        .lock()
        .unwrap()
        .as_ref()
        .expect("the fence is installed")
        .accepts_terminal_head_retirement(mutation)
}

fn effect_marked(fence: &Mutex<Option<ReclaimFenceState>>) -> bool {
    fence
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|state| state.phase == ReclaimPhase::RetirementEffect)
}

#[test]
fn a_fence_installed_after_the_reader_was_captured_denies_already_fenced() {
    let (fence, _attempt) = fenced();
    let installed = fence.lock().unwrap();
    assert_eq!(
        unfenced_source(installed.as_ref(), root(1), root(1)),
        Err(Denial::AlreadyFenced)
    );
    // The fence answers before the root: its owner may be mid-publication.
    assert_eq!(
        unfenced_source(installed.as_ref(), root(2), root(1)),
        Err(Denial::AlreadyFenced)
    );
}

#[test]
fn a_root_published_after_the_reader_was_captured_denies_source_root_changed() {
    assert_eq!(
        unfenced_source(None, root(2), root(1)),
        Err(Denial::SourceRootChanged)
    );
    assert_eq!(unfenced_source(None, root(1), root(1)), Ok(root(1)));
}

#[test]
fn the_attempt_promotes_its_claim_and_only_in_the_owners_registry() {
    const LOST: Denial = Denial::Claim(PhysicalBlobSessionClaimDenial::ClaimLost);
    let owner = Arc::new(BlobClaimRegistry::new(1));
    let mut claim = PhysicalBlobSessionClaim::fixture_in(&owner, [0x51; 16]);
    let foreign = Arc::new(BlobClaimRegistry::new(1));
    assert_eq!(promoted_attempt(&mut claim, &foreign), Err(LOST));
    // The denial left the claim inspecting: the owner's registry promotes it.
    let attempt = promoted_attempt(&mut claim, &owner).expect("an inspecting claim");
    assert_ne!(attempt, PhysicalReclaimAttemptId([0; 16]));
    assert_eq!(
        claim.promote_live(),
        Err(PhysicalBlobSessionClaimDenial::ClaimLost),
        "a claim promoted for the retirement admits no other producer"
    );
    assert_eq!(
        promoted_attempt(&mut claim, &owner),
        Err(LOST),
        "one claim funds one attempt"
    );
}

#[test]
fn the_fence_admits_only_the_member_registered_on_it() {
    let (fence, attempt) = fenced();
    let (member, other) = (mutation(2), mutation(3));
    assert!(!accepts(&fence, member), "nothing is registered yet");
    assert!(attempt.register_terminal_head_retirement(member));
    assert!(
        !attempt.register_terminal_head_retirement(other),
        "the fence holds one member"
    );
    assert!(accepts(&fence, member));
    assert!(!accepts(&fence, other), "another member borrowed the fence");
    assert!(attempt.mark_terminal_head_retirement_effect());
    assert!(accepts(&fence, member), "the effect is the member's own");
    assert!(!accepts(&fence, other));
    fence.lock().unwrap().as_mut().unwrap().phase = ReclaimPhase::RetirementPublished;
    assert!(
        !accepts(&fence, member),
        "a published member is not admitted twice"
    );
}

#[test]
fn only_a_fence_without_an_effect_mark_is_released_by_its_dropped_attempt() {
    let (fence, attempt) = fenced();
    assert!(
        !attempt.mark_terminal_head_retirement_effect(),
        "no member is registered"
    );
    assert!(attempt.register_terminal_head_retirement(mutation(2)));
    drop(attempt);
    assert!(
        fence.lock().unwrap().is_none(),
        "a pre-effect fence releases"
    );

    let (fence, attempt) = fenced();
    assert!(attempt.register_terminal_head_retirement(mutation(2)));
    assert!(attempt.mark_terminal_head_retirement_effect());
    assert!(effect_marked(&fence));
    assert!(
        !attempt.mark_terminal_head_retirement_effect(),
        "the effect is marked once"
    );
    drop(attempt);
    assert!(
        effect_marked(&fence),
        "a fence whose effect may exist outlives its attempt"
    );
}
