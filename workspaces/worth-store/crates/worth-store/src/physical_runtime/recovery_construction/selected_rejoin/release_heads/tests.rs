use super::observation::observe_with_read;
use super::storage::HeadWalkStorage;
use super::*;
use worth_store_physical_format::{
    PersistedRecordIdentity, ReleaseCustodyHeadBlockV1, ReleaseCustodyHeadKeyV1,
    ReleaseCustodyHeadRosterDigestV1,
};
use worth_store_physical_integrity::ReleaseCustodyHeadWalkLimitsV1;

mod fingerprint_lifetime;
mod fixture;
mod native_ownership;
mod origin_binding;

fn resident(window: &PhysicalRecoveryReadAllocation<'_>, bytes: u64) -> StoreRejoinResidentLedger {
    StoreRejoinResidentLedger::for_test(
        PhysicalRecoveryAllocationAdmission::new(
            window.store_identity(),
            window.recovery_byte_limit(),
        ),
        0,
        bytes,
    )
    .unwrap()
}

fn charged_frame(frame: &[u8], resident: &mut HeadWalkStorage<'_, '_>) -> Result<Vec<u8>, Denial> {
    let mut bytes = resident.reserve_bytes(frame.len())?;
    bytes.copy_from_slice(frame);
    Ok(bytes)
}

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

fn head(object: u8) -> ReleaseCustodyHeadEntryV1 {
    let record = |ordinal| PersistedRecordIdentity::new([object; 16], ordinal).unwrap();
    ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new([object; 16], 1).unwrap(),
        record(1),
        [1; 32],
        record(2),
        [2; 32],
        record(3),
        [3; 32],
        [4; 32],
        None,
        1,
        1,
        false,
    )
    .unwrap()
}

fn one_head() -> (DurablePhysicalRootManifest, Vec<u8>, [u8; 32]) {
    let block = ReleaseCustodyHeadBlockV1::leaf(9, 2, 1, vec![head(1)], format()).unwrap();
    let reference = block.reference(format());
    let root = DurablePhysicalRootManifest::builder(2, 9, 2, 43)
        .release_custody_head_root(Some(reference))
        .next_release_custody_head_block(2)
        .admit()
        .unwrap();
    let mut digest = ReleaseCustodyHeadRosterDigestV1::new(Some(reference), 1);
    digest.push(head(1)).unwrap();
    (root, block.encode(format()), digest.finish().1)
}

fn branched_heads() -> (DurablePhysicalRootManifest, [Vec<u8>; 3], [u8; 32]) {
    let left = ReleaseCustodyHeadBlockV1::leaf(9, 2, 1, vec![head(1)], format()).unwrap();
    let right = ReleaseCustodyHeadBlockV1::leaf(9, 2, 2, vec![head(2)], format()).unwrap();
    let branch = ReleaseCustodyHeadBlockV1::branch(
        9,
        2,
        3,
        1,
        vec![left.reference(format()), right.reference(format())],
        format(),
    )
    .unwrap();
    let reference = branch.reference(format());
    let root = DurablePhysicalRootManifest::builder(2, 9, 2, 43)
        .release_custody_head_root(Some(reference))
        .next_release_custody_head_block(4)
        .admit()
        .unwrap();
    let mut digest = ReleaseCustodyHeadRosterDigestV1::new(Some(reference), 2);
    digest.push(head(1)).unwrap();
    digest.push(head(2)).unwrap();
    (
        root,
        [
            left.encode(format()),
            right.encode(format()),
            branch.encode(format()),
        ],
        digest.finish().1,
    )
}

fn resident_preflight(rooted: bool, count: u64) -> u64 {
    let max_nodes = if count == 0 { 1 } else { count * 2 + 16 };
    let retained = count * std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64
        + max_nodes * std::mem::size_of::<SelectedArtifactSlice>() as u64;
    retained
        + if rooted {
            ReleaseCustodyHeadWalkLimitsV1::root_resident_preflight_bytes(format(), max_nodes)
                .unwrap()
        } else {
            1
        }
}

