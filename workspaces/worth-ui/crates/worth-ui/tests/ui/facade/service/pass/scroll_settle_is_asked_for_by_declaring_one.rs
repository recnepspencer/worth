//! The lawful counterpart: a product author asks for a settle by declaring one
//! and reads back what the declaration means. Nothing here names a prepared
//! result, because nothing product-side ever needs to -- the horizon is the
//! whole of the request, and its admission is answered at the declaration.

use worth_ui::facade::service::{
    UiScrollPolicy, UiScrollWheelBehavior, UI_SCROLL_WHEEL_SETTLE_TICK_CEILING,
};

fn main() {
    let immediate = UiScrollPolicy::nested_region();
    assert_eq!(
        immediate.wheel_behavior().settle_ticks(),
        None,
        "a policy that declares no smooth wheel asks for no settle at all"
    );

    let smooth = UiScrollPolicy::nested_region().with_wheel_behavior(
        UiScrollWheelBehavior::smooth(120).expect("120 ticks is an admitted settle horizon"),
    );
    assert_eq!(
        smooth.wheel_behavior().settle_ticks(),
        Some(120),
        "the authored horizon is what the policy carries"
    );

    assert!(
        UiScrollWheelBehavior::smooth(0).is_err(),
        "a horizon of zero is no settle"
    );
    assert!(
        UiScrollWheelBehavior::smooth(UI_SCROLL_WHEEL_SETTLE_TICK_CEILING + 1).is_err(),
        "a horizon past the ceiling is refused where it is authored"
    );
}
