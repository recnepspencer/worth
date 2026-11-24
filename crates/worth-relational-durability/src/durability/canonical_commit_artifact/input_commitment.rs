//! Result-image-first v8 input commitments reuse one funded SHA state per carrier.
use crate::durability::data::DurableBranchRootSchemaImage;
use contracts::config::data::policies::MutationConfig;
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use sha2::{Digest, Sha256};
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::history::commit::payload_commitment::writer::{
    in_context, CommitmentWriter,
};
use worth_relational_contracts::internal_port as contracts;

#[derive(Clone)]
pub(crate) struct CanonicalInputDigestPrefix(Sha256);
impl CanonicalInputDigestPrefix {
    pub(crate) fn capture(
        image: Option<&DurableBranchRootSchemaImage>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalRecoveryStop> {
        in_context(request, |context| {
            let mut writer = CommitmentWriter::new(Sha256::new(), context);
            writer.bytes(&[0x84])?;
            writer.value(&"result_image")?;
            writer.value(&image)?;
            Ok(Self(writer.hash))
        })
    }
    pub(crate) fn finish(
        &self,
        prestate: [u8; 32],
        result: [u8; 32],
        mutation: &MutationConfig,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<[u8; 32], RelationalRecoveryStop> {
        in_context(request, |context| {
            let mut writer = CommitmentWriter::new(self.0.clone(), context);
            writer.value(&"prestate_schema")?;
            writer.value(&prestate)?;
            writer.value(&"result_schema")?;
            writer.value(&result)?;
            writer.value(&"mutation")?;
            writer.value(mutation)?;
            Ok(writer.hash.finalize().into())
        })
    }
}

pub(crate) fn position_leaf(
    position: contracts::publication::patch::data::patch_position::PatchStreamPosition,
    request: &ExecutionRequest<'_, '_>,
) -> Result<[u8; 32], RelationalRecoveryStop> {
    in_context(request, |context| {
        let mut writer = CommitmentWriter::new(Sha256::new(), context);
        writer.value(&position)?;
        Ok(writer.hash.finalize().into())
    })
}
