use sha2::{Digest, Sha256};

use super::head_effect_tests::{record, upsert_projection};
use super::*;
use crate::{
    BlobGenerationPublicationV1, BlobReclaimSourceBasisV1, PersistedPhysicalRecoveryOperation,
    PersistedPhysicalRecoveryRootState, PhysicalRecordFormatDeclaration,
    ReleaseCustodyHeadBlockReferenceV1 as HeadRef, ReleaseCustodyHeadBlockV1,
    ReleaseCustodyHeadEntryV1 as HeadEntry, ReleaseCustodyHeadKeyV1, ReleaseCustodyHeadMutationV1,
    ReleaseCustodyHeadPathNodeV1, ReleaseCustodyHeadTransitionLimitsV1,
    ReleaseCustodyHeadTransitionV1, ReleasedGenerationReclaimBasisV1,
};

#[path = "release_head_retirement_tests/body_grammar.rs"]
mod body_grammar;
#[path = "release_head_retirement_tests/record_less_member.rs"]
mod record_less_member;
#[path = "release_head_retirement_tests/wire.rs"]
mod wire;
use wire::WireBody;

const TREE: u64 = 6;
const SOURCE_GENERATION: u64 = 11;
const STORE: [u8; 16] = [7; 16];
const DECLARATION_SHA256: [u8; 32] = [0x44; 32];
const LIMITS: PhysicalRecoveryProjectionDecodeLimits = PhysicalRecoveryProjectionDecodeLimits {
    frames: 0,
    record_identities: 0,
    placements: 0,
    segment_updates: 0,
    manifests: 0,
    total_entries: 2,
    inline_allocations: 0,
};
const MALFORMED: PhysicalRecoveryProjectionDenial = PhysicalRecoveryProjectionDenial::Malformed;

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

fn basis() -> ReleasedGenerationReclaimBasisV1 {
    let publication = BlobGenerationPublicationV1::new(
        STORE,
        [2; 16],
        [3; 16],
        3,
        record(3),
        [4; 32],
        8,
        [5; 32],
        64 * 1024,
        [6; 32],
    )
    .unwrap();
    ReleasedGenerationReclaimBasisV1::new(
        publication,
        record(4),
        Sha256::digest(publication.encode()).into(),
        [6; 32],
    )
    .unwrap()
}

fn basis_digest() -> [u8; 32] {
    BlobReclaimSourceBasisV1::ReleasedGeneration(basis()).digest(STORE)
}

fn head(
    object: [u8; 16],
    generation: u64,
    source_basis_digest: [u8; 32],
    source_root_generation: u64,
    terminal: bool,
) -> HeadEntry {
    HeadEntry::new(
        ReleaseCustodyHeadKeyV1::new(object, generation).unwrap(),
        record(9),
        [5; 32],
        record(7),
        [8; 32],
        record(8),
        [9; 32],
        source_basis_digest,
        None,
        source_root_generation,
        1,
        terminal,
    )
    .unwrap()
}

/// The terminal head of exactly the object key the source basis names.
fn terminal_head() -> HeadEntry {
    head(
        basis().object(),
        basis().generation(),
        basis_digest(),
        9,
        true,
    )
}

fn survivor() -> HeadEntry {
    head([9; 16], 1, [0x33; 32], 8, false)
}

/// A one-leaf selected head tree written at generation 10 as block 1.
fn source_tree(entries: Vec<HeadEntry>) -> (HeadRef, Vec<ReleaseCustodyHeadPathNodeV1>) {
    let leaf = ReleaseCustodyHeadBlockV1::leaf(TREE, 10, 1, entries, format()).unwrap();
    let reference = leaf.reference(format());
    let node = ReleaseCustodyHeadPathNodeV1::new(reference, leaf.encode(format()));
    (reference, vec![node])
}

fn planned_retirement(
    root: HeadRef,
    path: &[ReleaseCustodyHeadPathNodeV1],
    expected_prior: HeadEntry,
) -> ReleaseCustodyHeadTransitionV1 {
    ReleaseCustodyHeadTransitionV1::plan(
        Some(root),
        2,
        path,
        ReleaseCustodyHeadMutationV1::RetireTerminal { expected_prior },
        SOURCE_GENERATION + 1,
        TREE,
        format(),
        ReleaseCustodyHeadTransitionLimitsV1::new(1, 1, 8 * 16_384).unwrap(),
    )
    .expect("the pure planner admits retiring a present terminal head")
}

fn retire(
    entries: Vec<HeadEntry>,
    expected_prior: HeadEntry,
    declaration_record: PersistedRecordIdentity,
    declaration_frame_sha256: [u8; 32],
) -> Result<PersistedTerminalReleaseHeadRetirementV1, PhysicalRecoveryProjectionDenial> {
    let (root, path) = source_tree(entries);
    let planned = planned_retirement(root, &path, expected_prior);
    PersistedTerminalReleaseHeadRetirementV1::new(
        SOURCE_GENERATION,
        declaration_record,
        declaration_frame_sha256,
        TREE,
        basis(),
        path,
        planned,
        format(),
    )
}