#[test]
fn rooted_head_precharges_combined_retention_before_first_read() {
    let (_directory, _media, coordination) = fixture::coordination();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let (root, frame, digest) = one_head();
    let required = resident_preflight(true, 1);
    let entries = std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64;
    let slices = 18 * std::mem::size_of::<SelectedArtifactSlice>() as u64;
    let walker =
        ReleaseCustodyHeadWalkLimitsV1::root_resident_preflight_bytes(format(), 18).unwrap();
    assert!(entries < required - 1 && slices < required - 1 && walker < required - 1);

    let mut denied_reads = 0;
    let denied = observe_with_read(
        &root,
        format(),
        1,
        digest,
        u64::MAX,
        &window,
        &mut resident(&window, required - 1),
        |_, _, resident| {
            denied_reads += 1;
            charged_frame(&frame, resident)
        },
    );
    assert!(matches!(denied, Err(Denial::Resident(_))));
    assert_eq!(denied_reads, 0);

    let mut accepted_reads = 0;
    let observed = observe_with_read(
        &root,
        format(),
        1,
        digest,
        u64::MAX,
        &window,
        &mut resident(&window, required + 128 * 1024),
        |reference, remaining, resident| {
            accepted_reads += 1;
            assert_eq!(reference, root.release_custody_head_root().unwrap());
            assert!(frame.len() as u64 <= remaining);
            charged_frame(&frame, resident)
        },
    )
    .expect("canonical rooted head fits an adequate combined resident budget");
    assert_eq!(accepted_reads, 1);
    assert_eq!(observed.entries(), &[head(1)]);
    assert_eq!((observed.count(), observed.digest()), (1, digest));
    assert!(observed.owned_heap_bytes().unwrap() + walker <= required + 128 * 1024);
}

#[test]
fn empty_head_has_canonical_digest_and_never_reads_a_block() {
    let (_directory, _media, coordination) = fixture::coordination();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let root = DurablePhysicalRootManifest::builder(2, 9, 2, 43)
        .admit()
        .unwrap();
    let digest = ReleaseCustodyHeadRosterDigestV1::new(None, 0).finish().1;
    let mut reads = 0;
    let observed = observe_with_read(
        &root,
        format(),
        0,
        digest,
        u64::MAX,
        &window,
        &mut resident(&window, resident_preflight(false, 0)),
        |_, _, _| {
            reads += 1;
            Err(Denial::MissingFrame)
        },
    )
    .expect("empty rooted roster has no block");
    assert_eq!(reads, 0);
    assert_eq!(observed.count(), 0);
    assert_eq!(observed.digest(), digest);
    assert!(observed.entries().is_empty());
}

#[test]
fn rooted_head_denies_count_mismatch_and_overflow_before_unbounded_work() {
    let (_directory, _media, coordination) = fixture::coordination();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let (root, frame, digest) = one_head();
    let mut mismatch_reads = 0;
    let mismatch = observe_with_read(
        &root,
        format(),
        2,
        digest,
        u64::MAX,
        &window,
        &mut resident(&window, resident_preflight(true, 2) + 128 * 1024),
        |_, _, resident| {
            mismatch_reads += 1;
            charged_frame(&frame, resident)
        },
    );
    assert!(matches!(mismatch, Err(Denial::CertificateRoster)));
    assert_eq!(mismatch_reads, 1);

    let mut overflow_reads = 0;
    let overflow = observe_with_read(
        &root,
        format(),
        u64::MAX,
        digest,
        u64::MAX,
        &window,
        &mut resident(&window, u64::MAX),
        |_, _, resident| {
            overflow_reads += 1;
            charged_frame(&frame, resident)
        },
    );
    assert!(matches!(overflow, Err(Denial::BoundExceeded)));
    assert_eq!(overflow_reads, 0);
}

