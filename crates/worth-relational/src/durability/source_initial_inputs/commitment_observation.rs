//! Passive observation of actual artifact roles through the production hash catalog.
use super::{
    input_commitments::OriginalInputCommitments, RelationalCompleteSourceArtifact,
    RelationalInitialInputsRefusal as Refusal,
};
use crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload;
use worth_execution::{ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder};

impl RelationalCompleteSourceArtifact {
    /// Read only this artifact's actual schema catalog and normalized rows.
    /// The result is computed DATA, never source admission or a cache handle.
    pub fn input_commitment_leaves_for_test(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        ExecutionArray<(
            crate::publication::patch::data::PatchStreamPosition,
            [u8; 32],
        )>,
        Refusal,
    > {
        let checkpoint = self.checkpoint()?;
        let raw = checkpoint
            .bytes()
            .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
            .ok_or(Refusal::Malformed)?;
        let schema_bytes = BorrowedPayload(raw)
            .field("manifest", request)
            .map_err(Refusal::Payload)?
            .field("schemas", request)
            .map_err(Refusal::Payload)?;
        let count = schema_bytes
            .elements(request, |_| Ok(()))
            .map_err(Refusal::Payload)?;
        let mut images = ExecutionRequestArrayBuilder::allocate(*request, count).map_err(array)?;
        let mut stopped = None;
        schema_bytes
            .elements(request, |bytes| {
                let admission =
                    crate::durability::canonical_commit_artifact::prepare_schema_image_decode(
                        bytes, request,
                    )?;
                let mut image = rmp_serde::from_slice(bytes.0).map_err(|_| malformed())?;
                admission.attach(&mut image)?;
                if let Err(cause) = images.push(image) {
                    stopped = Some(array(cause));
                    return Err(malformed());
                }
                Ok(())
            })
            .map_err(|cause| stopped.unwrap_or(Refusal::Payload(cause)))?;
        let images = images.seal().map_err(array)?;
        let normalized = super::normalized_admission::readmit(
            self.normalized_bytes(),
            self.covered().0,
            self.covered_transitions().0,
            request,
        )?;
        let catalog = OriginalInputCommitments::capture(images.iter(), request)?;
        let mut leaves =
            ExecutionRequestArrayBuilder::allocate(*request, normalized.canonical.len())
                .map_err(array)?;
        for row in &normalized.canonical {
            leaves
                .push((
                    row.position,
                    catalog.input_leaf(
                        row.prestate_schema,
                        row.result_schema,
                        &row.mutation,
                        request,
                    )?,
                ))
                .map_err(array)?;
        }
        leaves.seal().map_err(array)
    }
}
fn array(cause: worth_execution::ExecutionRequestArrayDenial) -> Refusal {
    Refusal::Native(
        crate::durability::checkpoint_history::capture::array_capture_stop(cause).into(),
    )
}
fn malformed(
) -> crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal {
    crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::PayloadDecode {
        detail: "Complete artifact schema catalog is malformed".into(),
    }
}
