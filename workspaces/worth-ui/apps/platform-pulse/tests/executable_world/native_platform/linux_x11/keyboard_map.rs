//! Keysym to keycode resolution through the server's own mapping. XTEST
//! synthesises keycodes, the courtroom speaks in keys; the server's
//! `GetKeyboardMapping` is the only honest bridge, and a key the server
//! cannot produce is a typed environment denial rather than a guessed code.
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt as _, Keycode, Keysym};

use crate::external_observation::NativeKeyboardCommand;
use crate::native_platform::NativePlatformFailure;

use super::connection::{input_failure, X11Observation};

// X11/keysymdef.h values; stable protocol constants, not a vendor table.
pub(super) const XK_ESCAPE: Keysym = 0xff1b;
pub(super) const XK_TAB: Keysym = 0xff09;
pub(super) const XK_SHIFT_L: Keysym = 0xffe1;
pub(super) const XK_CONTROL_L: Keysym = 0xffe3;
pub(super) const XK_A: Keysym = 0x61;
pub(super) const XK_P: Keysym = 0x70;

/// What the observer presses: one key, or the product's primary chord.
/// On Linux the product's primary modifier is Control (`keyboard.rs`
/// renders `super` as a separate modifier off macOS).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NativeKeyboardInput {
    Single(Keysym),
    PrimaryShiftP,
}

impl NativeKeyboardInput {
    pub(super) const fn expected_event_count(self) -> u32 {
        match self {
            Self::Single(_) => 2,
            Self::PrimaryShiftP => 6,
        }
    }

    pub(super) const fn for_command(command: NativeKeyboardCommand) -> Self {
        match command {
            NativeKeyboardCommand::Escape => Self::Single(XK_ESCAPE),
            NativeKeyboardCommand::Tab => Self::Single(XK_TAB),
            NativeKeyboardCommand::PrimaryShiftP => Self::PrimaryShiftP,
        }
    }

    /// Press/release keycode sequence in delivery order.
    pub(super) fn keycode_sequence(
        self,
        map: &KeyboardMap,
    ) -> Result<Vec<(Keycode, bool)>, NativePlatformFailure> {
        Ok(match self {
            Self::Single(keysym) => {
                let code = map.keycode(keysym)?;
                vec![(code, true), (code, false)]
            }
            Self::PrimaryShiftP => {
                let control = map.keycode(XK_CONTROL_L)?;
                let shift = map.keycode(XK_SHIFT_L)?;
                let p = map.keycode(XK_P)?;
                vec![
                    (control, true),
                    (shift, true),
                    (p, true),
                    (p, false),
                    (shift, false),
                    (control, false),
                ]
            }
        })
    }
}

pub(super) struct KeyboardMap {
    min_keycode: Keycode,
    keysyms_per_keycode: u8,
    keysyms: Vec<Keysym>,
}

impl KeyboardMap {
    pub(super) fn read(x11: &X11Observation) -> Result<Self, NativePlatformFailure> {
        let setup = x11.connection().setup();
        let (min_keycode, max_keycode) = (setup.min_keycode, setup.max_keycode);
        let count = max_keycode
            .checked_sub(min_keycode)
            .and_then(|span| span.checked_add(1))
            .ok_or_else(|| input_failure("server keycode range is inverted"))?;
        let reply = x11
            .connection()
            .get_keyboard_mapping(min_keycode, count)
            .map_err(input_failure)?
            .reply()
            .map_err(input_failure)?;
        Ok(Self {
            min_keycode,
            keysyms_per_keycode: reply.keysyms_per_keycode,
            keysyms: reply.keysyms,
        })
    }

    /// The first keycode whose unshifted (column 0) keysym is `keysym`.
    /// Column 0 only: a keysym reachable solely through a modifier would
    /// need that modifier pressed too, which is not the key the courtroom
    /// asked for.
    pub(super) fn keycode(&self, keysym: Keysym) -> Result<Keycode, NativePlatformFailure> {
        keycode_in(
            self.min_keycode,
            self.keysyms_per_keycode,
            &self.keysyms,
            keysym,
        )
        .ok_or_else(|| input_failure(format!("keysym {keysym:#x} has no unshifted keycode")))
    }
}

fn keycode_in(
    min_keycode: Keycode,
    keysyms_per_keycode: u8,
    keysyms: &[Keysym],
    keysym: Keysym,
) -> Option<Keycode> {
    let columns = usize::from(keysyms_per_keycode).max(1);
    keysyms
        .chunks(columns)
        .position(|row| row.first() == Some(&keysym))
        .and_then(|offset| u8::try_from(offset).ok())
        .and_then(|offset| min_keycode.checked_add(offset))
}

#[cfg(test)]
mod tests {
    use super::{keycode_in, NativeKeyboardInput, XK_A, XK_ESCAPE, XK_SHIFT_L};
    use crate::external_observation::NativeKeyboardCommand;

    #[test]
    fn unshifted_column_resolves_and_shifted_only_keysyms_do_not() {
        // keycode 8: a/A; keycode 9: Escape; keycode 10: Shift_L
        let keysyms = [XK_A, 0x41, XK_ESCAPE, 0, XK_SHIFT_L, 0];
        assert_eq!(keycode_in(8, 2, &keysyms, XK_A), Some(8));
        assert_eq!(keycode_in(8, 2, &keysyms, XK_ESCAPE), Some(9));
        assert_eq!(keycode_in(8, 2, &keysyms, XK_SHIFT_L), Some(10));
        assert_eq!(keycode_in(8, 2, &keysyms, 0x41), None);
    }

    #[test]
    fn command_inputs_carry_the_same_event_counts_as_the_windows_observer() {
        assert_eq!(
            NativeKeyboardInput::for_command(NativeKeyboardCommand::Escape).expected_event_count(),
            2
        );
        assert_eq!(
            NativeKeyboardInput::for_command(NativeKeyboardCommand::PrimaryShiftP)
                .expected_event_count(),
            6
        );
    }
}
