use super::*;
use std::num::NonZeroU64;

fn owner(incarnation: u64) -> HomeProgramTransitionOwner {
    HomeProgramTransitionOwner::fresh(
        [1; 16],
        HomeIncarnationFence::from_owner(NonZeroU64::new(incarnation).unwrap()),
    )
}

#[test]
fn planning_requests_require_the_attached_owner_and_incarnation_before_source_scope() {
    let original = owner(1);
    let request = original.admit_planning_request(42);
    assert_eq!(
        original
            .authority()
            .admit_planning_request(&request, [1; 16]),
        Ok(())
    );
    assert_eq!(*request.source(), 42);
    assert_eq!(
        original
            .authority()
            .admit_planning_request(&request, [9; 16]),
        Err(HomeProgramTransitionAdmissionDenial::SourceMismatch)
    );
    assert_eq!(
        owner(1)
            .authority()
            .admit_planning_request(&request, [1; 16]),
        Err(HomeProgramTransitionAdmissionDenial::ForeignOwner)
    );
    assert_eq!(
        owner(2)
            .authority()
            .admit_planning_request(&request, [9; 16]),
        Err(HomeProgramTransitionAdmissionDenial::StaleIncarnation)
    );
}

#[test]
fn transition_admission_requires_the_attached_owner_and_exact_commitment() {
    let installed = owner(1);
    let foreign = owner(1);
    let authority = installed.authority();
    let proof = installed.admit_step([7; 32], [3; 32], [5; 32], ());
    assert_eq!(authority.admit_step(&proof, &[7; 32], &[3; 32]), Ok(()));
    assert_eq!(
        authority.admit_step(&proof, &[8; 32], &[3; 32]),
        Err(HomeProgramTransitionAdmissionDenial::CommitmentMismatch)
    );
    assert_eq!(
        authority.admit_step(
            &foreign.admit_step([7; 32], [3; 32], [5; 32], ()),
            &[7; 32],
            &[3; 32]
        ),
        Err(HomeProgramTransitionAdmissionDenial::ForeignOwner)
    );
    let plan = installed.admit_plan([9; 32]);
    assert_eq!(authority.admit_plan(&plan, &[9; 32]), Ok(()));
    assert_eq!(
        authority.admit_plan(&foreign.admit_plan([9; 32]), &[9; 32]),
        Err(HomeProgramTransitionAdmissionDenial::ForeignOwner)
    );
}

#[test]
fn transition_step_grants_refuse_a_different_original_caller_association() {
    let owner = owner(1);
    let proof = owner.admit_step([7; 32], [3; 32], [5; 32], ());
    assert_eq!(
        owner.authority().admit_step(&proof, &[7; 32], &[4; 32]),
        Err(HomeProgramTransitionAdmissionDenial::CallerAssociationMismatch)
    );
    assert_eq!(
        owner.authority().admit_step(&proof, &[7; 32], &[3; 32]),
        Ok(())
    );
}

#[test]
fn earlier_incarnation_transition_proofs_refuse_before_commitment_inspection() {
    let earlier = owner(1);
    let current = owner(2).authority();
    let foreign = HomeProgramTransitionOwner::fresh(
        [2; 16],
        HomeIncarnationFence::from_owner(NonZeroU64::new(1).unwrap()),
    );
    assert_eq!(
        current.admit_step(
            &foreign.admit_step([7; 32], [3; 32], [5; 32], ()),
            &[8; 32],
            &[4; 32]
        ),
        Err(HomeProgramTransitionAdmissionDenial::ForeignOwner)
    );
    assert_eq!(
        current.admit_step(
            &earlier.admit_step([7; 32], [3; 32], [5; 32], ()),
            &[8; 32],
            &[4; 32]
        ),
        Err(HomeProgramTransitionAdmissionDenial::StaleIncarnation)
    );
    assert_eq!(
        current.admit_plan(&earlier.admit_plan([7; 32]), &[8; 32]),
        Err(HomeProgramTransitionAdmissionDenial::StaleIncarnation)
    );
}

#[test]
fn transition_proofs_retain_original_backing_without_sharing_owner_authority() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[derive(Debug)]
    struct Backing(Arc<AtomicUsize>);
    impl Drop for Backing {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let reclaimed = Arc::new(AtomicUsize::new(0));
    let backing = Arc::new(Backing(reclaimed.clone()));
    let fence = HomeIncarnationFence::from_owner(NonZeroU64::new(1).unwrap());
    let owner =
        HomeProgramTransitionOwner::from_retained_owner_backing([1; 16], fence, backing.clone());
    let other =
        HomeProgramTransitionOwner::from_retained_owner_backing([1; 16], fence, backing.clone());
    let authority = owner.authority();
    let proof = owner.admit_step([7; 32], [3; 32], [5; 32], ());
    assert_eq!(
        other.authority().admit_step(&proof, &[7; 32], &[3; 32]),
        Err(HomeProgramTransitionAdmissionDenial::ForeignOwner)
    );
    drop(backing);
    drop(owner);
    drop(other);
    assert_eq!(reclaimed.load(Ordering::SeqCst), 0);
    assert_eq!(authority.admit_step(&proof, &[7; 32], &[3; 32]), Ok(()));
    drop(authority);
    assert_eq!(reclaimed.load(Ordering::SeqCst), 0);
    drop(proof);
    assert_eq!(reclaimed.load(Ordering::SeqCst), 1);
}

#[test]
fn successor_binding_requires_the_same_home_occurrence_incarnation_and_commitment() {
    let installed = owner(1);
    let required = installed.require_transition([7; 32]);
    let successor = installed.admit_successor([7; 32], 42);
    assert_eq!(
        installed.authority().admit_successor(&required, &successor),
        Ok(())
    );
    assert_eq!(*successor.binding(), 42);
    assert_eq!(
        installed
            .authority()
            .admit_successor(&required, &installed.admit_successor([8; 32], 42)),
        Err(HomeProgramTransitionAdmissionDenial::CommitmentMismatch)
    );
    assert_eq!(
        installed
            .authority()
            .admit_successor(&required, &owner(1).admit_successor([7; 32], 42)),
        Err(HomeProgramTransitionAdmissionDenial::ForeignOwner)
    );
    assert_eq!(
        owner(2).authority().admit_successor(&required, &successor),
        Err(HomeProgramTransitionAdmissionDenial::StaleIncarnation)
    );
}
