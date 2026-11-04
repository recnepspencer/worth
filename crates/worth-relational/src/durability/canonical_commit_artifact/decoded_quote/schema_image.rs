//! Before-growth admission for the existing full schema-image grammar.
use super::*;
use crate::durability::data::{
    DurableBranchRootSchemaImage, DurableEntitySchemaRegistration, DurableSchemaDeclarationMemory,
};
use worth_foundational::facade::PortableAspectContract;
mod integrity;

pub(crate) struct SchemaImageDecodeAdmission {
    memory: Option<worth_execution::ExecutionMemoryReservation>,
    declarations: Vec<DurableSchemaDeclarationMemory>,
    _tree: Option<worth_execution::ExecutionMemoryReservation>,
    _transient: Option<worth_execution::ExecutionMemoryReservation>,
}
impl SchemaImageDecodeAdmission {
    pub(crate) fn attach(
        mut self,
        image: &mut DurableBranchRootSchemaImage,
    ) -> Result<(), Refusal> {
        image
            .retain_decode_memory(self.memory.take(), &mut self.declarations)
            .map_err(|detail| Refusal::PayloadDecode {
                detail: detail.into(),
            })
    }
}
pub(crate) fn prepare(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<SchemaImageDecodeAdmission, Refusal> {
    let allocation = quote(value, request)?;
    let mut count = 0usize;
    let mut leaves = 0u64;
    visit_declarations(value, request, |raw| {
        count = count.checked_add(1).ok_or_else(malformed)?;
        leaves = leaves
            .checked_add(super::codec_retirements::admission::declaration_leaf_bytes(
                raw, request,
            )?)
            .ok_or_else(malformed)?;
        Ok(())
    })?;
    let memory = request
        .reserve_retained_memory(
            allocation
                .retained
                .checked_sub(leaves)
                .ok_or_else(malformed)?,
        )
        .map_err(|cause| Refusal::Request(cause.into()))?;
    let transient = request
        .reserve_retained_memory(allocation.transient)
        .map_err(|cause| Refusal::Request(cause.into()))?;
    let tree_bytes = std::alloc::Layout::array::<DurableSchemaDeclarationMemory>(count)
        .map_err(|_| malformed())?
        .size();
    let tree = request
        .reserve_retained_memory(u64::try_from(tree_bytes).map_err(|_| malformed())?)
        .map_err(|cause| Refusal::Request(cause.into()))?;
    let mut declarations = Vec::with_capacity(count);
    visit_declarations(value, request, |raw| {
        declarations.push(super::codec_retirements::admission::prepare_declaration(
            raw, request,
        )?);
        Ok(())
    })?;
    Ok(SchemaImageDecodeAdmission {
        memory,
        declarations,
        _tree: tree,
        _transient: transient,
    })
}
fn visit_declarations<'a>(
    value: BorrowedPayload<'a>,
    request: &ExecutionRequest<'_, '_>,
    mut visit: impl FnMut(BorrowedPayload<'a>) -> Result<(), Refusal>,
) -> Result<(), Refusal> {
    for name in ["entity_registrations", "relation_registrations"] {
        field(value, name, request)?.elements(request, |registration| {
            visit(field(registration, "declarations", request)?)
        })?;
    }
    Ok(())
}
fn quote(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut total = OwnerAllocation::from(
        u64::try_from(DurableBranchRootSchemaImage::decode_control_bytes().ok_or_else(malformed)?)
            .map_err(|_| malformed())?,
    );
    value.fields(request, |name, raw| {
        let bytes = match name {
            "entity_registrations" => {
                vector::<DurableEntitySchemaRegistration>(raw, request, |row| {
                    super::contracts::entity(row, request)
                })?
            }
            "relation_registrations" => {
                let mut bytes = OwnerAllocation::default();
                let count = raw.elements(request, |row| {
                    add(&mut bytes, super::contracts::entity(row, request)?)?;
                    add(
                        &mut bytes,
                        integrity::quote(field(row, "relation_integrity", request)?, request)?,
                    )
                })?;
                let roster = u64::try_from(
                    DurableBranchRootSchemaImage::decode_relation_roster_bytes(vector_capacity(
                        count,
                    )?)
                    .ok_or_else(malformed)?,
                )
                .map_err(|_| malformed())?;
                add(
                    &mut bytes,
                    OwnerAllocation {
                        retained: roster,
                        transient: roster,
                    },
                )?;
                bytes
            }
            "retained_aspect_contracts" => vector::<PortableAspectContract>(raw, request, |row| {
                super::contracts::contract(row, request)
            })?,
            "derived_relations" => {
                vector::<crate::derived_relation::DerivedRelationDefinitionParts>(
                    raw,
                    request,
                    |row| super::schema::derived(row, request),
                )?
            }
            "projected_member_domains" => vector::<
                crate::projected_member_domain::ProjectedMemberDomainParts,
            >(raw, request, |row| {
                super::schema::projected(row, request)
            })?,
            "format_version"
            | "carrier_digest"
            | "schema_root"
            | "descriptor_semantics_version" => OwnerAllocation::default(),
            _ => return Err(malformed()),
        };
        add(&mut total, bytes)
    })?;
    Ok(total)
}
