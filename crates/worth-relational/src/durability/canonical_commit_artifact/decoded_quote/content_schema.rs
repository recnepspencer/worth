//! Admit Content's two Arc<Vec> owners and descriptor names before decode.
use super::*;
use crate::content::{RelationalContentFieldBinding, RelationalContentFormatDeclaration};

pub(super) fn quote(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let formats = field(value, "formats", request)?;
    let fields = field(value, "fields", request)?;
    let mut bytes = vector::<RelationalContentFormatDeclaration>(formats, request, |row| {
        let descriptor = field(row, "descriptor", request)?;
        text(field(descriptor, "name", request)?, request)
    })?;
    add(
        &mut bytes,
        shared_root::<Vec<RelationalContentFormatDeclaration>>()?,
    )?;
    add(
        &mut bytes,
        vector::<RelationalContentFieldBinding>(fields, request, |_| {
            Ok(OwnerAllocation::default())
        })?,
    )?;
    add(
        &mut bytes,
        shared_root::<Vec<RelationalContentFieldBinding>>()?,
    )?;
    Ok(bytes)
}