fn retirement(entries: Vec<HeadEntry>) -> PersistedTerminalReleaseHeadRetirementV1 {
    retire(entries, terminal_head(), record(2), DECLARATION_SHA256).unwrap()
}

fn root_state() -> PersistedPhysicalRecoveryRootState {
    PersistedPhysicalRecoveryRootState::new(4096, 1, 32, vec![], None, None).unwrap()
}

fn retirement_projection(
    retirement: PersistedTerminalReleaseHeadRetirementV1,
) -> PersistedPhysicalRecoveryProjection {
    PersistedPhysicalRecoveryProjection::new_with_operation(
        SOURCE_GENERATION,
        root_state(),
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        PersistedPhysicalRecoveryOperation::TerminalReleaseHeadRetired(retirement),
    )
    .expect("a record-less terminal head retirement is one whole projection")
}

fn decode(
    bytes: &[u8],
) -> Result<PersistedPhysicalRecoveryProjection, PhysicalRecoveryProjectionDenial> {
    PersistedPhysicalRecoveryProjection::decode(bytes, LIMITS, format())
}

#[test]
fn retiring_the_last_head_empties_the_tree_and_round_trips_the_exact_wire_layout() {
    let retirement = retirement(vec![terminal_head()]);
    assert_eq!(retirement.result_root(), None);
    assert!(retirement.node_writes().is_empty());
    assert_eq!(
        retirement.result_next_block(),
        retirement.source_next_block()
    );
    assert_eq!(
        retirement.mutation(),
        ReleaseCustodyHeadMutationV1::RetireTerminal {
            expected_prior: terminal_head()
        }
    );
    let projection = retirement_projection(retirement.clone());
    let bytes = projection.encode();
    assert_eq!(bytes, WireBody::of(&retirement).projection_bytes());
    assert_eq!(decode(&bytes), Ok(projection.clone()));
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode_frames(&bytes, LIMITS),
        Err(MALFORMED),
        "a head-tree claim cannot be admitted without the bootstrap format"
    );
    let claim = projection.operation().release_head_tree_claim().unwrap();
    assert_eq!(claim.entry_count(), Some(1));
    assert_eq!(claim.framed_bytes(), retirement.framed_bytes());
    assert_eq!(projection.owned_heap_bytes(), retirement.owned_heap_bytes());
}

#[test]
fn retiring_one_head_leaves_exactly_the_other_entries_and_round_trips() {
    let retirement = retirement(vec![terminal_head(), survivor()]);
    let result_root = retirement
        .result_root()
        .expect("a surviving head keeps a tree");
    let [write] = retirement.node_writes() else {
        panic!("one leaf is rewritten")
    };
    assert_eq!(write.reference(), result_root);
    let (leaf, _) = ReleaseCustodyHeadBlockV1::decode(write.frame(), result_root, TREE).unwrap();
    assert_eq!(leaf.entries(), Some(&[survivor()][..]));
    let projection = retirement_projection(retirement.clone());
    let bytes = projection.encode();
    assert_eq!(bytes, WireBody::of(&retirement).projection_bytes());
    assert_eq!(decode(&bytes), Ok(projection));

    let frame = write.frame();
    let offset = bytes.len() - 1;
    assert_eq!(bytes[offset], frame[frame.len() - 1]);
    let mut substituted = bytes;
    substituted[offset] ^= 1;
    assert_eq!(decode(&substituted), Err(MALFORMED));
}

#[test]
fn each_head_mutation_decodes_only_under_its_own_tag() {
    let retirement = retirement(vec![terminal_head()]);
    let mut body = WireBody::of(&retirement);
    assert!(decode(&body.projection_bytes()).is_ok());
    body.mutation_tag = 1;
    assert_eq!(decode(&body.projection_bytes()), Err(MALFORMED));

    let (projection, format) = upsert_projection();
    let PersistedPhysicalRecoveryOperation::RecordsDropped {
        head_effect: Some(effect),
        ..
    } = projection.operation()
    else {
        panic!("fixture must carry a head effect")
    };
    let encoded = encode_head_effect(effect);
    let decode_upsert = |bytes: &[u8]| {
        decode_head_effect(
            bytes,
            11,
            &mut 2,
            format,
            &mut decode_storage::UnrestrictedDecodeStorage,
        )
        .map_err(decode_storage::projection_denial)
    };
    assert_eq!(decode_upsert(&encoded).as_ref(), Ok(effect));
    // tree identity, source basis field, absent source root flag, next block.
    let tag = 8 + 8 + effect.source_basis().encode().len() + 1 + 8;
    assert_eq!(encoded[tag], 1);
    let mut retire_tagged = encoded.clone();
    retire_tagged[tag] = 2;
    assert_eq!(decode_upsert(&retire_tagged), Err(MALFORMED));

    // An upsert always names its result root: the reference is bare on the
    // wire, so an absent-root flag in its place cannot decode.
    let result_root = tag + 1 + 1 + HeadEntry::ENCODED_BYTES;
    let mut expected = [0; HeadRef::ENCODED_BYTES];
    effect.result_root().encode_into(&mut expected);
    assert_eq!(
        &encoded[result_root..result_root + HeadRef::ENCODED_BYTES],
        &expected
    );
    let mut rootless = encoded[..result_root].to_vec();
    rootless.push(0);
    rootless.extend_from_slice(&encoded[result_root + HeadRef::ENCODED_BYTES..]);
    assert_eq!(decode_upsert(&rootless), Err(MALFORMED));
}

