//! Erased comparisons cannot swallow the caller's exact original failure.
pub(super) fn with_checkpoint<E>(
    admit: &mut impl FnMut(u64) -> Result<(), E>,
    compare: &mut impl FnMut(&mut dyn FnMut(u64) -> bool) -> bool,
) -> Result<bool, E> {
    let mut failure = None;
    let equal = compare(&mut |work| {
        if failure.is_some() {
            return false;
        }
        match admit(work) {
            Ok(()) => true,
            Err(cause) => {
                failure = Some(cause);
                false
            }
        }
    });
    match failure {
        Some(cause) => Err(cause),
        None => {
            admit(0)?;
            Ok(equal)
        }
    }
}
#[cfg(test)]
mod tests;
