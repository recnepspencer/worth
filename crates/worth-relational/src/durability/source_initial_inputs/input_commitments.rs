//! One admitted result-image hash prefix per distinct selected schema carrier.
use super::{normalized::CanonicalOriginalRecord, RelationalInitialInputsRefusal as Refusal};
use crate::config::data::MutationConfig;
use crate::durability::canonical_commit_artifact::input_commitment::{
    position_leaf, CanonicalInputDigestPrefix,
};
use crate::durability::data::DurableBranchRootSchemaImage;
use crate::history::RelationalCommitIndexEntry;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};

pub(super) struct OriginalInputCommitments {
    schemas: ExecutionOrderedMap<[u8; 32], DurableBranchRootSchemaImage>,
    prefixes: ExecutionOrderedMap<Option<[u8; 32]>, CanonicalInputDigestPrefix>,
}
impl OriginalInputCommitments {
    pub(super) fn capture<'a>(
        images: impl IntoIterator<Item = &'a DurableBranchRootSchemaImage>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let mut captured = Self {
            schemas: Default::default(),
            prefixes: Default::default(),
        };
        let nil = CanonicalInputDigestPrefix::capture(None, request).map_err(Refusal::Native)?;
        captured
            .prefixes
            .insert_cloned_native_for_request(&None, &nil, *request)
            .map_err(map)?;
        for image in images {
            if captured.schemas.get(&image.carrier_digest()).is_some() {
                return Err(Refusal::Malformed);
            }
            captured
                .schemas
                .insert_cloned_native_for_request(&image.carrier_digest(), image, *request)
                .map_err(map)?;
            let prefix = CanonicalInputDigestPrefix::capture(Some(image), request)
                .map_err(Refusal::Native)?;
            captured
                .prefixes
                .insert_cloned_native_for_request(&Some(image.carrier_digest()), &prefix, *request)
                .map_err(map)?;
        }
        Ok(captured)
    }
    pub(super) fn has_schema(&self, digest: [u8; 32]) -> bool {
        self.schemas.get(&digest).is_some()
    }
    pub(super) fn input_leaf(
        &self,
        prestate: [u8; 32],
        result: [u8; 32],
        mutation: &MutationConfig,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<[u8; 32], Refusal> {
        if !self.has_schema(prestate) || !self.has_schema(result) {
            return Err(Refusal::Malformed);
        }
        let carrier = (prestate != result).then_some(result);
        self.prefixes
            .get(&carrier)
            .ok_or(Refusal::Malformed)?
            .finish(prestate, result, mutation, request)
            .map_err(Refusal::Native)
    }
    pub(super) fn require_record(
        &self,
        record: &CanonicalOriginalRecord,
        index: &RelationalCommitIndexEntry,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        request
            .consult()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        if record.position != index.position
            || record.commit != index.commit.commit_id
            || record.payload_digest != index.payload_digest
            || self
                .schemas
                .get(&record.result_schema)
                .is_none_or(|schema| schema.schema_root() != *index.root.schema_root())
        {
            return Err(Refusal::Malformed);
        }
        let input_leaf = self.input_leaf(
            record.prestate_schema,
            record.result_schema,
            &record.mutation,
            request,
        )?;
        let computed = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactDigest::from_leaves(
            record.position.0, record.commit.0, position_leaf(record.position, request).map_err(Refusal::Native)?,
            record.envelope_leaf, input_leaf,
        );
        if computed.as_bytes() != &index.payload_digest {
            return Err(Refusal::Malformed);
        }
        Ok(())
    }
}
fn map(cause: worth_execution::ExecutionOrderedMapDenial) -> Refusal {
    Refusal::Native(crate::durability::checkpoint_history::capture::map_stop(
        cause,
    ))
}
