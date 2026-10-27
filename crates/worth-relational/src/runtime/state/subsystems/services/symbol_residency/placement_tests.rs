//! Dictionary placement survives checkpoint replacement without warming payloads.
use super::*;
use crate::durability::medium::RelationalDurableMemberCustody;
use std::collections::BTreeSet;

#[derive(Debug)]
struct PruningSource {
    bytes: Mutex<BTreeMap<RelationalDurableMemberLocator, Arc<[u8]>>>,
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
    let (_, mut directory, initial, all) = checkpoint_dictionary(&runtime);
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
    owner.install_cold_pages(&directory).unwrap();
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
    let (_, replacement, next, _) = checkpoint_dictionary(&runtime);
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
    let denial = table.install_cold_pages(&directory).unwrap_err();
    let crate::durability::data::RelationalRecoveryStop::Durability(error) =
        denial.into_recovery_stop()
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
    table.install_cold_pages(&directory).unwrap();
    assert!(table.has_cold_pages());
    assert_eq!(source.reads.load(Ordering::SeqCst), 0);
}
