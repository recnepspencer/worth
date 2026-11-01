use super::*;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

#[derive(Debug)]
struct LegacyOnly;
impl RelationalDurableColdSource for LegacyOnly {
    // All controlled callbacks inherit the actual production defaults.
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<super::super::RelationalDurableMemberCustody, Cause> {
        panic!("legacy custody not an owned read")
    }
    fn read_member(&self, _: RelationalDurableMemberLocator, _: u64) -> Result<Vec<u8>, Cause> {
        panic!("owned read fell back to legacy Vec")
    }
    fn read_commit(&self, _: PatchStreamPosition, _: u64) -> Result<Vec<u8>, Cause> {
        panic!("owned read fell back to legacy Vec")
    }
}
#[derive(Debug)]
struct LegacyCut;
impl RelationalDurableColdSource for LegacyCut {
    fn checkpoint_read_protection(
        &self,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<(RelationalDurableReadBounds, RelationalDurableReadPayload), Refusal> {
        Ok((
            RelationalDurableReadBounds {
                canonical: PatchStreamPosition(1),
                transitions: RelationalBranchTransitionSequence(1),
            },
            Box::new(()),
        ))
    }
    // An actual cut does not supply the missing owned-read callbacks.
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<super::super::RelationalDurableMemberCustody, Cause> {
        panic!("legacy custody not an owned read")
    }
    fn read_member(&self, _: RelationalDurableMemberLocator, _: u64) -> Result<Vec<u8>, Cause> {
        panic!("owned read fell back to legacy Vec")
    }
    fn read_commit(&self, _: PatchStreamPosition, _: u64) -> Result<Vec<u8>, Cause> {
        panic!("owned read fell back to legacy Vec")
    }
}
fn serial(limit: u64) -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(limit)),
        CancellationSource::new().token(),
        None,
    )
}
#[test]
fn unknown_provider_never_calls_legacy_vec_reads() {
    let host = serial(1 << 20);
    let request = ExecutionRequest::serial(&host);
    let unknown: Arc<dyn RelationalDurableColdSource> = Arc::new(LegacyOnly);
    assert!(matches!(
        RelationalDurableReadProtection::capture(&unknown, &request),
        Err(Refusal::UnsupportedProvider)
    ));
    let source: Arc<dyn RelationalDurableColdSource> = Arc::new(LegacyCut);
    let cut = RelationalDurableReadProtection::capture(&source, &request).unwrap();
    assert!(matches!(
        cut.read_commit_owned_at(&source, &request, PatchStreamPosition(1), 100),
        Err(Refusal::UnsupportedProvider)
    ));
    assert!(matches!(
        cut.read_transition_owned_at(
            &source,
            &request,
            RelationalBranchTransitionSequence(1),
            100
        ),
        Err(Refusal::UnsupportedProvider)
    ));
    assert!(matches!(
        cut.read_member_owned_at(
            &source,
            &request,
            RelationalDurableMemberLocator {
                sha256: [0; 32],
                length: 1
            },
            100
        ),
        Err(Refusal::UnsupportedProvider)
    ));
    assert!(matches!(
        cut.read_commit_owned_at(&unknown, &request, PatchStreamPosition(1), 100),
        Err(Refusal::ForeignSource)
    ));
}
#[test]
fn memory_cut_keeps_original_buffers_and_refunds_after_last_owned_clone() {
    use worth_testkit_relational::durable_medium::{
        bytes::MemberBytes, OwnedNativeReadSnapshot, OwnedSnapshotDenial,
    };
    let host = serial(1 << 20);
    let request = ExecutionRequest::serial(&host);
    let mut original = vec![57u8; 57];
    let transition = [31u8; 31];
    let snapshot = OwnedNativeReadSnapshot::capture(
        &request,
        [(1, original.as_slice())].into_iter(),
        [(1, transition.as_slice())].into_iter(),
        &MemberBytes::default(),
    )
    .unwrap();
    assert_eq!(snapshot.bounds()[0], 1);
    assert_eq!(snapshot.bounds()[1], 1);
    original.fill(0);
    let owned = snapshot.canonical(&request, 1, 57).unwrap();
    assert_eq!(owned.bytes(), &[57u8; 57]);
    assert_eq!(
        snapshot.transition(&request, 1, 31).unwrap().bytes(),
        transition
    );
    assert!(matches!(
        snapshot.canonical(&request, 1, 56),
        Err(OwnedSnapshotDenial::Limit {
            length: 57,
            limit: 56
        })
    ));
    let clone = owned.clone();
    drop(owned);
    drop(snapshot);
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 57);
    drop(clone);
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
    let zero = serial(0);
    let denied = ExecutionRequest::serial(&zero);
    assert!(matches!(
        OwnedNativeReadSnapshot::capture(
            &denied,
            [(1, original.as_slice())].into_iter(),
            std::iter::empty(),
            &MemberBytes::default()
        ),
        Err(OwnedSnapshotDenial::Request(_))
    ));
    assert_eq!(zero.memory().unwrap().charged_memory_bytes(), 0);
}
