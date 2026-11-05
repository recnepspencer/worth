use {crate::domain_computation::vocabulary::provider_session::denial::WorthQueryGraphProviderStepDenial, crate::domain_computation::vocabulary::provider_session::denial::WorthQueryGraphProviderStepDenialKind};

pub(crate) fn allocate_scratch_bytes(
    byte_count: usize,
) -> Result<Vec<u8>, WorthQueryGraphProviderStepDenial> {
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(byte_count).map_err(|_| {
        WorthQueryGraphProviderStepDenial::new(
            WorthQueryGraphProviderStepDenialKind::MemoryAllocationFailed,
            "provider scratch-memory allocation failed",
        )
    })?;
    bytes.resize(byte_count, 0);
    Ok(bytes)
}
