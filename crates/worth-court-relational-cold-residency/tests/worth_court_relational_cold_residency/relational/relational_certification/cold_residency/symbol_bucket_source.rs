//! Original 600-key population and selected real collision bucket in independent receivers.
use super::{
    checkpoint_receiver::reopen, current_field_meanings::name_field_patch,
    current_publications::create_entity,
};
use sha2::{Digest, Sha256};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::RelationalDurableMemberLocator, identity::*, symbols::ClientKey, transactions::*,
};
use worth_testkit_relational::{
    aspect_storage::publications::{
        begin_main_transaction, changed_entities, release_test_commit_snapshot,
    },
    cold_recovery::{capture_source, ColdRecoverySourceFixture},
    durable_medium::TestDurableMedium,
    durable_publication::durable_runtime,
};
pub(super) struct BucketRecipe {
    original: ColdRecoverySourceFixture,
    pub endpoints: [EntityId; 2],
    pub label_bucket: RelationalDurableMemberLocator,
    pub cold_key: String,
}
pub(super) struct BucketFixture{pub runtime:worth_relational::facade::runtime::RelationalRuntime,pub source:std::sync::Arc<worth_testkit_relational::durable_medium::immutable_members::ImmutableCheckpointMemberSource>,pub medium:std::sync::Arc<TestDurableMedium>,pub endpoints:[EntityId;2],pub label_bucket:RelationalDurableMemberLocator,pub cold_key:String}
impl BucketRecipe {
    pub fn new(request: &ExecutionRequest<'_, '_>) -> Self {
        let medium = TestDurableMedium::new();
        let producer = durable_runtime(&medium);
        let bucket = Sha256::digest(b"label")[0];
        let cold_key = (0..65_536)
            .map(|id| format!("cold-label-bucket-{id}"))
            .find(|text| Sha256::digest(text.as_bytes())[0] == bucket)
            .unwrap();
        let mut keys = (0..600)
            .map(|id| ClientKey::raw(format!("symbol-seed-{id:04}")))
            .collect::<Vec<_>>();
        keys.push(ClientKey::raw(cold_key.clone()));
        let mut txn = begin_main_transaction(&producer);
        txn.push_batch(
            WorkerIntentBatch::new("cold-symbol-population").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    field_patches: keys.iter().map(|_| name_field_patch("seed")).collect(),
                    client_keys: keys,
                }),
            )),
            request.allocation_policy(),
        )
        .unwrap();
        let committed = txn.commit(request, &producer).unwrap();
        let entities = changed_entities(&committed);
        let endpoints = [entities[0], entities[1]];
        release_test_commit_snapshot(&producer, &committed);
        let entity = create_entity(request, &producer, "member-before");
        let original = capture_source(request, producer, medium, entity, false);
        let directory = super::selected_directory::read(&original.input);
        let label_bucket = directory
            .symbol_buckets
            .iter()
            .find(|entry| entry.bucket == bucket)
            .unwrap()
            .pages[0];
        assert!(
            directory.symbols.len() > 1,
            "fixture spans dictionary ID pages"
        );
        Self {
            original,
            endpoints,
            label_bucket,
            cold_key,
        }
    }
    pub fn reopen(&self, request: &ExecutionRequest<'_, '_>) -> BucketFixture {
        {
            let reader = reopen(request, &self.original);
            BucketFixture {
                runtime: reader.runtime,
                source: reader.source,
                medium: reader.medium,
                endpoints: self.endpoints,
                label_bucket: self.label_bucket,
                cold_key: self.cold_key.clone(),
            }
        }
    }
}
