/// What the platform's wheel-lines setting store answered this host.
///
/// This is the raw answer and nothing more. It carries no notch meaning: the
/// wheel notch report is derived from it, never stored beside it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeWheelScrollLinesSetting {
    /// The store held this text for the current user.
    Stated(String),
    /// The store held an answer this host cannot read as text.
    Unreadable,
    /// No answer reached this host: the key, the value or the whole store is
    /// absent on this platform.
    Absent,
}

/// Read the platform's wheel-lines setting.
///
/// On Windows the setting lives at `HKCU\Control Panel\Desktop\WheelScrollLines`
/// as a `REG_SZ`. That key is the documented backing store the
/// `SPI_GETWHEELSCROLLLINES` system parameter reports, and reading it needs only
/// the safe `winsafe` registry surface, so this host obtains the user's real
/// setting without `unsafe` while `unsafe_code = "forbid"` stands.
///
/// The read is a query and never a write: no test and no code path here mutates
/// the reader's registry.
#[cfg(target_os = "windows")]
pub(crate) fn read_wheel_scroll_lines_setting() -> UiNativeWheelScrollLinesSetting {
    let read = winsafe::HKEY::CURRENT_USER.RegGetValue(
        Some(r"Control Panel\Desktop"),
        Some("WheelScrollLines"),
        winsafe::co::RRF::RT_ANY,
    );
    match read {
        Err(_) => UiNativeWheelScrollLinesSetting::Absent,
        Ok(winsafe::RegistryValue::Sz(text) | winsafe::RegistryValue::ExpandSz(text)) => {
            UiNativeWheelScrollLinesSetting::Stated(text)
        }
        Ok(winsafe::RegistryValue::Dword(count)) => {
            UiNativeWheelScrollLinesSetting::Stated(count.to_string())
        }
        Ok(winsafe::RegistryValue::None) => UiNativeWheelScrollLinesSetting::Absent,
        Ok(_) => UiNativeWheelScrollLinesSetting::Unreadable,
    }
}

/// No platform but Windows publishes a wheel-lines setting this host can read.
#[cfg(not(target_os = "windows"))]
pub(crate) const fn read_wheel_scroll_lines_setting() -> UiNativeWheelScrollLinesSetting {
    UiNativeWheelScrollLinesSetting::Absent
}
