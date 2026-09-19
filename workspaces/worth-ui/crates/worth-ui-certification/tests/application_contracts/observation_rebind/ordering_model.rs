use worth_ui::facade::observation::{
    UiObservationCoalescingPolicy, UiObservationDuplicatePolicy, UiObservationFamily,
    UiObservationLossPolicy, UiObservationOwner, UiObservationResetPolicy,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ModelFamily {
    Source,
    HostViewport,
    HostDeviceScale,
    PointerPresence,
    Measurement,
    Query,
    IntentPosture,
    Scroll,
    Portal,
    Focus,
    Selection,
    Motion,
    CommandRoute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModelDuplicate {
    Reject,
    OwnerEquivalentMayCoalesce,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModelLoss {
    Lossless,
    OwnerDeclaredLoss,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModelReset {
    NoReset,
    OwnerIssuedReset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ModelCoalescing {
    Forbidden,
    OwnerEquivalentOnly,
}

#[derive(Clone, Copy)]
struct ModelLaw {
    family: ModelFamily,
    rank: u8,
    duplicate: ModelDuplicate,
    loss: ModelLoss,
    reset: ModelReset,
    coalescing: ModelCoalescing,
}

const LAWS: [ModelLaw; 13] = [
    law(
        ModelFamily::Source,
        0,
        ModelDuplicate::Reject,
        ModelLoss::Lossless,
        ModelReset::NoReset,
        ModelCoalescing::Forbidden,
    ),
    law(
        ModelFamily::HostViewport,
        1,
        ModelDuplicate::OwnerEquivalentMayCoalesce,
        ModelLoss::OwnerDeclaredLoss,
        ModelReset::NoReset,
        ModelCoalescing::OwnerEquivalentOnly,
    ),
    law(
        ModelFamily::HostDeviceScale,
        2,
        ModelDuplicate::OwnerEquivalentMayCoalesce,
        ModelLoss::OwnerDeclaredLoss,
        ModelReset::NoReset,
        ModelCoalescing::OwnerEquivalentOnly,
    ),
    law(
        ModelFamily::PointerPresence,
        3,
        ModelDuplicate::OwnerEquivalentMayCoalesce,
        ModelLoss::Lossless,
        ModelReset::NoReset,
        ModelCoalescing::OwnerEquivalentOnly,
    ),
    law(
        ModelFamily::Measurement,
        4,
        ModelDuplicate::Reject,
        ModelLoss::Lossless,
        ModelReset::NoReset,
        ModelCoalescing::Forbidden,
    ),
    law(
        ModelFamily::Query,
        5,
        ModelDuplicate::Reject,
        ModelLoss::OwnerDeclaredLoss,
        ModelReset::OwnerIssuedReset,
        ModelCoalescing::Forbidden,
    ),
    law(
        ModelFamily::IntentPosture,
        6,
        ModelDuplicate::Reject,
        ModelLoss::Lossless,
        ModelReset::NoReset,
        ModelCoalescing::Forbidden,
    ),
    law(
        ModelFamily::Scroll,
        7,
        ModelDuplicate::OwnerEquivalentMayCoalesce,
        ModelLoss::Lossless,
        ModelReset::NoReset,
        ModelCoalescing::OwnerEquivalentOnly,
    ),
    law(
        ModelFamily::Portal,
        8,
        ModelDuplicate::OwnerEquivalentMayCoalesce,
        ModelLoss::Lossless,
        ModelReset::NoReset,
        ModelCoalescing::OwnerEquivalentOnly,
    ),
    law(
        ModelFamily::Focus,
        9,
        ModelDuplicate::OwnerEquivalentMayCoalesce,
        ModelLoss::Lossless,
        ModelReset::NoReset,
        ModelCoalescing::OwnerEquivalentOnly,
    ),
    law(
        ModelFamily::Selection,
        10,
        ModelDuplicate::OwnerEquivalentMayCoalesce,
        ModelLoss::Lossless,
        ModelReset::NoReset,
        ModelCoalescing::OwnerEquivalentOnly,
    ),
    law(
        ModelFamily::Motion,
        11,
        ModelDuplicate::OwnerEquivalentMayCoalesce,
        ModelLoss::Lossless,
        ModelReset::NoReset,
        ModelCoalescing::OwnerEquivalentOnly,
    ),
    law(
        ModelFamily::CommandRoute,
        12,
        ModelDuplicate::OwnerEquivalentMayCoalesce,
        ModelLoss::Lossless,
        ModelReset::NoReset,
        ModelCoalescing::OwnerEquivalentOnly,
    ),
];

const fn law(
    family: ModelFamily,
    rank: u8,
    duplicate: ModelDuplicate,
    loss: ModelLoss,
    reset: ModelReset,
    coalescing: ModelCoalescing,
) -> ModelLaw {
    ModelLaw {
        family,
        rank,
        duplicate,
        loss,
        reset,
        coalescing,
    }
}

#[test]
fn closed_owner_laws_match_production_definitions() {
    for expected in LAWS {
        let actual = production_family(expected.family).definition();
        assert_eq!(actual.family(), production_family(expected.family));
        assert_eq!(actual.owner(), expected_owner(expected.family));
        assert_eq!(actual.framework_rank(), expected.rank);
        assert_eq!(
            actual.duplicate_policy(),
            production_duplicate(expected.duplicate)
        );
        assert_eq!(actual.loss_policy(), production_loss(expected.loss));
        assert_eq!(actual.reset_policy(), production_reset(expected.reset));
        assert_eq!(
            actual.coalescing_policy(),
            production_coalescing(expected.coalescing)
        );
    }
}

#[test]
fn generated_cross_owner_traces_have_one_canonical_order() {
    for left in LAWS {
        for right in LAWS {
            for left_order in 0..3 {
                for right_order in 0..3 {
                    let mut model = [(left.rank, left_order), (right.rank, right_order)];
                    model.sort_unstable();
                    let mut production = [
                        (
                            production_family(left.family).definition().framework_rank(),
                            left_order,
                        ),
                        (
                            production_family(right.family)
                                .definition()
                                .framework_rank(),
                            right_order,
                        ),
                    ];
                    production.sort_unstable();
                    assert_eq!(production, model);
                }
            }
        }
    }
}

fn production_family(family: ModelFamily) -> UiObservationFamily {
    match family {
        ModelFamily::Source => UiObservationFamily::AuthoredSource,
        ModelFamily::HostViewport => UiObservationFamily::HostViewport,
        ModelFamily::HostDeviceScale => UiObservationFamily::HostDeviceScale,
        ModelFamily::PointerPresence => UiObservationFamily::PointerPresenceTarget,
        ModelFamily::Measurement => UiObservationFamily::Measurement,
        ModelFamily::Query => UiObservationFamily::Query,
        ModelFamily::IntentPosture => UiObservationFamily::IntentPosture,
        ModelFamily::Scroll => UiObservationFamily::CommittedScrollExtent,
        ModelFamily::Portal => UiObservationFamily::CommittedPortalAnchor,
        ModelFamily::Focus => UiObservationFamily::CommittedFocus,
        ModelFamily::Selection => UiObservationFamily::CommittedSelection,
        ModelFamily::Motion => UiObservationFamily::CommittedMotionTrack,
        ModelFamily::CommandRoute => UiObservationFamily::CommittedCommandRoute,
    }
}

fn expected_owner(family: ModelFamily) -> UiObservationOwner {
    match family {
        ModelFamily::Source => UiObservationOwner::SourceIngress,
        ModelFamily::HostViewport => UiObservationOwner::HostViewport,
        ModelFamily::HostDeviceScale => UiObservationOwner::HostDeviceScale,
        ModelFamily::PointerPresence => UiObservationOwner::PointerPresenceRuntimeState,
        ModelFamily::Measurement => UiObservationOwner::MeasurementExchange,
        ModelFamily::Query => UiObservationOwner::QueryBinding,
        ModelFamily::IntentPosture => UiObservationOwner::IntentRuntime,
        ModelFamily::Scroll => UiObservationOwner::ScrollRuntimeState,
        ModelFamily::Portal => UiObservationOwner::PortalRuntimeState,
        ModelFamily::Focus => UiObservationOwner::FocusRuntimeState,
        ModelFamily::Selection => UiObservationOwner::SelectionRuntimeState,
        ModelFamily::Motion => UiObservationOwner::MotionRuntimeState,
        ModelFamily::CommandRoute => UiObservationOwner::CommandRoutingRuntimeState,
    }
}

fn production_duplicate(value: ModelDuplicate) -> UiObservationDuplicatePolicy {
    match value {
        ModelDuplicate::Reject => UiObservationDuplicatePolicy::Reject,
        ModelDuplicate::OwnerEquivalentMayCoalesce => {
            UiObservationDuplicatePolicy::OwnerEquivalentMayCoalesce
        }
    }
}

fn production_loss(value: ModelLoss) -> UiObservationLossPolicy {
    match value {
        ModelLoss::Lossless => UiObservationLossPolicy::Lossless,
        ModelLoss::OwnerDeclaredLoss => UiObservationLossPolicy::OwnerDeclaredLoss,
    }
}

fn production_reset(value: ModelReset) -> UiObservationResetPolicy {
    match value {
        ModelReset::NoReset => UiObservationResetPolicy::NoReset,
        ModelReset::OwnerIssuedReset => UiObservationResetPolicy::OwnerIssuedReset,
    }
}

fn production_coalescing(value: ModelCoalescing) -> UiObservationCoalescingPolicy {
    match value {
        ModelCoalescing::Forbidden => UiObservationCoalescingPolicy::Forbidden,
        ModelCoalescing::OwnerEquivalentOnly => UiObservationCoalescingPolicy::OwnerEquivalentOnly,
    }
}
