use super::pointer_target::{classify_pointer_target, PointerTargetCheckPhase};
use crate::external_observation::NativeInputProbeKind;
use crate::native_platform::NativePlatformFailure;

#[test]
fn hostile_pointer_target_is_classified_by_effect_boundary() {
    assert!(matches!(
        classify_pointer_target(0x11, 0x22, PointerTargetCheckPhase::BeforeEffect),
        Err(NativePlatformFailure::InputEnvironment(_))
    ));
    assert!(matches!(
        classify_pointer_target(
            0x11,
            0x22,
            PointerTargetCheckPhase::AfterEffect {
                kind: NativeInputProbeKind::Pointer,
                delivered_event_count: 2,
            },
        ),
        Err(NativePlatformFailure::InputDeliveryIndeterminate {
            kind: NativeInputProbeKind::Pointer,
            delivered_event_count: 2,
            ..
        })
    ));
}
