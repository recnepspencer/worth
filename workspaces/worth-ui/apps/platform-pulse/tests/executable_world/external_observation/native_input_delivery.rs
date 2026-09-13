use super::{NativeWindowIdentity, ProcessBoundNativeClientAreaObservation};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeInputProbeKind {
    Pointer,
    Keyboard,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeKeyboardCommand {
    Escape,
    Tab,
    PrimaryShiftP,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeInputDeliveryObservation {
    kind: NativeInputProbeKind,
    process_id: u32,
    window: NativeWindowIdentity,
    qualified_screen_x: i32,
    qualified_screen_y: i32,
    delivered_event_count: u32,
}

impl NativeInputDeliveryObservation {
    pub(crate) fn for_client(
        kind: NativeInputProbeKind,
        client: ProcessBoundNativeClientAreaObservation,
        qualified_screen_point: (i32, i32),
        delivered_event_count: u32,
    ) -> Self {
        Self {
            kind,
            process_id: client.process_id(),
            window: client.window(),
            qualified_screen_x: qualified_screen_point.0,
            qualified_screen_y: qualified_screen_point.1,
            delivered_event_count,
        }
    }

    pub(crate) fn kind(self) -> NativeInputProbeKind {
        self.kind
    }

    pub(crate) fn process_id(self) -> u32 {
        self.process_id
    }

    pub(crate) fn window(self) -> NativeWindowIdentity {
        self.window
    }

    pub(crate) fn qualified_screen_point(self) -> (i32, i32) {
        (self.qualified_screen_x, self.qualified_screen_y)
    }

    pub(crate) fn delivered_event_count(self) -> u32 {
        self.delivered_event_count
    }
}