#[test]
fn canonical_branch_cannot_grow_past_remaining_walker_residency() {
    let (_directory, _media, coordination) = fixture::coordination();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let (root, frames, digest) = branched_heads();
    let preflight = resident_preflight(true, 2);
    let mut narrow_reads = 0;
    let narrow = observe_with_read(
        &root,
        format(),
        2,
        digest,
        u64::MAX,
        &window,
        &mut resident(&window, preflight),
        |reference, remaining, resident| {
            narrow_reads += 1;
            let frame = frames[(reference.block() - 1) as usize].clone();
            assert!(frame.len() as u64 <= remaining);
            charged_frame(&frame, resident)
        },
    );
    assert!(matches!(narrow, Err(Denial::Resident(_))));
    assert_eq!(
        narrow_reads, 1,
        "rooted branch is read before stack growth is denied"
    );

    let mut adequate_reads = 0;
    let adequate = observe_with_read(
        &root,
        format(),
        2,
        digest,
        u64::MAX,
        &window,
        &mut resident(&window, preflight + 128 * 1024),
        |reference, _, resident| {
            adequate_reads += 1;
            charged_frame(&frames[(reference.block() - 1) as usize], resident)
        },
    )
    .expect("same canonical tree is accepted with sufficient walker residency");
    assert_eq!(adequate_reads, 3);
    assert_eq!(adequate.entries(), &[head(1), head(2)]);
    assert_eq!((adequate.count(), adequate.digest()), (2, digest));
}

#[test]
fn second_rooted_observation_keeps_first_roster_resident_before_read() {
    let (_directory, _media, coordination) = fixture::coordination();
    let window = PhysicalRecoveryReadAllocation::for_coordination(&coordination).unwrap();
    let (root, frame, digest) = one_head();
    let required = resident_preflight(true, 1);
    let mut probe = resident(&window, required + 128 * 1024);
    let probe_first = observe_with_read(
        &root,
        format(),
        1,
        digest,
        u64::MAX,
        &window,
        &mut probe,
        |_, _, resident| charged_frame(&frame, resident),
    )
    .unwrap();
    let first_heap = probe_first.owned_heap_bytes().unwrap();
    assert_eq!(
        probe.used(),
        first_heap,
        "walker scratch and frame were dropped"
    );

    let mut narrow = resident(&window, first_heap + required - 1);
    let first = observe_with_read(
        &root,
        format(),
        1,
        digest,
        u64::MAX,
        &window,
        &mut narrow,
        |_, _, resident| charged_frame(&frame, resident),
    )
    .unwrap();
    assert_eq!(narrow.used(), first.owned_heap_bytes().unwrap());
    let mut second_reads = 0;
    let denied = observe_with_read(
        &root,
        format(),
        1,
        digest,
        u64::MAX,
        &window,
        &mut narrow,
        |_, _, resident| {
            second_reads += 1;
            charged_frame(&frame, resident)
        },
    );
    assert!(matches!(denied, Err(Denial::Resident(_))));
    assert_eq!(second_reads, 0);
    assert_eq!(first.entries(), &[head(1)]);

    let mut ample = resident(&window, first_heap + required + 128 * 1024);
    let first = observe_with_read(
        &root,
        format(),
        1,
        digest,
        u64::MAX,
        &window,
        &mut ample,
        |_, _, resident| charged_frame(&frame, resident),
    )
    .unwrap();
    let second = observe_with_read(
        &root,
        format(),
        1,
        digest,
        u64::MAX,
        &window,
        &mut ample,
        |_, _, resident| charged_frame(&frame, resident),
    )
    .unwrap();
    assert!(first.same_bytes(&second));
    assert_eq!(
        ample.used(),
        first.owned_heap_bytes().unwrap() + second.owned_heap_bytes().unwrap()
    );
}
