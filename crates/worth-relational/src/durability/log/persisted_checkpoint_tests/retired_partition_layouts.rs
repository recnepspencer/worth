//! Valid MessagePack/current framing must not shelter retired inner partition layouts.
use super::*;
use crate::durability::log::native_file_codec::{decode_checkpoint, CHECKPOINT_MAGIC};

pub(super) fn assert_retired_partition_layouts_refuse(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    checkpoint: &DurableCheckpoint,
) {
    let current = PersistedDurableCheckpointFile::from_checkpoint(checkpoint.clone());
    for version in [Some(0), Some(1), None] {
        let wire = rmp_serde::to_vec_named(&LayoutFile {
            checkpoint: &current.checkpoint,
            version,
        })
        .unwrap();
        let mut framed = CHECKPOINT_MAGIC.to_vec();
        framed.extend_from_slice(&wire);
        let before = framed.clone();
        let error = decode_checkpoint(execution, &framed).unwrap_err();
        let crate::durability::data::RelationalRecoveryStop::Durability(error) = error else {
            panic!("retired partition grammar must fail as typed format evidence")
        };
        assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
        assert!(
            error.detail.contains(if version.is_some() {
                "unsupported checkpoint partition alias format"
            } else {
                "partition_alias_format"
            }),
            "wrong refusal: {}",
            error.detail
        );
        assert_eq!(framed, before);
    }
    let mut framed = CHECKPOINT_MAGIC.to_vec();
    framed.extend(
        rmp_serde::to_vec_named(&LayoutFile {
            checkpoint: &current.checkpoint,
            version: Some(PARTITION_DELTA_FORMAT_VERSION),
        })
        .unwrap(),
    );
    let expected = current.readmit().unwrap();
    let current = decode_checkpoint(execution, &framed).unwrap();
    assert_eq!(current.checkpoint, expected.checkpoint);
    assert_eq!(current.checkpoint.branch_roots, checkpoint.branch_roots);
    assert_eq!(
        current.checkpoint.partition_images,
        checkpoint.partition_images
    );
}

// Fixture-local named-field encoding borrows the actual native DTO. Every payload
// retains its own Serialize implementation, including numeric/tuple map keys.
struct LayoutFile<'a> {
    checkpoint: &'a PersistedDurableCheckpoint,
    version: Option<u16>,
}
impl serde::Serialize for LayoutFile<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut file = serializer.serialize_struct("PersistedDurableCheckpointFile", 2)?;
        file.serialize_field("checkpoint_protocol_version", &6u16)?;
        file.serialize_field("checkpoint", &LayoutCheckpoint(self))?;
        file.end()
    }
}
struct LayoutCheckpoint<'a, 'b>(&'a LayoutFile<'b>);
impl serde::Serialize for LayoutCheckpoint<'_, '_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        // Exhaustive native destructure makes a new checkpoint field require
        // explicit fixture propagation instead of silently dropping its meaning.
        let PersistedDurableCheckpoint {
            coverage,
            branch_cells,
            branch_roots,
            branch_root_schema_images,
            record_identity,
            record_generation_high_water,
            reusable_record_slots,
            record_slot_frontiers,
            envelopes,
            partition_images,
            aspect_contracts,
            lineage,
            index_definitions,
            derived_index_artifacts,
            derived_index_checkpoint,
            derived_index_checkpoint_format,
            symbol_table,
            runtime_name,
            partition_alias_format: _,
            branch_root_partition_aliases_v2,
        } = self.0.checkpoint;
        let mut fields = serializer.serialize_struct(
            "PersistedDurableCheckpoint",
            19 + usize::from(self.0.version.is_some()),
        )?;
        macro_rules! payload_fields {
            ($($name:ident),* $(,)?) => {
                $(fields.serialize_field(stringify!($name), $name)?;)*
            };
        }
        payload_fields!(
            coverage,
            branch_cells,
            branch_roots,
            branch_root_schema_images,
            record_identity,
            record_generation_high_water,
            reusable_record_slots,
            record_slot_frontiers,
            envelopes,
            partition_images,
            aspect_contracts,
            lineage,
            index_definitions,
            derived_index_artifacts,
            derived_index_checkpoint,
            derived_index_checkpoint_format,
            symbol_table,
            runtime_name,
            branch_root_partition_aliases_v2,
        );
        if let Some(version) = self.0.version {
            fields.serialize_field("partition_alias_format", &version)?;
        }
        fields.end()
    }
}

#[test]
fn borrowed_native_checkpoint_refuses_v4_before_reading_its_body() {
    use crate::durability::log::native_file_codec::{decode_checkpoint, CHECKPOINT_MAGIC};
    #[derive(serde::Serialize)]
    struct PriorHeader {
        checkpoint_protocol_version: u16,
        checkpoint: (),
    }
    let mut bytes = CHECKPOINT_MAGIC.to_vec();
    bytes.extend(
        rmp_serde::to_vec_named(&PriorHeader {
            checkpoint_protocol_version: 4,
            checkpoint: (),
        })
        .unwrap(),
    );
    let request = crate::test_execution_policy::serial_request();
    let denied = decode_checkpoint(&worth_execution::ExecutionRequest::serial(&request), &bytes)
        .unwrap_err();
    let crate::durability::data::RelationalRecoveryStop::Durability(denied) = denied else {
        panic!("format refusal")
    };
    assert_eq!(denied.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(
        denied.detail.contains("v6 required first"),
        "{}",
        denied.detail
    );
}
