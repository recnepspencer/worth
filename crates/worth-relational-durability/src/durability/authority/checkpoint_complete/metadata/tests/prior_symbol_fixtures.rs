//! Pure manifest fixtures and invariants for prior cold symbol page carry-forward tests.
use crate::durability::data::*;
use crate::durability::log::native_checkpoint_codec::member_frame;
use contracts::lineage::data::artifacts::checkpoint_artifact::LineageCheckpointDigestBasis;
use contracts::symbols::data::stored_text::StoredSymbolText;
use contracts::symbols::data::string_interner::{StringInterner, StringInternerOwnerPort as _};
use contracts::symbols::data::symbol::Symbol;
use contracts::symbols::data::symbol_table_snapshot::SymbolTableSnapshot;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use worth_execution::ExecutionOrderedMap;
use worth_relational_branch::internal_port::branch::registry::checkpoint_capture::data::CheckpointBranchMetadata;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
use worth_relational_storage::internal_port::storage::data::member_frame::MemberKind;
use worth_relational_storage::internal_port::storage::data::symbol_page_roster::RelationalCheckpointSymbolBucket;

pub(super) fn manifest_with_symbols(
    symbols: CheckpointPages,
    symbol_buckets: CheckpointSymbolBuckets,
    symbol_identity_frontier: u32,
) -> RelationalCheckpointManifest {
    RelationalCheckpointManifestDraft {
        branch_data: CheckpointBranchMetadata::from_ordinary_parts(
            "dictionary-pages-test".into(),
            vec![],
            vec![],
            vec![],
        ),
        coverage: CheckpointCoverage {
            up_to_commit: None,
            up_to_version: None,
            up_to_branch_transition: Default::default(),
        },
        covered_position: contracts::publication::patch::data::patch_position::PatchStreamPosition(
            0,
        ),
        schemas: Default::default(),
        record_identity: Default::default(),
        aspect_contracts: Vec::new(),
        index_generation_frontier: 1,
        index_definition_frontier: 1,
        index_definitions: Default::default(),
        roots: Default::default(),
        retained_empty_components: Vec::new(),
        mirror_schema: None,
        mirror: Vec::new(),
        history_dependencies: Vec::new(),
        commits: Default::default(),
        canonical_members: Default::default(),
        symbols,
        symbol_identity_frontier,
        symbol_buckets,
        lineage: Default::default(),
        lineage_events: Default::default(),
        lineage_basis: CheckpointLineageBasis::from_ordinary(LineageCheckpointDigestBasis::new(
            0,
            Vec::new(),
            0,
            0,
        )),
        derived_indexes: member_frame::locator(b"derived-indexes"),
        allocations: Default::default(),
    }
    .seal_ordinary()
}

pub(super) fn interner_with(frontier: Option<u32>, words: &[&str]) -> StringInterner {
    let mut interner = StringInterner::default();
    if let Some(frontier) = frontier {
        interner.reserve_frontier(frontier).unwrap();
    }
    for word in words {
        interner.intern(word).unwrap();
    }
    interner
}

pub(super) fn prior_member_bytes(
    members: &ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
) -> BTreeMap<RelationalDurableMemberLocator, Vec<u8>> {
    let mut map = BTreeMap::new();
    for (&locator, member) in members.iter() {
        if let RelationalCheckpointMember::New { bytes, .. } = member {
            map.insert(locator, bytes.bytes().to_vec());
        }
    }
    map
}

