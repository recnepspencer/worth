use super::UiScrollOwnerRecord;

pub(super) fn reconcile_owner_record(
    previous: Option<UiScrollOwnerRecord>,
    registration: super::super::UiScrollOwnerRegistration,
    successor_anchor: Option<super::super::UiScrollAnchor>,
    policy: super::super::UiScrollAnchorPolicy,
) -> (
    super::super::UiScrollAnchorReconciliationOutcome,
    super::super::UiScrollOffset,
    Option<super::super::UiScrollAnchor>,
) {
    let Some(previous) = previous else {
        return (
            super::super::UiScrollAnchorReconciliationOutcome::Replaced,
            registration.initial_offset(),
            successor_anchor,
        );
    };
    match policy {
        super::super::UiScrollAnchorPolicy::Preserve
            if previous.incarnation == registration.incarnation()
                && previous
                    .anchor
                    .zip(successor_anchor)
                    .is_some_and(|(old, new)| old.exact_basis(new)) =>
        {
            let offset = registration.bounds().clamp(previous.offset);
            let outcome = if offset == previous.offset {
                super::super::UiScrollAnchorReconciliationOutcome::Preserved
            } else {
                super::super::UiScrollAnchorReconciliationOutcome::Clamped
            };
            (outcome, offset, successor_anchor)
        }
        super::super::UiScrollAnchorPolicy::Rebase => match previous.anchor.zip(successor_anchor) {
            Some((old, new))
                if previous.incarnation == registration.incarnation() && old.exact_basis(new) =>
            {
                let offset = registration.bounds().clamp(previous.offset);
                let outcome = if offset == previous.offset {
                    super::super::UiScrollAnchorReconciliationOutcome::Preserved
                } else {
                    super::super::UiScrollAnchorReconciliationOutcome::Clamped
                };
                (outcome, offset, successor_anchor)
            }
            Some((old, new)) if old.same_identity(new) => {
                let offset = registration
                    .bounds()
                    .clamp(rebased_offset(previous.offset, old, new));
                (
                    super::super::UiScrollAnchorReconciliationOutcome::Rebased,
                    offset,
                    successor_anchor,
                )
            }
            _ => dropped(registration),
        },
        super::super::UiScrollAnchorPolicy::Clamp => (
            super::super::UiScrollAnchorReconciliationOutcome::Clamped,
            registration.bounds().clamp(previous.offset),
            successor_anchor.or(previous.anchor),
        ),
        super::super::UiScrollAnchorPolicy::Preserve => dropped(registration),
    }
}

fn dropped(
    registration: super::super::UiScrollOwnerRegistration,
) -> (
    super::super::UiScrollAnchorReconciliationOutcome,
    super::super::UiScrollOffset,
    Option<super::super::UiScrollAnchor>,
) {
    (
        super::super::UiScrollAnchorReconciliationOutcome::Dropped,
        registration.initial_offset(),
        None,
    )
}

fn rebased_offset(
    offset: super::super::UiScrollOffset,
    old: super::super::UiScrollAnchor,
    new: super::super::UiScrollAnchor,
) -> super::super::UiScrollOffset {
    let inline = i128::from(offset.inline_subpixels()) + i128::from(new.inline_subpixels())
        - i128::from(old.inline_subpixels());
    let block = i128::from(offset.block_subpixels()) + i128::from(new.block_subpixels())
        - i128::from(old.block_subpixels());
    super::super::UiScrollOffset::new(clamp_nonnegative(inline), clamp_nonnegative(block)).unwrap()
}

fn clamp_nonnegative(value: i128) -> i64 {
    i64::try_from(value.clamp(0, i128::from(i64::MAX))).expect("clamped into i64")
}
