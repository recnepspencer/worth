//! Split escaping schema text admissions before the actual schema decoder.
use super::*;
use crate::schema::data::{SchemaTextReadmissionMemory, SchemaTransitionReadmissionMemory};

#[cfg(test)]
pub(in super::super::super) fn prepare_readmission(
    payload: &[u8],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(SchemaTransitionReadmissionMemory, u64), Refusal> {
    let canonical = field(BorrowedPayload(payload), "canonical", request)?;
    prepare_canonical_readmission(canonical, request)
}
pub(in super::super::super) fn prepare_canonical_readmission(
    canonical: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(SchemaTransitionReadmissionMemory, u64), Refusal> {
    // Serde's Option field accepts absence as None in the checkpoint wire.
    // Artifact admission still requires its exact authoritative byte form.
    let transition = canonical.optional_field("schema_transition", request)?;
    let mut count = 0usize;
    let mut retained = 0u64;
    visit(transition, request, &mut |text| {
        count = count.checked_add(1).ok_or_else(malformed)?;
        retained = retained
            .checked_add(change_text(text, request)?.retained)
            .ok_or_else(malformed)?;
        Ok(())
    })?;
    let bytes = u64::try_from(
        std::alloc::Layout::array::<SchemaTextReadmissionMemory>(count)
            .map_err(|_| malformed())?
            .size(),
    )
    .map_err(|_| malformed())?;
    let tree_memory = request
        .reserve_retained_memory(bytes)
        .map_err(|cause| Refusal::Request(cause.into()))?;
    let mut texts = Vec::with_capacity(count);
    visit(transition, request, &mut |text| {
        let memory = request
            .reserve_retained_memory(change_text(text, request)?.retained)
            .map_err(|cause| Refusal::Request(cause.into()))?;
        texts.push(SchemaTextReadmissionMemory {
            length: text.text(request)?.len(),
            memory,
        });
        Ok(())
    })?;
    Ok((
        SchemaTransitionReadmissionMemory {
            texts,
            _tree_memory: tree_memory,
        },
        retained,
    ))
}
fn visit(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
    visit: &mut impl FnMut(BorrowedPayload<'_>) -> Result<(), Refusal>,
) -> Result<(), Refusal> {
    if value.is_none() {
        return Ok(());
    }
    field(value, "diff_atoms", request)?.elements(request, |atom| {
        visit(field(
            field(atom, "element", request)?,
            "element_name",
            request,
        )?)?;
        field(atom, "detail", request)?.fields(request, |variant, detail| {
            match variant {
                "AddedField" => {
                    let text = detail.optional_field("default_expression", request)?;
                    if !text.is_none() {
                        visit(text)?;
                    }
                }
                "RemovedField" => {}
                "TypeChanged" => {
                    visit(field(detail, "from_type", request)?)?;
                    visit(field(detail, "to_type", request)?)?;
                }
                "EnumDomainExpanded" => {
                    field(detail, "added_variants", request)?.elements(request, &mut *visit)?;
                }
                "InvariantContractChanged" | "SubscriberContractChanged" => {
                    visit(field(detail, "contract_name", request)?)?
                }
                "ProjectionContractChanged" => visit(field(detail, "projection_name", request)?)?,
                "FreeText" => visit(field(detail, "detail", request)?)?,
                _ => return Err(malformed()),
            }
            Ok(())
        })
    })?;
    Ok(())
}
