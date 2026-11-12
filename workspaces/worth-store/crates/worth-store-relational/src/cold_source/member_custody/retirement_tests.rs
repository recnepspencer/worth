//! Exact three private tests authorized by the M3 cold member prerequisite.
use super::*;
use crate::cold_source::StoreColdSource;
use std::collections::BTreeMap;
use std::{
    any::Any,
    sync::atomic::{AtomicBool, Ordering},
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::durability::{
    RelationalDurableColdSource, RelationalDurableMemberCustody, RelationalDurableMemberLocator,
    RelationalDurableMemberRetirement as Task, RelationalDurableMemberRetirementObservation as O,
    RelationalDurableMemberRetirementRefusal as R,
};
mod directory;
use directory::{assert_cold_reads, directory, Directory};

fn request(cancellation: &CancellationSource) -> SerialRequest {
    SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None)
}
fn counts(source: &StoreColdSource) -> BTreeMap<RelationalDurableMemberLocator, usize> {
    source
        .custody
        .state
        .read()
        .unwrap()
        .counts
        .iter()
        .map(|(key, value)| (*key, *value))
        .collect()
}
fn finish(task: &mut Task, execution: &ExecutionRequest<'_, '_>, source: &StoreColdSource) {
    loop {
        match task.advance(execution, source) {
            O::Advanced => {}
            O::Complete => break,
            other => panic!("unexpected retirement: {other:?}"),
        }
    }
}
#[test]
fn selected_directory_guard_retirement_preserves_partial_custody_and_cold_reads() {
    let cancellation = CancellationSource::new();
    let host = request(&cancellation);
    let execution = ExecutionRequest::serial(&host);
    let fixture = directory(&execution);
    let baseline = counts(&fixture.source);
    let mut task = fixture
        .source
        .retain_members(&fixture.members)
        .unwrap()
        .into_retirement();
    assert!(matches!(
        task.advance(&execution, fixture.source.as_ref()),
        O::Advanced
    ));
    let registered = counts(&fixture.source);
    let held = fixture.source.custody.state.write().unwrap();
    assert!(matches!(
        task.advance(&execution, fixture.source.as_ref()),
        O::Busy
    ));
    assert_eq!(
        held.counts
            .iter()
            .map(|(key, value)| (*key, *value))
            .collect::<BTreeMap<_, _>>(),
        registered
    );
    drop(held);
    assert!(matches!(
        task.advance(&execution, fixture.source.as_ref()),
        O::Advanced
    ));
    let partial = counts(&fixture.source);
    let mut expected = registered.clone();
    let first = fixture.members[0];
    let count = expected.get_mut(&first).unwrap();
    *count -= 1;
    if *count == 0 {
        expected.remove(&first);
    }
    assert_eq!(partial, expected, "exactly one actual member retired");
    cancellation.cancel();
    assert!(matches!(
        task.advance(&execution, fixture.source.as_ref()),
        O::Stopped(_)
    ));
    assert_eq!(counts(&fixture.source), partial);
    let fresh_cancel = CancellationSource::new();
    let fresh = request(&fresh_cancel);
    finish(
        &mut task,
        &ExecutionRequest::serial(&fresh),
        &fixture.source,
    );
    assert_eq!(counts(&fixture.source), baseline);
    assert_cold_reads(&fixture, &ExecutionRequest::serial(&fresh));
    assert!(
        matches!(
            task.advance(&execution, &FalseComplete::default()),
            O::Complete
        ),
        "actual terminal fact precedes late stop/source change"
    );
    let retired = task.into_retired().unwrap();
    let serving = fixture.serving.clone();
    drop(fixture); // Explicitly UNBUDGETED runtime/source teardown.
    let serving = Arc::try_unwrap(serving)
        .ok()
        .expect("completed nonempty guard detached physical protection before inert reclaim");
    let _closed = serving.close();
    retired.reclaim();
}

#[derive(Debug, Default)]
struct FalseComplete {
    callback_entered: AtomicBool,
}
impl RelationalDurableColdSource for FalseComplete {
    fn validate_member_custody_retirement(&self, _: &(dyn Any + Send + Sync)) -> Result<(), R> {
        self.callback_entered.store(true, Ordering::SeqCst);
        Ok(())
    }
    fn advance_member_custody_retirement(
        &self,
        _: &ExecutionRequest<'_, '_>,
        _: &mut (dyn Any + Send + Sync),
    ) -> O {
        self.callback_entered.store(true, Ordering::SeqCst);
        O::Complete
    }
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        panic!("no fake positive custody")
    }
    fn read_member(&self, _: RelationalDurableMemberLocator, _: u64) -> Result<Vec<u8>, Cause> {
        panic!("no fake member bytes")
    }
    fn read_commit(
        &self,
        _: worth_relational::facade::publication::PatchStreamPosition,
        _: u64,
    ) -> Result<Vec<u8>, Cause> {
        panic!("no fake artifact")
    }
}

