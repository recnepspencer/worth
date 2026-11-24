//! Two committed editions with the same entity and different summary values.
use crate::config::data::RelationalRuntimeConfig;
use crate::identity::data::{EntityId, KindId, PartitionId, VersionId};
use crate::runtime::state::subsystems::storage::{PartitionEditionCopyLane, StorageSubsystem};
use crate::schema::data::{
    DeclaredAspectContractBinding, EntityKindRegistration, KindAspectContractDeclarations,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::storage::overlay::{small_relation_partition::relation_partition, PartitionState};
use crate::storage::substrate::EntityExtra;
use worth_foundational::facade::*;
use worth_proof::TransitionOutcome;

pub(super) fn aspect_key(key: &str) -> AspectKey {
    AspectKey::new(key).unwrap()
}

pub(super) fn field_key(key: &str) -> FieldKey {
    FieldKey::new(key).unwrap()
}

pub(super) struct SummaryImage {
    pub(super) storage: StorageSubsystem,
    pub(super) config: RelationalRuntimeConfig,
    pub(super) entity: EntityId,
    pub(super) version: VersionId,
    partition: PartitionState,
    summary: AspectContract,
    name: AspectContract,
}

impl SummaryImage {
    pub(super) fn new() -> Self {
        let summary = AspectContract::struct_aspect(
            aspect_key("summary"),
            AspectIdentity(1),
            AspectContractRevision(1),
            aspects()
                .struct_fields()
                .required("title", ScalarAspectType::String)
                .optional("status", ScalarAspectType::String)
                .finish()
                .unwrap(),
        );
        let name = AspectContract::scalar(
            aspect_key("name"),
            AspectIdentity(2),
            AspectContractRevision(1),
            ScalarAspectType::String,
        );
        let mut config = RelationalRuntimeConfig::default();
        config.schema.registry = RelationalSchemaRegistry::new()
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(1),
                kind_name: "test.entity".into(),
                schema_id: SchemaId("test".into()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                    DeclaredAspectContractBinding {
                        binding: AspectBinding::EntityField {
                            field: field_key("name"),
                        },
                        contract: name.clone(),
                    },
                    DeclaredAspectContractBinding {
                        binding: AspectBinding::EntityField {
                            field: field_key("summary"),
                        },
                        contract: summary.clone(),
                    },
                ]),
            })
            .unwrap();
        let partition_id = PartitionId::main();
        let (partition, entities, _) = relation_partition(partition_id, 1, &[]);
        let mut image = Self {
            storage: StorageSubsystem::default(),
            config,
            entity: entities[0],
            version: VersionId(1),
            partition,
            summary,
            name,
        };
        image.install_title("before");
        image
    }

    pub(super) fn replace_title(&mut self, title: &str) {
        self.version = VersionId(2);
        self.install_title(title);
    }

    fn install_title(&mut self, title: &str) {
        let name = ContractValidationInput::Scalar(AspectValue::String("subject".into()));
        let summary = ContractValidationInput::Struct(
            StructAspectValue::new([
                (field_key("title"), AspectValue::String(title.into())),
                (field_key("status"), AspectValue::String("open".into())),
            ])
            .unwrap(),
        );
        let TransitionOutcome::Success(name) = validate_aspect_value(&self.name, name) else {
            panic!("fixture name satisfies its declared contract")
        };
        let TransitionOutcome::Success(summary) = validate_aspect_value(&self.summary, summary)
        else {
            panic!("fixture summary satisfies its declared contract")
        };
        let TransitionOutcome::Success(patch) =
            AuthoritativeRecordAspectPatch::whole_aspect([name, summary], [])
        else {
            panic!("fixture uses distinct declared aspects")
        };
        let TransitionOutcome::Success(state) = patch.apply_to_optional(None) else {
            panic!("fixture owns an admissible record state")
        };
        let extra = EntityExtra::from_unbudgeted_parts(
            None,
            None,
            state.map(|state| state.into_parts().into_parts().0),
        );
        self.partition
            .entity_arena
            .apply_extra_update(self.entity.slot_index(), extra, self.version)
            .expect("the fixture replaces the exact selected slot");
        self.storage.install_owned(
            &Default::default(),
            PartitionEditionCopyLane::Ordinary,
            [(self.entity.partition_id, self.partition.clone())],
        );
    }

    pub(super) fn assert_summary(&self, title: &str, status: &str) {
        let partition = self.storage.partition(self.entity.partition_id).unwrap();
        let record = partition.entity_arena.get(&self.entity).unwrap();
        let summary = record
            .extra()
            .authoritative_aspect_state()
            .and_then(|state| state.get(&aspect_key("summary")))
            .expect("summary aspect remains present");
        let ContractValidatedAspectValueView::Struct(summary) = summary.view() else {
            panic!("summary aspect changed shape after a committed replacement");
        };
        assert_eq!(
            summary.get(&field_key("title")),
            Some(&AspectValue::String(InternedString::Raw(title.to_owned())))
        );
        assert_eq!(
            summary.get(&field_key("status")),
            Some(&AspectValue::String(InternedString::Raw(status.to_owned())))
        );
    }
}
