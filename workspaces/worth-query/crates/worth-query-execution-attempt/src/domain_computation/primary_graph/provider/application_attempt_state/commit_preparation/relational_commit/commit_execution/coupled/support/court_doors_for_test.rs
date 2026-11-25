//! Sealed test doors over final support's private R4 window check.

/// R4's window verdict over integer images, image `n` being the candidate
/// after the round at position `n`: whether a read observed at `position`,
/// superseded at `invalidator`, is refused because the round at `changed_by`
/// changed it inside its window. `rounds` are the group's round positions,
/// a chain's superseded rounds included.
#[doc(hidden)]
pub fn superseded_window_refuses_for_test(
    position: usize,
    invalidator: usize,
    rounds: &[usize],
    changed_by: usize,
) -> bool {
    let images = rounds.iter().map(|round| (*round, round));
    super::window::window_verdict(
        position,
        invalidator,
        images,
        &position,
        |earlier, image| {
            Ok::<_, std::convert::Infallible>(
                (*earlier < changed_by && *image >= changed_by).then_some(changed_by),
            )
        },
    )
    .unwrap_or_else(|never| match never {})
    .is_some()
}
