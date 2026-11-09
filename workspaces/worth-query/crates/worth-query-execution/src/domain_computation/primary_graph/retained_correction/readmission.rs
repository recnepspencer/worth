//! Decode the complete positioned prefix before lending any semantic material.
use super::*;

impl CorrectionMaterialOwner {
    pub(in crate::domain_computation::primary_graph) fn readmit(
        medium: Arc<dyn QueryCorrectionsMedium>,
        request: &ExecutionRequest<'_, '_>,
        join: impl FnOnce(&MaterialCatalog) -> Result<(), Denial>,
    ) -> Result<Self, Denial> {
        let mut catalog = MaterialCatalog::empty(medium.home());
        let mut funding = Vec::new();
        loop {
            request.consult().map_err(Denial::Execution)?;
            let next = catalog
                .position
                .checked_add(1)
                .ok_or(Denial::IdentityExhausted)?;
            let Some(bytes) =
                medium.read_frame(request, next, request.memory_limit().unwrap_or(u64::MAX))?
            else {
                break;
            };
            let retained = (bytes.len() as u64)
                .checked_mul(4)
                .ok_or(Denial::Allocation)?;
            funding.push(
                request
                    .reserve_retained_memory(retained)
                    .map_err(|cause| Denial::Execution(cause.into()))?,
            );
            let frame = QueryCorrectionMaterialFrame::readmit(bytes)?;
            catalog.fold(&frame)?;
        }
        // DATA decoding does not admit a source. The actual Query receiver must
        // join every surviving original and command before constructing this owner.
        join(&catalog)?;
        let pins = pins::LiveMaterialPins::readmit(&catalog, medium.authority());
        let generation = catalog.generation;
        Ok(Self {
            medium: Mutex::new(Some(medium)),
            publication: Mutex::new(()),
            state: Mutex::new(MaterialState {
                catalog,
                allocated_generation: generation,
                pending: None,
                pins,
                funding: Arc::new(MaterialFunding(Mutex::new(funding))),
            }),
        })
    }
}
