//! Dictionary placement survives checkpoint replacement without warming payloads.
use super::*;
use crate::durability::medium::RelationalDurableColdSource;
use crate::durability::medium::RelationalDurableMemberCustody;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::storage::data::RelationalStorageSubject;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::Mutex;
use worth_relational_branch::internal_port::mvcc::observation::RelationalBranchObservationOwnerPort as _;

#[derive(Debug)]
struct PruningSource {
    bytes:
        Mutex<BTreeMap<RelationalDurableMemberLocator, worth_execution::ExecutionImmutableBytes>>,
    retained: Arc<Mutex<BTreeMap<RelationalDurableMemberLocator, usize>>>,
    refusal: Mutex<Option<Cause>>,
    reads: AtomicUsize,
}
struct PlacementOwner {
    members: Vec<RelationalDurableMemberLocator>,
    retained: Arc<Mutex<BTreeMap<RelationalDurableMemberLocator, usize>>>,
}
impl Drop for PlacementOwner {
    fn drop(&mut self) {
        let mut retained = self.retained.lock().unwrap();
        for member in &self.members {
            let count = retained.get_mut(member).unwrap();
            *count -= 1;
            if *count == 0 {
                retained.remove(member);
            }
        }
    }
}
impl PruningSource {
    fn replace(&self, next: &PageSource) {
        let retained = self.retained.lock().unwrap();
        let mut bytes = self.bytes.lock().unwrap();
        bytes.retain(|member, _| retained.contains_key(member));
        bytes.extend(next.members.clone());
    }
}
impl RelationalDurableColdSource for PruningSource {
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        if let Some(cause) = *self.refusal.lock().unwrap() {
            return Err(cause);
        }
        let mut retained = self.retained.lock().unwrap();
        let bytes = self.bytes.lock().unwrap();
        if members.iter().any(|member| !bytes.contains_key(member)) {
            return Err(Cause::MissingMember);
        }
        for member in members {
            *retained.entry(*member).or_default() += 1;
        }
        Ok(RelationalDurableMemberCustody::new(PlacementOwner {
            members: members.to_vec(),
            retained: self.retained.clone(),
        }))
    }
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        if locator.length > limit {
            return Err(Cause::LimitExceeded {
                length: locator.length,
                limit,
            });
        }
        self.bytes
            .lock()
            .unwrap()
            .get(&locator)
            .map(|bytes| bytes.to_vec())
            .ok_or(Cause::MissingMember)
    }
    fn read_commit(&self, _: PatchStreamPosition, _: u64) -> Result<Vec<u8>, Cause> {
        Err(Cause::SourceUnavailable)
    }
}

#[test]
fn detached_dictionary_keeps_replaced_id_and_bucket_pages_until_its_last_owner_drops() {
    use sha2::{Digest, Sha256};
    let backing = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&backing);
    let runtime = runtime_with_test_schema();
    create_entity(&execution, &runtime, "dictionary-custody-seed");
    let (_, mut directory, initial, all) = checkpoint_dictionary(&execution, &runtime);
    let (symbol, text) = all.entries.iter().max_by_key(|(id, _)| *id).unwrap();
    let bucket = Sha256::digest(text.as_str().as_bytes())[0];
    let old_id_page = directory.manifest.symbols.last().unwrap().member;
    let old_bucket = directory
        .manifest
        .symbol_buckets
        .iter()
        .find(|page| page.bucket == bucket)
        .unwrap()
        .pages
        .clone();
    let source = Arc::new(PruningSource {
        bytes: Mutex::new(initial.members.clone()),
        retained: Arc::default(),
        refusal: Mutex::new(None),
        reads: AtomicUsize::new(0),
    });
    directory.source = source.clone();
    let owner = RuntimeSymbolTable::default();
    owner
        .install_cold_symbol_pages(
            directory.manifest.symbol_page_roster(),
            directory.source.clone(),
        )
        .unwrap();
    let by_id = owner.detached_owner_snapshot();
    let by_value = owner.detached_owner_snapshot();
    drop(owner);
    let new_text = (0..)
        .map(|index| format!("dictionary-custody-added-{index}"))
        .find(|value| Sha256::digest(value.as_bytes())[0] == bucket)
        .unwrap();
    let input = worth_execution::ExecutionImmutableBytes::from_external_bytes(Arc::new(
        new_text.into_bytes().into_boxed_slice(),
    ));
    let transaction = test_owner_begin_transaction_for_main(&runtime);
    runtime
        .admit_field_text(&execution, &transaction.basis, &input)
        .unwrap();
    let (_, replacement, next, _) = checkpoint_dictionary(&execution, &runtime);
    let selected = replacement
        .manifest
        .symbols
        .iter()
        .map(|page| page.member)
        .chain(
            replacement
                .manifest
                .symbol_buckets
                .iter()
                .flat_map(|page| page.pages.iter().copied()),
        )
        .collect::<BTreeSet<_>>();
    assert!(
        !selected.contains(&old_id_page),
        "the real producer must replace the tail ID page"
    );
    assert!(
        old_bucket.iter().any(|page| !selected.contains(page)),
        "the real producer must replace this bucket"
    );
    source.replace(&next);
    assert_eq!(
        source.reads.load(Ordering::SeqCst),
        0,
        "placement custody does not load payloads"
    );
    assert!(by_id.snapshot().entries.is_empty() && by_value.snapshot().entries.is_empty());
    by_id.demand_symbol_ids(&execution, [*symbol]).unwrap();
    by_value.demand_text(&execution, text.as_str()).unwrap();
    assert_eq!(
        by_id.with_read(|table| table.symbol(text.as_str())),
        Some(*symbol)
    );
    assert_eq!(
        by_value.with_read(|table| table.symbol(text.as_str())),
        Some(*symbol)
    );
    assert!(source.reads.load(Ordering::SeqCst) < 5);
    drop(by_id);
    source.replace(&next);
    assert!(
        source.bytes.lock().unwrap().contains_key(&old_id_page),
        "the detached bucket owner still retains its directory"
    );
    drop(by_value);
    source.replace(&next);
    assert!(!source.bytes.lock().unwrap().contains_key(&old_id_page));
    assert!(source.retained.lock().unwrap().is_empty());
}