#[test]
fn a_nonterminal_head_cannot_be_retired() {
    let nonterminal = head(
        basis().object(),
        basis().generation(),
        basis_digest(),
        9,
        false,
    );
    let (root, path) = source_tree(vec![nonterminal]);
    let body = |prior: HeadEntry, root: HeadRef, path: &[ReleaseCustodyHeadPathNodeV1]| WireBody {
        mutation_tag: 2,
        source_root: root,
        expected_prior: prior,
        result_root: None,
        result_next_block: 2,
        path: path.to_vec(),
        writes: vec![],
    };
    assert_eq!(
        decode(&body(nonterminal, root, &path).projection_bytes()),
        Err(MALFORMED)
    );
    let (terminal_root, terminal_path) = source_tree(vec![terminal_head()]);
    assert!(
        decode(&body(terminal_head(), terminal_root, &terminal_path).projection_bytes()).is_ok()
    );
}

#[test]
fn the_source_basis_must_bind_the_exact_retired_key() {
    let admitted = |entry: HeadEntry| retire(vec![entry], entry, record(2), DECLARATION_SHA256);
    let (object, generation) = (basis().object(), basis().generation());
    assert!(admitted(terminal_head()).is_ok());
    assert!(admitted(head(object, generation, basis_digest(), 10, true)).is_ok());
    for foreign in [
        head([4; 16], generation, basis_digest(), 9, true),
        head(object, generation + 1, basis_digest(), 9, true),
        head(object, generation, [0x55; 32], 9, true),
        head(object, generation, basis_digest(), SOURCE_GENERATION, true),
    ] {
        assert_eq!(admitted(foreign), Err(MALFORMED));
        let (root, path) = source_tree(vec![foreign]);
        let planned = planned_retirement(root, &path, foreign);
        let body = WireBody {
            mutation_tag: 2,
            source_root: root,
            expected_prior: foreign,
            result_root: planned.result_root(),
            result_next_block: planned.result_next_block(),
            path,
            writes: vec![],
        };
        assert_eq!(decode(&body.projection_bytes()), Err(MALFORMED));
    }
}

#[test]
fn the_session_declaration_binding_is_required_and_distinct_from_the_publication() {
    let attempt =
        |declaration, digest| retire(vec![terminal_head()], terminal_head(), declaration, digest);
    let admitted = attempt(record(2), DECLARATION_SHA256).unwrap();
    assert_eq!(admitted.declaration_record(), record(2));
    assert_eq!(admitted.declaration_frame_sha256(), DECLARATION_SHA256);
    assert_eq!(attempt(record(2), [0; 32]), Err(MALFORMED));
    assert_eq!(basis().publication_record(), record(4));
    assert_eq!(attempt(record(4), DECLARATION_SHA256), Err(MALFORMED));
}

#[test]
fn only_the_exact_planned_removal_of_the_carried_path_is_admitted() {
    let (root, path) = source_tree(vec![terminal_head()]);
    let (_, other_path) = source_tree(vec![terminal_head(), survivor()]);
    let construct = |path, planned| {
        PersistedTerminalReleaseHeadRetirementV1::new(
            SOURCE_GENERATION,
            record(2),
            DECLARATION_SHA256,
            TREE,
            basis(),
            path,
            planned,
            format(),
        )
    };
    let planned = || planned_retirement(root, &path, terminal_head());
    assert!(construct(path.clone(), planned()).is_ok());
    assert_eq!(construct(other_path, planned()), Err(MALFORMED));

    let upsert = ReleaseCustodyHeadTransitionV1::plan(
        None,
        1,
        &[],
        ReleaseCustodyHeadMutationV1::Upsert {
            expected_prior: None,
            next: terminal_head(),
        },
        SOURCE_GENERATION + 1,
        TREE,
        format(),
        ReleaseCustodyHeadTransitionLimitsV1::new(1, 1, 8 * 16_384).unwrap(),
    )
    .unwrap();
    assert_eq!(construct(vec![], upsert), Err(MALFORMED));
}