pub(super) fn assert_valid_and_complete(
    by_id: &CheckpointPages,
    by_text: &CheckpointSymbolBuckets,
    members: &ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    prior_bytes: &BTreeMap<RelationalDurableMemberLocator, Vec<u8>>,
    expected: &[(u32, &str)],
) {
    let mut previous = None;
    let mut range = None;
    let mut occupied = [0_u64; 4];
    let mut seen_members = BTreeSet::new();
    let mut recovered = BTreeMap::new();
    for page in by_id {
        assert!(page.valid(), "page must be valid: {page:?}");
        let key = (page.range(), page.first);
        assert!(previous.is_none_or(|prev| key > prev));
        assert!(seen_members.insert(page.member));
        if range != Some(page.range()) {
            occupied = [0; 4];
            range = Some(page.range());
        }
        for (prior, incoming) in occupied.iter_mut().zip(page.membership) {
            assert_eq!(*prior & incoming, 0);
            *prior |= incoming;
        }
        previous = Some(key);
        let snapshot = decode_symbol_member(page.member, members, prior_bytes);
        for (symbol, text) in snapshot.entries {
            assert!(page.contains(u64::from(symbol.0)));
            assert!(recovered.insert(symbol, text).is_none());
        }
    }
    let expected_map: BTreeMap<Symbol, StoredSymbolText> = expected
        .iter()
        .map(|(id, text)| (Symbol(*id), StoredSymbolText::from((*text).to_owned())))
        .collect();
    assert_eq!(recovered, expected_map);

    let mut prior_bucket = None;
    let mut spellings = BTreeSet::new();
    for bucket in by_text {
        assert!(prior_bucket.is_none_or(|prev| prev < bucket.bucket));
        assert!(!bucket.pages.is_empty());
        assert_eq!(
            bucket.pages.iter().collect::<BTreeSet<_>>().len(),
            bucket.pages.len()
        );
        for &member in &bucket.pages {
            let snapshot = decode_symbol_member(member, members, prior_bytes);
            for (symbol, text) in snapshot.entries {
                assert_eq!(Sha256::digest(text.as_str().as_bytes())[0], bucket.bucket);
                assert_eq!(recovered.get(&symbol), Some(&text));
                assert!(spellings.insert(symbol));
            }
        }
        prior_bucket = Some(bucket.bucket);
    }
    assert_eq!(spellings.len(), expected_map.len());
}

pub(super) fn corrupt_prior_manifests() -> Vec<RelationalCheckpointManifest> {
    let loc_a = member_frame::locator(b"page-a");
    let loc_b = member_frame::locator(b"page-b");
    let valid_page = RelationalCheckpointPage::from_ids([1, 2], loc_a).unwrap();
    let valid_bucket = RelationalCheckpointSymbolBucket {
        bucket: 1,
        pages: vec![loc_a],
    };
    let mut forged = valid_page;
    forged.membership[0] |= 1 << 5;
    let overlap_page = RelationalCheckpointPage::from_ids([2, 3], loc_b).unwrap();
    vec![
        manifest_with_symbols(
            CheckpointPages::default(),
            CheckpointSymbolBuckets::default(),
            0,
        ),
        manifest_with_symbols(
            CheckpointPages::from_ordinary(vec![forged]),
            CheckpointSymbolBuckets::from_ordinary(vec![valid_bucket.clone()]),
            10,
        ),
        manifest_with_symbols(
            CheckpointPages::from_ordinary(vec![valid_page]),
            CheckpointSymbolBuckets::from_ordinary(vec![valid_bucket.clone()]),
            2,
        ),
        manifest_with_symbols(
            CheckpointPages::from_ordinary(vec![valid_page, overlap_page]),
            CheckpointSymbolBuckets::from_ordinary(vec![valid_bucket.clone()]),
            10,
        ),
        manifest_with_symbols(
            CheckpointPages::from_ordinary(vec![valid_page]),
            CheckpointSymbolBuckets::default(),
            10,
        ),
        manifest_with_symbols(
            CheckpointPages::from_ordinary(vec![valid_page]),
            CheckpointSymbolBuckets::from_ordinary(vec![RelationalCheckpointSymbolBucket {
                bucket: 1,
                pages: vec![loc_a, loc_a],
            }]),
            10,
        ),
        manifest_with_symbols(
            CheckpointPages::from_ordinary(vec![valid_page]),
            CheckpointSymbolBuckets::from_ordinary(vec![
                RelationalCheckpointSymbolBucket {
                    bucket: 5,
                    pages: vec![loc_a],
                },
                RelationalCheckpointSymbolBucket {
                    bucket: 2,
                    pages: vec![loc_b],
                },
            ]),
            10,
        ),
    ]
}

