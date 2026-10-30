//! Completed managed lifecycle effects survive the outer request handoff.
/// Preserve a completed effect across the execution owner's final liveness poll.
/// An absent or refused body never converts a pre-effect interruption to success.
pub(super) fn completed_effect<E>(
    advancement: Result<(), E>,
    completed: Option<Result<(), E>>,
) -> Result<(), E> {
    match (advancement, completed) {
        (_, Some(Ok(()))) => Ok(()),
        (Err(cause), _) | (Ok(()), Some(Err(cause))) => Err(cause),
        (Ok(()), None) => unreachable!("completed advancement ran the managed lifecycle body"),
    }
}

#[cfg(test)]
mod tests;
