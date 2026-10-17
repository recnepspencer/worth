//! Confined variant's deterministic std HashMap seed. This is not host entropy.
//! The only supported ABI is WASIp1 random_get(pointer, 16) -> errno.
use super::execution::{memory, Invocation};
use wasmi::{Caller, Error, Val};

pub(super) fn fill(
    mut caller: Caller<'_, Invocation>,
    parameters: &[Val],
    results: &mut [Val],
) -> Result<(), Error> {
    let [Val::I32(pointer), Val::I32(16)] = parameters else {
        return Err(Error::host(super::Failure::ForbiddenEffect("unsupported guest random request; only deterministic 16-byte hash seeding is qualified".into())));
    };
    if results.len() != 1 {
        return Err(Error::new("invalid deterministic hash seed ABI"));
    }
    memory(&caller)?
        .write(&mut caller, *pointer as u32 as usize, &[0x57; 16])
        .map_err(|e| Error::new(e.to_string()))?;
    results[0] = Val::I32(0);
    Ok(())
}
