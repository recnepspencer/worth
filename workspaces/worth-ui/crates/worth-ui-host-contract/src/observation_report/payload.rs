use super::UiHostObservationFamily;
use super::{
    UiHostScrollDeltaPhase, UiHostScrollDeltaPrecision, UiHostScrollDeltaSource,
    UiHostScrollDeltaTargetAffinity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiHostObservationPayload {
    Viewport {
        width_subpixels: i64,
        height_subpixels: i64,
    },
    DeviceScale {
        micros: u32,
    },
    PointerMotion {
        pointer: super::UiHostPointerIdentity,
        capture_epoch: super::UiHostPointerCaptureEpoch,
        pressed_buttons: super::UiHostPressedPointerButtons,
        position: super::UiHostSurfacePosition,
    },
    PointerButton {
        pointer: super::UiHostPointerIdentity,
        capture_epoch: super::UiHostPointerCaptureEpoch,
        button: super::UiHostPointerButton,
        transition: super::UiHostPointerButtonTransition,
        position: super::UiHostSurfacePosition,
    },
    Keyboard {
        logical_key: super::UiHostKey,
        physical_key: Option<super::UiHostKey>,
        modifiers: super::UiHostKeyboardModifiers,
        transition: super::UiHostKeyTransition,
    },
    WindowFocus {
        surface: crate::UiHostSurfaceIdentity,
        focused: bool,
    },
    ScrollDelta {
        source: UiHostScrollDeltaSource,
        phase: UiHostScrollDeltaPhase,
        precision: UiHostScrollDeltaPrecision,
        target: UiHostScrollDeltaTargetAffinity,
        x_subpixels: i64,
        y_subpixels: i64,
    },
    Clock {
        tick: u64,
    },
    Tick {
        tick: u64,
    },
    TextInput {
        revision: u64,
        text: Box<str>,
    },
    ImeComposition {
        revision: u64,
        phase: super::UiHostImeCompositionPhase,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostObservationCoalescingIdentity {
    Family(UiHostObservationFamily),
    PointerMotion {
        pointer: super::UiHostPointerIdentity,
        capture_epoch: super::UiHostPointerCaptureEpoch,
        pressed_buttons: super::UiHostPressedPointerButtons,
        device_kind: super::UiHostPointerDeviceKind,
    },
}

impl UiHostObservationPayload {
    pub const fn family(&self) -> UiHostObservationFamily {
        match self {
            Self::Viewport { .. } => UiHostObservationFamily::Viewport,
            Self::DeviceScale { .. } => UiHostObservationFamily::DeviceScale,
            Self::PointerMotion { .. } => UiHostObservationFamily::PointerMotion,
            Self::PointerButton { .. } => UiHostObservationFamily::PointerButton,
            Self::Keyboard { .. } => UiHostObservationFamily::Keyboard,
            Self::WindowFocus { .. } => UiHostObservationFamily::WindowFocus,
            Self::ScrollDelta { .. } => UiHostObservationFamily::ScrollDelta,
            Self::Clock { .. } => UiHostObservationFamily::Clock,
            Self::Tick { .. } => UiHostObservationFamily::Tick,
            Self::TextInput { .. } => UiHostObservationFamily::TextComposition,
            Self::ImeComposition { .. } => UiHostObservationFamily::ImeComposition,
        }
    }

    pub fn encoded_len(&self) -> usize {
        match self {
            Self::Viewport { .. } => 16,
            Self::DeviceScale { .. } => 4,
            Self::PointerMotion { .. } => 35,
            Self::PointerButton { .. } => 36,
            Self::Keyboard { .. } => 8,
            Self::WindowFocus { .. } => 17,
            Self::ScrollDelta { target, .. } => 19 + target.encoded_len(),
            Self::Clock { .. } | Self::Tick { .. } => 8,
            Self::TextInput { text, .. } => 8 + text.len(),
            Self::ImeComposition { phase, .. } => 9 + ime_encoded_len(phase),
        }
    }

    pub const fn coalescing_identity(&self) -> Option<UiHostObservationCoalescingIdentity> {
        match self {
            Self::Viewport { .. }
            | Self::DeviceScale { .. }
            | Self::Clock { .. }
            | Self::Tick { .. } => Some(UiHostObservationCoalescingIdentity::Family(self.family())),
            Self::PointerMotion {
                pointer,
                capture_epoch,
                pressed_buttons,
                ..
            } => Some(UiHostObservationCoalescingIdentity::PointerMotion {
                pointer: *pointer,
                capture_epoch: *capture_epoch,
                pressed_buttons: *pressed_buttons,
                device_kind: super::UiHostPointerDeviceKind::Mouse,
            }),
            _ => None,
        }
    }

    pub(super) fn integrity_digest(&self) -> u64 {
        let mut digest = UiHostObservationPayloadDigest::for_family(self.family());
        match self {
            Self::Viewport {
                width_subpixels,
                height_subpixels,
            } => digest.fold_pair(*width_subpixels as u64, *height_subpixels as u64),
            Self::DeviceScale { micros } => digest.fold(u64::from(*micros)),
            Self::PointerMotion {
                pointer,
                capture_epoch,
                pressed_buttons,
                position,
            } => {
                digest.fold(pointer.value());
                digest.fold(capture_epoch.value());
                digest.fold(u64::from(pressed_buttons.bits()));
                digest.fold_position(*position);
            }
            Self::PointerButton {
                pointer,
                capture_epoch,
                button,
                transition,
                position,
            } => digest.fold_pointer_button(
                *pointer,
                *capture_epoch,
                *button,
                *transition,
                *position,
            ),
            Self::Keyboard {
                logical_key,
                physical_key,
                modifiers,
                transition,
            } => digest.fold_keyboard(*logical_key, *physical_key, *modifiers, *transition),
            Self::WindowFocus { surface, focused } => {
                digest.fold(surface.diagnostic_value());
                digest.fold(u64::from(*focused));
            }
            Self::ScrollDelta {
                source,
                phase,
                precision,
                target,
                x_subpixels,
                y_subpixels,
            } => {
                digest.fold_pair(*source as u64, *phase as u64);
                digest.fold(*precision as u64);
                digest.fold_scroll_target(*target);
                digest.fold_pair(*x_subpixels as u64, *y_subpixels as u64);
            }
            Self::Clock { tick } | Self::Tick { tick } => digest.fold(*tick),
            Self::TextInput { revision, text } => {
                digest.fold_text(*revision, text);
            }
            Self::ImeComposition { revision, phase } => {
                digest.fold_ime(*revision, phase);
            }
        }
        digest.finish()
    }
}

struct UiHostObservationPayloadDigest(u64);

impl UiHostObservationPayloadDigest {
    fn for_family(family: UiHostObservationFamily) -> Self {
        Self(family as u64 + 1)
    }

    fn fold(&mut self, value: u64) {
        self.0 = self.0.rotate_left(9) ^ value.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    }

    fn fold_pair(&mut self, first: u64, second: u64) {
        self.fold(first);
        self.fold(second);
    }

    fn fold_pointer_button(
        &mut self,
        pointer: super::UiHostPointerIdentity,
        capture_epoch: super::UiHostPointerCaptureEpoch,
        button: super::UiHostPointerButton,
        transition: super::UiHostPointerButtonTransition,
        position: super::UiHostSurfacePosition,
    ) {
        self.fold(pointer.value());
        self.fold(capture_epoch.value());
        self.fold_pair(button as u64, transition as u64);
        self.fold_position(position);
    }

    fn fold_keyboard(
        &mut self,
        logical_key: super::UiHostKey,
        physical_key: Option<super::UiHostKey>,
        modifiers: super::UiHostKeyboardModifiers,
        transition: super::UiHostKeyTransition,
    ) {
        self.fold(logical_key as u64 + 1);
        self.fold(physical_key.map_or(0, |key| key as u64 + 1));
        self.fold(u64::from(modifiers.bits()));
        match transition {
            super::UiHostKeyTransition::Pressed { repeat } => {
                self.fold_pair(1, u64::from(repeat));
            }
            super::UiHostKeyTransition::Released => self.fold_pair(2, 0),
        }
    }

    fn fold_text(&mut self, revision: u64, text: &str) {
        self.fold(revision);
        for byte in text.bytes() {
            self.fold(u64::from(byte));
        }
    }

    fn fold_ime(&mut self, revision: u64, phase: &super::UiHostImeCompositionPhase) {
        self.fold(revision);
        match phase {
            super::UiHostImeCompositionPhase::Preedit(preedit) => {
                self.fold(1);
                self.fold_text(0, preedit.text());
                match preedit.selection() {
                    super::UiHostImePreeditSelection::Unspecified => self.fold(0),
                    super::UiHostImePreeditSelection::Converted(receipt) => {
                        self.fold(1);
                        self.fold_pair(
                            u64::from(receipt.source().start()),
                            u64::from(receipt.source().end()),
                        );
                        self.fold_pair(
                            u64::from(receipt.canonical().start()),
                            u64::from(receipt.canonical().end()),
                        );
                    }
                }
            }
            super::UiHostImeCompositionPhase::Commit(text) => {
                self.fold(2);
                self.fold_text(0, text);
            }
            super::UiHostImeCompositionPhase::Cancel => self.fold(3),
        }
    }

    fn fold_position(&mut self, position: super::UiHostSurfacePosition) {
        self.fold_pair(
            position.basis().coordinate_space() as u64,
            position.basis().coordinate_unit() as u64,
        );
        self.fold_pair(position.x_subpixels() as u64, position.y_subpixels() as u64);
    }

    fn fold_scroll_target(&mut self, target: UiHostScrollDeltaTargetAffinity) {
        let presentation = target.presentation();
        self.fold(presentation.host_surface().diagnostic_value());
        self.fold(presentation.frame().diagnostic_value());
        self.fold(presentation.binding().diagnostic_value());
        self.fold(presentation.epoch().diagnostic_value());
        match target {
            UiHostScrollDeltaTargetAffinity::ExactCoordinate { position, .. } => {
                self.fold(1);
                self.fold_position(position);
            }
            UiHostScrollDeltaTargetAffinity::ExactMountedTarget { mounted, .. } => {
                self.fold(2);
                self.fold(mounted.instance().diagnostic_value());
                self.fold(mounted.node_receipt().diagnostic_value());
            }
            UiHostScrollDeltaTargetAffinity::PresentedSurfaceFallback { .. } => self.fold(3),
        }
    }

    fn finish(self) -> u64 {
        self.0
    }
}

fn ime_encoded_len(phase: &super::UiHostImeCompositionPhase) -> usize {
    match phase {
        super::UiHostImeCompositionPhase::Preedit(preedit) => preedit.encoded_len(),
        super::UiHostImeCompositionPhase::Commit(text) => text.len(),
        super::UiHostImeCompositionPhase::Cancel => 0,
    }
}
