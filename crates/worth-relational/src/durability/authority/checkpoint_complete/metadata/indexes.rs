//! Preserve the supported full cache tuple over the same generation owners.
use crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts;
use crate::durability::log::native_checkpoint_codec::member_frame::{self, MemberKind};
use crate::durability::{data::RelationalRecoveryStop, medium::RelationalDurableMemberLocator};
use crate::indexes::data::DerivedIndexGeneration;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use std::sync::Arc;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};

pub(in crate::durability::authority::checkpoint_complete) fn encode_indexes(
    generations: &[Arc<DerivedIndexGeneration>],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(RelationalDurableMemberLocator, ExecutionImmutableBytes), RelationalRecoveryStop> {
    member_frame::encode_for_request(MemberKind::DerivedIndex, &view(generations), request)
}
pub(super) fn view(generations: &[Arc<DerivedIndexGeneration>]) -> impl Serialize + '_ {
    (
        Generations(generations),
        None::<DerivedIndexCheckpointArtifacts>,
        0_u16,
    )
}
struct Generations<'a>(&'a [Arc<DerivedIndexGeneration>]);
impl Serialize for Generations<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut row = serializer.serialize_struct("DerivedIndexArtifacts", 1)?;
        row.serialize_field("generations", self.0)?;
        row.end()
    }
}
