//! Decode the finite chronology only after its actual owners are admitted.
use super::{frame::*, image::*};
use crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

impl RelationalSourceInitialInputsArtifact {
    pub(crate) fn readmit(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<InitialInputsImage>, RelationalInitialInputsRefusal> {
        use RelationalInitialInputsRefusal as Refusal;
        request
            .check_live()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        let raw = BorrowedPayload(self.payload());
        let entries = raw
            .field("entries", request)
            .map_err(Refusal::Payload)?
            .elements(request, |_| Ok(()))
            .map_err(Refusal::Payload)?;
        let schema_raw = raw.field("schemas", request).map_err(Refusal::Payload)?;
        let schemas = schema_raw
            .elements(request, |_| Ok(()))
            .map_err(Refusal::Payload)?;
        if !(1..=2).contains(&entries) || schemas > 3 {
            return Err(Refusal::Malformed);
        }
        let main = raw
            .field("main", request)
            .map_err(Refusal::Payload)?
            .text_length(request)
            .map_err(Refusal::Payload)?;
        let memory = request
            .reserve_retained_memory(layout(entries, schemas, main).ok_or(Refusal::Malformed)?)
            .map_err(|cause| Refusal::Request(cause.into()))?;
        let scratch_bytes = std::alloc::Layout::array::<
            crate::durability::canonical_commit_artifact::SchemaImageDecodeAdmission,
        >(schemas)
        .map_err(|_| Refusal::Malformed)?
        .size();
        let _scratch_memory = request
            .reserve_retained_memory(u64::try_from(scratch_bytes).map_err(|_| Refusal::Malformed)?)
            .map_err(|cause| Refusal::Request(cause.into()))?;
        let mut admissions = Vec::with_capacity(schemas);
        schema_raw
            .elements(request, |raw| {
                admissions.push(
                    crate::durability::canonical_commit_artifact::prepare_schema_image_decode(
                        raw, request,
                    )?,
                );
                Ok(())
            })
            .map_err(Refusal::Payload)?;
        let mut image: InitialInputsImage =
            rmp_serde::from_slice(self.payload()).map_err(|_| Refusal::Malformed)?;
        if image.schemas.len() != admissions.len() || image.revision != self.revision() {
            return Err(Refusal::Malformed);
        }
        for (schema, admission) in image.schemas.iter_mut().zip(admissions) {
            admission.attach(schema).map_err(Refusal::Payload)?;
        }
        image.validate()?;
        image.artifact = Some(self.clone());
        image._memory = memory;
        Ok(Arc::new(image))
    }
}

pub(super) fn layout(entries: usize, schemas: usize, main: usize) -> Option<u64> {
    let control = std::alloc::Layout::new::<(usize, usize)>()
        .extend(std::alloc::Layout::new::<InitialInputsImage>())
        .ok()?
        .0
        .pad_to_align()
        .size();
    let entries = std::alloc::Layout::array::<InitialInputEntry>(entries)
        .ok()?
        .size();
    let schemas =
        std::alloc::Layout::array::<crate::durability::data::DurableBranchRootSchemaImage>(schemas)
            .ok()?
            .size();
    u64::try_from(
        control
            .checked_add(entries)?
            .checked_add(schemas)?
            .checked_add(main)?,
    )
    .ok()
}
