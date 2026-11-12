mod combination;
mod field_level;
mod owned_fields;
#[cfg(test)]
mod tests;
mod whole_aspect;

pub(super) fn clear_ordinal_storage_bytes(count: usize) -> Option<usize> {
    Some(
        std::alloc::Layout::array::<owned_fields::ClearOrdinal>(count)
            .ok()?
            .size(),
    )
}
