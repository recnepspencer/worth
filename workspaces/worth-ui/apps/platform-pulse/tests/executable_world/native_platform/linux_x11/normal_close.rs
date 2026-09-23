//! The ICCCM close request: a `WM_PROTOCOLS` client message carrying
//! `WM_DELETE_WINDOW`, which winit answers with `CloseRequested`. Without a
//! window manager the observer is the party that sends it. A window that
//! does not advertise the protocol would be destroyed by the server instead
//! of asked, so the advertisement is checked before anything is sent.
use x11rb::protocol::xproto::{
    AtomEnum, ClientMessageData, ClientMessageEvent, ConnectionExt as _, EventMask, Window,
    CLIENT_MESSAGE_EVENT,
};

use crate::external_observation::NormalNativeCloseRequestObservation;
use crate::native_platform::NativePlatformFailure;

use super::connection::X11Observation;

pub(super) fn request(
    x11: &X11Observation,
    window: Window,
    process_id: u32,
) -> Result<NormalNativeCloseRequestObservation, NativePlatformFailure> {
    let atoms = x11.atoms();
    let advertised = x11
        .connection()
        .get_property(false, window, atoms.wm_protocols, AtomEnum::ATOM, 0, 32)
        .map_err(close_failure)?
        .reply()
        .map_err(close_failure)?
        .value32()
        .is_some_and(|mut protocols| protocols.any(|atom| atom == atoms.wm_delete_window));
    if !advertised {
        return Err(NativePlatformFailure::NormalClose(
            "the bound window does not advertise WM_DELETE_WINDOW".to_owned(),
        ));
    }
    let event = ClientMessageEvent {
        response_type: CLIENT_MESSAGE_EVENT,
        format: 32,
        sequence: 0,
        window,
        type_: atoms.wm_protocols,
        data: ClientMessageData::from([atoms.wm_delete_window, 0, 0, 0, 0]),
    };
    x11.connection()
        .send_event(false, window, EventMask::NO_EVENT, event)
        .map_err(close_failure)?
        .check()
        .map_err(close_failure)?;
    Ok(NormalNativeCloseRequestObservation::one(process_id))
}

fn close_failure(error: impl std::fmt::Display) -> NativePlatformFailure {
    NativePlatformFailure::NormalClose(error.to_string())
}