pub(super) fn verify_prior_symbol_carry_and_merge(
    capture_with: impl Fn(
        &StringInterner,
        Option<&RelationalCheckpointManifest>,
    ) -> Result<super::CheckpointMetadataMembers, super::Denial>,
) {
    let prior_interner = interner_with(None, &["alpha", "beta"]);
    let base = capture_with(&prior_interner, None).unwrap();
    let prior_bytes = prior_member_bytes(&base.members);
    let prior = manifest_with_symbols(base.symbols.clone(), base.symbol_buckets.clone(), 3);

    // 1. Cold prior with empty in-memory dictionary preserves all prior pages as Carried.
    let empty_interner = interner_with(Some(3), &[]);
    let cold = capture_with(&empty_interner, Some(&prior)).unwrap();
    assert_eq!(cold.symbols, base.symbols);
    assert_eq!(cold.symbol_buckets, base.symbol_buckets);
    assert!(cold.symbols.iter().all(|page| matches!(
        cold.members.get(&page.member),
        Some(RelationalCheckpointMember::Carried { .. })
    )));
    assert_valid_and_complete(
        &cold.symbols,
        &cold.symbol_buckets,
        &cold.members,
        &prior_bytes,
        &[(1, "alpha"), (2, "beta")],
    );

    let alpha_bucket = Sha256::digest(b"alpha")[0];
    let same_bucket_word = (0u32..)
        .map(|i| format!("same-bucket-{i}"))
        .find(|candidate| Sha256::digest(candidate.as_bytes())[0] == alpha_bucket)
        .expect("find word sharing alpha hash bucket");

    // 2. Cold prior + new symbols (same range 0, same hash bucket as alpha, and fresh range 1 id 260).
    let mut cold_plus_new_interner = interner_with(Some(3), &["delta", same_bucket_word.as_str()]);
    cold_plus_new_interner.reserve_frontier(260).unwrap();
    cold_plus_new_interner.intern("zeta").unwrap();
    let cold_plus_new = capture_with(&cold_plus_new_interner, Some(&prior)).unwrap();
    let merged_bucket = cold_plus_new
        .symbol_buckets
        .iter()
        .find(|bucket| bucket.bucket == alpha_bucket)
        .expect("merged alpha bucket present");
    assert!(
        merged_bucket.pages.len() >= 2,
        "new symbol in carried hash bucket must append a page without dropping carried pages"
    );
    assert_valid_and_complete(
        &cold_plus_new.symbols,
        &cold_plus_new.symbol_buckets,
        &cold_plus_new.members,
        &prior_bytes,
        &[
            (1, "alpha"),
            (2, "beta"),
            (3, "delta"),
            (4, same_bucket_word.as_str()),
            (260, "zeta"),
        ],
    );

    // 3. Warmed prior symbols + new symbols do not duplicate prior IDs or buckets.
    let mut warmed_plus_new_interner = prior_interner.clone();
    warmed_plus_new_interner.intern("delta").unwrap();
    warmed_plus_new_interner
        .intern(same_bucket_word.as_str())
        .unwrap();
    warmed_plus_new_interner.reserve_frontier(260).unwrap();
    warmed_plus_new_interner.intern("zeta").unwrap();
    let warmed_plus_new = capture_with(&warmed_plus_new_interner, Some(&prior)).unwrap();
    assert_valid_and_complete(
        &warmed_plus_new.symbols,
        &warmed_plus_new.symbol_buckets,
        &warmed_plus_new.members,
        &prior_bytes,
        &[
            (1, "alpha"),
            (2, "beta"),
            (3, "delta"),
            (4, same_bucket_word.as_str()),
            (260, "zeta"),
        ],
    );

    // 4. Corrupt or inconsistent prior directories fail closed.
    for bad in corrupt_prior_manifests() {
        assert!(matches!(
            capture_with(&empty_interner, Some(&bad)),
            Err(super::Denial::OriginalAuthorityUnavailable)
        ));
    }
}

fn decode_symbol_member(
    locator: RelationalDurableMemberLocator,
    members: &ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    prior_bytes: &BTreeMap<RelationalDurableMemberLocator, Vec<u8>>,
) -> SymbolTableSnapshot {
    let raw = match members.get(&locator).expect("member locator present") {
        RelationalCheckpointMember::New { bytes, .. } => bytes.bytes(),
        RelationalCheckpointMember::Carried { locator } => prior_bytes
            .get(locator)
            .expect("carried member exists in prior store")
            .as_slice(),
    };
    let payload = member_frame::decode(raw, locator, MemberKind::Symbols).unwrap();
    rmp_serde::from_slice(payload).unwrap()
}