#[test]
fn issued_source_identity_refuses_fake_and_switched_sources_before_callbacks() {
    let cancellation = CancellationSource::new();
    let host = request(&cancellation);
    let execution = ExecutionRequest::serial(&host);
    let fixture = directory(&execution);
    let foreign = StoreColdSource::new(fixture.serving.clone());
    let mut owner = fixture
        .source
        .retain_custody_owner(&fixture.members)
        .unwrap();
    assert!(fixture
        .source
        .validate_member_custody_retirement(&owner)
        .is_ok());
    assert_eq!(
        foreign.validate_member_custody_retirement(&owner),
        Err(R::ForeignCustody),
        "genuine same-Serving guard independently checks actual registry affinity"
    );
    let other_directory = tempfile::tempdir().unwrap();
    let other_serving = Arc::new(crate::cold_source::test_store::initialize(
        other_directory.path(),
    ));
    let registered_guard = counts(&fixture.source);
    let original_serving = owner.serving.replace(other_serving.clone());
    assert_eq!(
        fixture.source.validate_member_custody_retirement(&owner),
        Err(R::ForeignCustody),
        "negative-only genuine guard mutation independently checks Serving affinity"
    );
    assert_eq!(counts(&fixture.source), registered_guard);
    owner.serving = original_serving; // Restore authentic binding before any release.
    let _other_closed = Arc::try_unwrap(other_serving).ok().unwrap().close();
    drop(owner); // Explicitly unbudgeted original registration cleanup.
    let fake = FalseComplete::default();
    let mut task = fixture
        .source
        .retain_members(&fixture.members)
        .unwrap()
        .into_retirement();
    let registered = counts(&fixture.source);
    assert!(matches!(
        task.advance(&execution, &fake),
        O::Refused(R::ForeignSource)
    ));
    assert!(!fake.callback_entered.load(Ordering::SeqCst));
    assert!(matches!(
        task.advance(&execution, foreign.as_ref()),
        O::Refused(R::ForeignSource)
    ));
    assert_eq!(counts(&fixture.source), registered);
    assert!(matches!(
        task.advance(&execution, fixture.source.as_ref()),
        O::Advanced
    ));
    assert!(matches!(
        task.advance(&execution, fixture.source.as_ref()),
        O::Advanced
    ));
    let partial = counts(&fixture.source);
    assert!(matches!(
        task.advance(&execution, &fake),
        O::Refused(R::ForeignSource)
    ));
    assert!(matches!(
        task.advance(&execution, foreign.as_ref()),
        O::Refused(R::ForeignSource)
    ));
    assert!(!fake.callback_entered.load(Ordering::SeqCst));
    assert_eq!(counts(&fixture.source), partial);
    finish(&mut task, &execution, &fixture.source);
    task.into_retired().unwrap().reclaim();
    let mut opaque = RelationalDurableMemberCustody::new(()).into_retirement();
    assert!(matches!(
        opaque.advance(&execution, fixture.source.as_ref()),
        O::Refused(R::UnsupportedProvider)
    ));
    let mut foreign_payload =
        RelationalDurableMemberCustody::new_from_source((), fixture.source.weak_self.clone())
            .into_retirement();
    assert!(matches!(
        foreign_payload.advance(&execution, fixture.source.as_ref()),
        O::Refused(R::ForeignCustody)
    ));
    assert_cold_reads(&fixture, &execution);
}

#[test]
fn shared_local_retirement_preserves_other_owners_and_unbudgeted_drop_retires_only_remaining_suffix(
) {
    let cancellation = CancellationSource::new();
    let host = request(&cancellation);
    let execution = ExecutionRequest::serial(&host);
    let fixture = directory(&execution);
    let baseline = counts(&fixture.source);
    let custody = fixture.source.retain_members(&fixture.members).unwrap();
    let last = custody.clone();
    let mut shared = custody.into_retirement();
    let registered = counts(&fixture.source);
    assert!(matches!(
        shared.advance(&execution, fixture.source.as_ref()),
        O::Complete
    ));
    assert_eq!(
        counts(&fixture.source),
        registered,
        "local share release does not retire another clone"
    );
    shared.into_retired().unwrap().reclaim();
    let mut unique = last.into_retirement();
    assert!(matches!(
        unique.advance(&execution, fixture.source.as_ref()),
        O::Advanced
    ));
    assert!(matches!(
        unique.advance(&execution, fixture.source.as_ref()),
        O::Advanced
    ));
    drop(unique); // Explicitly UNBUDGETED abandonment must retire only suffix.
    assert_eq!(counts(&fixture.source), baseline);
    assert_cold_reads(&fixture, &execution);
    let mut empty = fixture
        .source
        .retain_members(&[])
        .unwrap()
        .into_retirement();
    assert!(matches!(
        empty.advance(&execution, fixture.source.as_ref()),
        O::Advanced
    ));
    assert!(matches!(
        empty.advance(&execution, fixture.source.as_ref()),
        O::Complete
    ));
    let empty_retired = empty.into_retired().unwrap();
    let mut pending = fixture
        .source
        .retain_members(&fixture.members)
        .unwrap()
        .into_retirement();
    assert!(matches!(
        pending.advance(&execution, fixture.source.as_ref()),
        O::Advanced
    ));
    assert!(matches!(
        pending.advance(&execution, fixture.source.as_ref()),
        O::Advanced
    ));
    let weak = fixture.source.weak_self.clone();
    let Directory {
        runtime,
        source,
        serving,
        _directory,
        ..
    } = fixture;
    drop(runtime);
    drop(source); // UNBUDGETED, outside bounded advancement.
    assert!(
        weak.upgrade().is_none(),
        "pending task retains weak issuer only"
    );
    let serving = Arc::try_unwrap(serving)
        .err()
        .expect("pending actual guard retains physical protection");
    let fake = FalseComplete::default();
    assert!(matches!(
        pending.advance(&execution, &fake),
        O::Refused(R::ForeignSource)
    ));
    assert!(!fake.callback_entered.load(Ordering::SeqCst));
    drop(pending); // Original suffix and physical protection, not fresh custody.
    let serving = Arc::try_unwrap(serving)
        .ok()
        .expect("retired empty guard no longer retains Serving");
    let _closed = serving.close();
    empty_retired.reclaim();
}