#[test]
fn dictionary_placement_refusal_is_typed_and_retryable_before_frontier_mutation() {
    let (_, mut directory, initial, _) = dictionary();
    let source = Arc::new(PruningSource {
        bytes: Mutex::new(initial.members.clone()),
        retained: Arc::default(),
        refusal: Mutex::new(Some(Cause::SourceUnavailable)),
        reads: AtomicUsize::new(0),
    });
    directory.source = source.clone();
    let table = RuntimeSymbolTable::default();
    let before = table.interner_snapshot();
    let denial = table
        .install_cold_symbol_pages(
            directory.manifest.symbol_page_roster(),
            directory.source.clone(),
        )
        .unwrap_err();
    let crate::durability::data::RelationalRecoveryStop::Durability(error) =
        crate::durability::data::RelationalRecoveryStop::from(denial)
    else {
        panic!("placement refusal must preserve durable failure classification");
    };
    assert_eq!(
        error.class,
        crate::durability::data::RecoveryFailureClass::ColdStorageUnavailable
    );
    let failure = error.storage_unavailable.unwrap();
    assert!(matches!(
        failure.subject,
        RelationalStorageSubject::Symbols { .. }
    ));
    assert_eq!(failure.cause, Cause::SourceUnavailable);
    assert_eq!(table.interner_snapshot(), before);
    assert!(!table.has_cold_pages());
    *source.refusal.lock().unwrap() = None;
    table
        .install_cold_symbol_pages(
            directory.manifest.symbol_page_roster(),
            directory.source.clone(),
        )
        .unwrap();
    assert!(table.has_cold_pages());
    assert_eq!(source.reads.load(Ordering::SeqCst), 0);
}

// Fixture adaptation (C2.4): restores 427700edd0:cold_metadata_pins.rs's
// private cached-Head / absent-per-runtime-dictionary `SymbolsNotResident`
// window over ExternalMembers recovery into a Memory runtime and its fork.

#[test]
fn metadata_pin_refuses_shared_head_until_its_runtime_has_the_aspect_meanings() {
    use crate::durability::data::{
        RecoveryVerificationMode, RelationalCheckpointArtifactPolicy, RelationalCheckpointMember,
    };
    use crate::durability::published_medium::{
        RelationalDurableCheckpointSource, RelationalDurableRecoverySource,
    };

    use crate::tests::support::{
        create_entity, runtime_with_test_schema, snapshot_for_owner_branch,
        update_entity_and_release_snapshot,
    };
    use crate::visibility::materialization::read_records::RelationalEntityMetadataPinDenial as Denial;
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let name_aspect = worth_foundational::facade::AspectKey::new("name").unwrap();
    let producer = runtime_with_test_schema();
    let entity = create_entity(execution, &producer, "shared-head-meta");
    update_entity_and_release_snapshot(execution, &producer, entity, "shared-head-meta-updated");
    let native = producer
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let members = native
        .members()
        .iter()
        .map(|member| match member {
            RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.clone()),
            RelationalCheckpointMember::Carried { .. } => unreachable!(),
        })
        .collect();
    let source = Arc::new(super::tests::PageSource::with_members(members));
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            native.covered_position().unwrap(),
            native.covered_branch_transitions().unwrap(),
            native,
        )),
        Vec::new(),
        Vec::new(),
        source.clone(),
    );
    let mut runtime = runtime_with_test_schema();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let fork = runtime.fork().unwrap();
    let orig_snap =
        snapshot_for_owner_branch(&runtime, &crate::history::data::BranchId("main".into()));
    let _ = runtime
        .read_truth()
        .read_exact_snapshot_entity_aspect_version(execution, &orig_snap, entity, &name_aspect)
        .unwrap();
    runtime.snapshots().release_snapshot(&orig_snap).unwrap();
    let snapshot = snapshot_for_owner_branch(&fork, &crate::history::data::BranchId("main".into()));
    let (_, basis) = fork.observe_branch(&fork.main_branch_identity()).unwrap();
    let reads = source.member_reads();
    basis
        .observation()
        .selected_root()
        .read()
        .demand_region_head_cached(execution, entity.partition_id)
        .unwrap();
    assert_eq!(source.member_reads(), reads);
    assert!(matches!(
        fork.read_truth().pin_resident_entity_metadata(&snapshot, entity),
        Err(Denial::SymbolsNotResident(partition)) if partition == entity.partition_id
    ));
    let original =
        snapshot_for_owner_branch(&runtime, &crate::history::data::BranchId("main".into()));
    let expected = runtime
        .read_truth()
        .read_exact_snapshot_entity_aspect_version(execution, &original, entity, &name_aspect)
        .unwrap();
    runtime.snapshots().release_snapshot(&original).unwrap();
    assert_eq!(
        fork.read_truth()
            .read_exact_snapshot_entity_aspect_version(execution, &snapshot, entity, &name_aspect)
            .unwrap(),
        expected
    );
    let metadata = fork
        .read_truth()
        .pin_resident_entity_metadata(&snapshot, entity)
        .unwrap()
        .unwrap();
    assert_eq!(metadata.live_kind(), Some(crate::identity::data::KindId(1)));
    assert_eq!(metadata.aspect_version(&name_aspect), expected);
    fork.snapshots().release_snapshot(&snapshot).unwrap();
}
