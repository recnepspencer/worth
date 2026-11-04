//! Independent legacy wire twins and actual retained owner lifetime.
use super::*;
use crate::durability::{
    data::*,
    log::native_checkpoint_codec::member_frame::{self, MemberKind},
};
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::identity::data::{EntityId, LineageId, PartitionId, VersionId};
use crate::indexes::data::*;
use crate::lineage::data::{
    LineageCheckpointDigestBasis, LineageEventKind, LineageEventRecord, LineageNode,
};
use crate::schema::data::SchemaVersionId;
use crate::symbols::data::{StoredSymbolText, Symbol, SymbolTableSnapshot};
use std::{alloc::Layout, sync::Arc};
use worth_execution::{
    CancellationSource, ExecutionMemoryReservation, ExecutionRequest, SerialMemoryBudget,
    SerialRequest,
};
fn same_wire(left: &impl serde::Serialize, right: &impl serde::Serialize) {
    assert_eq!(
        rmp_serde::to_vec_named(left).unwrap(),
        rmp_serde::to_vec_named(right).unwrap()
    );
    assert_eq!(
        rmp_serde::to_vec(left).unwrap(),
        rmp_serde::to_vec(right).unwrap()
    );
    assert_eq!(
        serde_json::to_vec(left).unwrap(),
        serde_json::to_vec(right).unwrap()
    );
}
fn controller() -> u64 {
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let mut observed = None;
    ExecutionRequest::serial(&host)
        .run_native_with_checkpoint(|_| observed = Some(budget.charged_memory_bytes()))
        .unwrap();
    assert_eq!(budget.charged_memory_bytes(), 0);
    observed.unwrap()
}
#[test]
fn borrowed_dictionary_and_full_cache_keep_actual_old_wire_and_members() {
    let entries = vec![
        (Symbol(1), StoredSymbolText::from("first".to_owned())),
        (
            Symbol(513),
            StoredSymbolText::from("unicode-\u{03bb}".to_owned()),
        ),
    ];
    let old = SymbolTableSnapshot {
        entries: entries.clone(),
    };
    same_wire(
        &dictionary::view(entries.len(), || {
            entries.iter().map(|(symbol, text)| (*symbol, text))
        }),
        &old,
    );
    let branch = BranchId("main".to_owned());
    let generation = DerivedIndexGeneration {
        generation_id: DerivedIndexGenerationId(7),
        index_id: DerivedIndexId(2),
        source_commit_id: CommitId(11),
        source_branch_id: branch.clone(),
        applicability: DerivedIndexApplicability {
            branch_id: branch,
            version_id: VersionId(13),
            schema_version: SchemaVersionId(3),
        },
        status: DerivedIndexPublicationStatus::Published,
        entries: DerivedIndexEntries::EntityField(Default::default()),
    };
    let generations = [Arc::new(generation.clone())];
    let legacy = (
        DerivedIndexArtifacts::new(vec![generation]),
        None::<crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts>,
        0_u16,
    );
    same_wire(&indexes::view(&generations), &legacy);
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let dictionary = encode_dictionary(
        entries.len(),
        || entries.iter().map(|(symbol, text)| (*symbol, text)),
        &request,
    )
    .unwrap();
    let old_dictionary = member_frame::encode(MemberKind::Symbols, &old).unwrap();
    assert_eq!(dictionary.0, old_dictionary.0);
    assert_eq!(dictionary.1.bytes(), old_dictionary.1);
    // This is the pure wire view; Complete emission requires an actual resolved owner.
    let cache = member_frame::encode_for_request(
        MemberKind::DerivedIndex,
        &indexes::view(&generations),
        &request,
    )
    .unwrap();
    assert_eq!(
        cache.1.bytes(),
        member_frame::encode(MemberKind::DerivedIndex, &legacy)
            .unwrap()
            .1
    );
    drop((dictionary, cache));
    assert_eq!(budget.charged_memory_bytes(), 0);
}
#[test]
fn borrowed_lineage_rows_and_funded_routes_keep_original_wire() {
    let commit = RelationalCommitReceipt {
        commit_id: CommitId(4),
        version_id: VersionId(8),
        branch_id: BranchId("branch".to_owned()),
        parents: vec![CommitId(1)],
    };
    let event = LineageEventRecord {
        event_id: 7,
        commit: commit.clone(),
        branch_id: commit.branch_id.clone(),
        kind: LineageEventKind::Create,
        sources: Vec::new(),
        targets: vec![LineageId(513)],
    };
    let borrowed = [OriginalLineageEvent {
        event: &event,
        publication_commit_id: CommitId(4),
    }];
    let old = [RelationalCheckpointLineageEventRow {
        event: Arc::new(event.clone()),
        publication_commit_id: CommitId(4),
    }];
    same_wire(&borrowed, &old);
    let nodes = [LineageNode {
        lineage_id: LineageId(513),
        entity_id: EntityId::new(PartitionId::main(), 2, 1),
    }];
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let encoded = encode_lineage_events(&borrowed, &request).unwrap();
    assert_eq!(
        encoded.1.bytes(),
        member_frame::encode(MemberKind::LineageEvents, &old)
            .unwrap()
            .1
    );
    let mut directory = CheckpointLineageEventPages::allocate_for_request(1, &request).unwrap();
    directory
        .push(encoded.0, borrowed.iter().map(|row| row.event), &request)
        .unwrap();
    let directory = directory.seal();
    let legacy = RelationalCheckpointLineageEventPage::from_rows(encoded.0, &old).unwrap();
    assert_eq!(&directory[..], std::slice::from_ref(&legacy));
    same_wire(&directory, &vec![legacy]);
    let nodes_encoded = encode_lineage(&nodes, &request).unwrap();
    assert_eq!(
        nodes_encoded.1.bytes(),
        member_frame::encode(MemberKind::Lineage, &nodes).unwrap().1
    );
    let alias = directory.clone();
    drop(directory);
    drop((encoded, nodes_encoded));
    assert!(budget.charged_memory_bytes() > 0);
    drop(alias);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
#[test]
fn lineage_basis_aliases_hold_the_exact_original_backing_until_last_drop() {
    struct IndependentBacking {
        _basis: LineageCheckpointDigestBasis,
        _memory: Option<ExecutionMemoryReservation>,
    }
    let count = 1024usize;
    let bill = (Layout::new::<[usize; 2]>()
        .extend(Layout::new::<IndependentBacking>())
        .unwrap()
        .0
        .pad_to_align()
        .size()
        + Layout::array::<u64>(count).unwrap().size()) as u64;
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let basis = CheckpointLineageBasis::from_original(
        3,
        count,
        5,
        count,
        (1..=count).map(|n| n as u64),
        &ExecutionRequest::serial(&host),
    )
    .unwrap();
    let legacy =
        LineageCheckpointDigestBasis::new(3, (1..=count).map(|n| n as u64).collect(), count, 5);
    same_wire(&basis, &legacy);
    assert_eq!(budget.charged_memory_bytes(), bill);
    let alias = basis.clone();
    drop(basis);
    assert_eq!(budget.charged_memory_bytes(), bill);
    assert_eq!(alias.ordinary_owned_copy(), legacy);
    drop(alias);
    assert_eq!(budget.charged_memory_bytes(), 0);
    let budget = SerialMemoryBudget::new(Some(controller() + bill - 1));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    assert!(
        matches!(CheckpointLineageBasis::from_original(3,count,5,count,(1..=count).map(|n|n as u64),&ExecutionRequest::serial(&host)),
        Err(crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {requested,admitted,ancestor:0}) if requested==bill && admitted==bill-1)
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
    cancellation.cancel();
    assert!(matches!(
        CheckpointLineageBasis::from_original(
            0,
            0,
            0,
            0,
            std::iter::empty(),
            &ExecutionRequest::serial(&host)
        ),
        Err(crate::execution::RelationalExecutionDenialCause::Cancelled)
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn metadata_member_directories_preserve_dictionary_ranges_and_alias_custody() {
    use super::super::symbols::OriginalSymbolDictionary;
    use crate::symbols::data::StringInterner;
    use sha2::{Digest, Sha256};
    let mut original = StringInterner::default();
    for ordinal in 0..260 {
        original.intern(&format!("word-{ordinal}")).unwrap();
    }
    let budget = SerialMemoryBudget::new(Some(8 * 1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let dictionary = OriginalSymbolDictionary::capture(&original, &[], &request).unwrap();
    let lineage = capture_lineage(&[], &request).unwrap();
    let original = crate::runtime::IndexingSubsystem::default()
        .checkpoint_inputs_for_request(&request)
        .unwrap()
        .resolve_for_request(&request)
        .unwrap();
    let output = capture_metadata_members(&dictionary, &lineage, &original, &request).unwrap();
    drop(original);
    let mut recovered = std::collections::BTreeMap::new();
    for page in &output.symbols {
        assert!(page.valid());
        let RelationalCheckpointMember::New { bytes, .. } =
            output.members.get(&page.member).unwrap()
        else {
            panic!("actual encoded member required")
        };
        let payload =
            member_frame::decode(bytes.bytes(), page.member, MemberKind::Symbols).unwrap();
        let snapshot: SymbolTableSnapshot = rmp_serde::from_slice(payload).unwrap();
        for (symbol, text) in snapshot.entries {
            assert!(page.contains(u64::from(symbol.0)));
            assert!(recovered.insert(symbol, text).is_none());
        }
    }
    assert_eq!(recovered.len(), dictionary.len());
    for (symbol, text) in dictionary.entries() {
        assert_eq!(recovered.get(&symbol), Some(text));
    }
    let mut spellings = std::collections::BTreeSet::new();
    for bucket in &output.symbol_buckets {
        for member in &bucket.pages {
            let RelationalCheckpointMember::New { bytes, .. } = output.members.get(member).unwrap()
            else {
                panic!("actual spelling member required")
            };
            let payload =
                member_frame::decode(bytes.bytes(), *member, MemberKind::Symbols).unwrap();
            let snapshot: SymbolTableSnapshot = rmp_serde::from_slice(payload).unwrap();
            for (symbol, text) in snapshot.entries {
                assert_eq!(Sha256::digest(text.as_str().as_bytes())[0], bucket.bucket);
                assert_eq!(recovered.get(&symbol), Some(&text));
                assert!(spellings.insert(symbol));
            }
        }
    }
    assert_eq!(spellings.len(), dictionary.len());
    let aliases = (output.symbols.clone(), output.symbol_buckets.clone());
    drop((output, lineage, dictionary));
    assert!(budget.charged_memory_bytes() > 0);
    drop(aliases);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
