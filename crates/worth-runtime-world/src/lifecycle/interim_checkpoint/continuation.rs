//! Carry unchanged World originals through exclusive Native bootstrap custody.
use super::*;
use worth_relational::facade::{
    branch::RelationalBranchBasisPort,
    durability::{CapturedNativeInterimAdmission, NativeInterimBootstrapContinuation},
};

impl CapturedWorldInterimOrigin {
    pub fn continue_after_native_transition(
        &self,
        request: &ExecutionRequest<'_, '_>,
        port: &RelationalBranchBasisPort,
        continuation: &NativeInterimBootstrapContinuation,
        native: &CapturedNativeInterimAdmission,
        application: CanonicalDigestId,
    ) -> Result<CapturedWorldInterimAdmission, Stop> {
        request_scope::run(request, || {
            self.validate_transport(self.0.wrapper.bytes(), request)?;
            let old = &self.0.captured;
            // Even an empty registry must join its actual original Native source.
            port.readmit_interim_memory_component(
                request,
                continuation.original_receiver(),
                &old.anchor_component,
            )
            .map_err(Stop::Native)?;
            let anchor_component = port
                .capture_interim_bootstrap_component(request, continuation, native)
                .map_err(Stop::Native)?;
            let mut rows =
                worth_execution::ExecutionRequestArrayBuilder::allocate(*request, old.rows.len())
                    .map_err(Stop::Backing)?;
            for row in old.rows.iter() {
                let before = port
                    .readmit_interim_memory_component(
                        request,
                        continuation.original_receiver(),
                        &row.before,
                    )
                    .map_err(Stop::Native)?;
                let after = port
                    .readmit_interim_memory_component(
                        request,
                        continuation.original_receiver(),
                        &row.after,
                    )
                    .map_err(Stop::Native)?;
                let mut carried = CapturedRow {
                    meaning: row.meaning.clone(),
                    before: port
                        .capture_interim_memory_component(request, native, &before)
                        .map_err(Stop::Component)?,
                    after: port
                        .capture_interim_memory_component(request, native, &after)
                        .map_err(Stop::Component)?,
                    bound_to_anchor: row.bound_to_anchor,
                    digest: CanonicalDigestId::new([0; 32]),
                };
                carried.digest = wire::row_digest(&carried, request)?;
                rows.push(carried).map_err(Stop::Backing)?;
            }
            CapturedRegistry::prepare(
                native,
                application,
                old.anchor.clone(),
                anchor_component,
                rows.seal().map_err(Stop::Backing)?,
                request,
            )
        })
    }
}

impl CapturedRegistry {
    pub(super) fn prepare(
        native: &CapturedNativeInterimAdmission,
        application: CanonicalDigestId,
        anchor: Arc<MemoryPublicationReceiverScope>,
        anchor_component: CapturedNativeInterimComponent,
        rows: ExecutionArray<CapturedRow>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CapturedWorldInterimAdmission, Stop> {
        let memory = reserve_arc::<Self>(request, 0)?;
        let (bytes, digest) = wire::encode(
            native,
            application,
            &anchor.reference,
            &anchor_component,
            &rows,
            request,
        )?;
        Ok(CapturedWorldInterimAdmission(Arc::new(Self {
            coordinate: native.coordinate(),
            native_digest: native.native_digest(),
            covered: native.covered_position(),
            transitions: native.covered_transitions(),
            application,
            anchor,
            anchor_component,
            rows,
            bytes,
            digest,
            _memory: memory,
        })))
    }
}
