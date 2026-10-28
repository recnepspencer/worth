//! Read-only exact-field inspection through the current Query and native codecs.
use super::*;
pub use worth_relational::facade::durability::RecordAllocationClass as WorthQueryCheckpointRecordClass;
pub use worth_relational::facade::durability::{
    CheckpointAllocatorInspection, CheckpointRecordFrontier, CheckpointRecordIdentity,
    CheckpointReservationAdvanceDenial, NativeCheckpointModelBytes, RelationalCheckpointInspection,
};
#[derive(Debug, PartialEq, Eq)]
/// Exact original model value bytes, including Query output metadata.
/// Native allocator frontiers and high-waters remain separate typed data.
pub struct WorthQueryCheckpointModelBytes<'a> {
    pub native: NativeCheckpointModelBytes<'a>,
    query_header: &'a [u8],
    accepted_count: &'a [u8],
    query_outputs: &'a [u8],
}
#[derive(Debug)]
/// Complete diagnostic projection through the current Query/native codecs.
/// These values grant no recovery, publication or mutation authority.
pub struct WorthQueryApplicationCheckpointInspection<'a> {
    pub model: WorthQueryCheckpointModelBytes<'a>,
    pub symbol_identity_frontier: u32,
    pub frontiers: Vec<CheckpointRecordFrontier>,
    pub generations: Vec<CheckpointRecordIdentity>,
    pub live_records: std::collections::BTreeSet<CheckpointRecordIdentity>,
}
#[derive(Debug)]
/// The original phase, Query-frame or native-codec refusal of inspection.
pub enum WorthQueryCheckpointInspectionDenial {
    ForeignPhase(super::super::WorthQueryForeignAdvancementPhase),
    Query(super::super::WorthQueryPrimaryGraphInstallationDenial),
    Native(Box<worth_relational::facade::durability::RelationalRecoveryStop>),
}
impl<Schema: worth_query_installation::facade::ApplicationSchema + 'static>
    super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// Reconstructive, read-only inspection of the complete checkpoint. Native
    /// grammar and arenas stay with their codec owner; Query metadata retains
    /// its original value bytes. This comparison view is not recovery admission.
    pub fn inspect_application_checkpoint<'a>(
        &self,
        phase: &super::super::WorthQueryAdvancementPhase<'_>,
        bytes: &'a [u8],
    ) -> Result<WorthQueryApplicationCheckpointInspection<'a>, WorthQueryCheckpointInspectionDenial>
    {
        let execution = phase
            .execution_request_for(&self.product_runtime)
            .map_err(WorthQueryCheckpointInspectionDenial::ForeignPhase)?;
        let header::CurrentSections {
            query_header,
            accepted_count_bytes,
            native,
            outputs,
            bootstrap_commit_id: _,
            accepted_count: _,
        } = header::sections(bytes).map_err(|e| {
            WorthQueryCheckpointInspectionDenial::Query(e.into_installation_denial())
        })?;
        let RelationalCheckpointInspection {
            model,
            symbol_identity_frontier,
            frontiers,
            generations,
            live_records,
        } = worth_relational::facade::durability::RelationalNativeCheckpoint::inspect_bytes(
            &execution, native,
        )
        .map_err(|e| WorthQueryCheckpointInspectionDenial::Native(Box::new(e)))?;
        Ok(WorthQueryApplicationCheckpointInspection {
            model: WorthQueryCheckpointModelBytes {
                native: model,
                query_header,
                accepted_count: accepted_count_bytes,
                query_outputs: outputs,
            },
            symbol_identity_frontier,
            frontiers,
            generations,
            live_records,
        })
    }
}
