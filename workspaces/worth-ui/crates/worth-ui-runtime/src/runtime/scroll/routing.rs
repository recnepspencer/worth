pub(super) fn consume_axis(current: i64, maximum: i64, delta: i64, accepts: bool) -> (i64, i64) {
    if !accepts || delta == 0 {
        return (current, 0);
    }
    let desired = i128::from(current) + i128::from(delta);
    let next = i64::try_from(desired.clamp(0, i128::from(maximum)))
        .expect("clamped between zero and an i64 maximum");
    (next, next - current)
}

pub(super) fn consume_delta(
    current: super::UiScrollOffset,
    bounds: super::UiScrollBounds,
    axes: super::UiScrollAxes,
    delta: super::UiScrollDelta,
) -> (super::UiScrollOffset, super::UiScrollDelta) {
    let (inline, consumed_inline) = consume_axis(
        current.inline_subpixels(),
        bounds.max_inline_subpixels(),
        delta.inline_subpixels(),
        axes.accepts_inline(),
    );
    let (block, consumed_block) = consume_axis(
        current.block_subpixels(),
        bounds.max_block_subpixels(),
        delta.block_subpixels(),
        axes.accepts_block(),
    );
    (
        super::UiScrollOffset::new(inline, block).expect("bounded scroll offset is non-negative"),
        super::UiScrollDelta::new(consumed_inline, consumed_block),
    )
}
