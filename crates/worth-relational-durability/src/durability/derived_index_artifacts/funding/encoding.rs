//! Two-pass named encoding into the index owner's funded fixed byte backing.
use worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::checkpoint_image::IndexArray;
use super::IndexCheckpointWork;
use worth_relational_contracts::internal_port::durability::data::recovery_stop::RelationalRecoveryStop as Stop;
use crate::durability::artifact_payload_emission::request::{extend_length, named_to};

impl IndexCheckpointWork<'_> {
    pub(in super::super) fn encode_named(
        &self,
        value: &impl serde::Serialize,
    ) -> Result<IndexArray<u8>, Stop> {
        let mut length = 0;
        named_to(
            value,
            |bytes| {
                let next = extend_length(length, bytes)
                    .ok_or_else(super::super::decode_budget::overflow)?;
                for _ in bytes.chunks(64 * 1024) {
                    self.consult()?;
                }
                length = next;
                Ok(())
            },
            encoding_failure,
        )?;
        let mut output = self.array::<u8>(length)?;
        named_to(
            value,
            |bytes| output.extend_from_slice(bytes),
            encoding_failure,
        )?;
        output.seal()
    }
}

fn encoding_failure(_: rmp_serde::encode::Error) -> Stop {
    super::super::corrupt("index payload serializer refused its producer DATA").into()
}
