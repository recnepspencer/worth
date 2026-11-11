//! Concrete storage newly copied into one immutable schema-image owner.
use super::*;
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use std::alloc::Layout;
use worth_execution::ExecutionRequest;

mod declarations;
mod integrity;
mod recipes;

pub(super) struct Quote<'a, 'scope, 'policy> {
    execution: &'a ExecutionRequest<'scope, 'policy>,
    bytes: usize,
}
impl Quote<'_, '_, '_> {
    fn poll(&self) -> Result<(), RelationalRecoveryStop> {
        crate::execution::check_request(self.execution).map_err(RelationalRecoveryStop::Request)
    }
    fn add(&mut self, bytes: usize) -> Result<(), RelationalRecoveryStop> {
        self.bytes = self.bytes.checked_add(bytes).ok_or_else(overflow)?;
        Ok(())
    }
    fn array<T>(&mut self, count: usize) -> Result<(), RelationalRecoveryStop> {
        self.poll()?;
        self.add(Layout::array::<T>(count).map_err(|_| overflow())?.size())
    }
    fn text(&mut self, text: &str) -> Result<(), RelationalRecoveryStop> {
        self.array::<u8>(text.len())
    }
}

fn overflow() -> RelationalRecoveryStop {
    DurabilityError::new(
        RecoveryFailureClass::CheckpointSizeOverflow,
        "branch-root schema-image backing exceeds addressable capacity",
    )
    .into()
}

pub(super) fn quote(
    authority: &crate::branch::RelationalBranchRootSchemaAuthority,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<usize, RelationalRecoveryStop> {
    let layout = Layout::new::<(usize, usize)>()
        .extend(Layout::new::<SchemaImageData>())
        .map_err(|_| overflow())?
        .0
        .pad_to_align();
    let mut quote = Quote {
        execution,
        bytes: layout.size(),
    };
    let registry = authority.registry();
    quote.array::<DurableEntitySchemaRegistration>(registry.entity_kinds.len())?;
    for registration in registry.entity_kinds.values() {
        quote.text(&registration.kind_name)?;
        quote.text(&registration.schema_id.0)?;
        declarations::quote(&mut quote, &registration.aspect_contract_declarations)?;
    }
    quote.array::<DurableRelationSchemaRegistration>(registry.relation_kinds.len())?;
    for registration in registry.relation_kinds.values() {
        quote.text(&registration.kind_name)?;
        quote.text(&registration.schema_id.0)?;
        declarations::quote(&mut quote, &registration.aspect_contract_declarations)?;
        integrity::quote(&mut quote, &registration.relation_integrity)?;
    }
    quote.array::<PortableAspectContract>(authority.retained_aspect_contracts().len())?;
    for contract in authority.retained_aspect_contracts() {
        declarations::contract(&mut quote, contract)?;
    }
    recipes::quote(&mut quote, registry)?;
    Ok(quote.bytes)
}
