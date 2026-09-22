use worth_ui_host_contract::{
    UiHostImePreeditConstructionDenial, UiHostObservationBatchConstructionDenial,
    UiHostObservationPresentationBasis, UiHostObservationRetentionDenial,
    UiHostSurfaceCoordinateSpace, UiHostSurfaceCoordinateUnit,
};

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeInputObservationEventFamily {
    Pointer,
    Keyboard,
    WindowFocus,
    Text,
    Ime,
    Scroll,
    Gesture,
    Touch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeInputObservationStop {
    NoPresentationBasis,
    MissingInputRecipientAffinity,
    StaleInputRecipientAffinity,
    StalePresentationAffinity,
    OverCapacityText,
    MissingEventProfile,
    InvalidScale,
    CoordinateNotFinite,
    CoordinateOutOfRange,
    PointerPositionUnavailable,
    Unsupported(UiNativeInputObservationEventFamily),
    UnsupportedKey,
    ImeRangeNotScalarBoundary,
    ImePreedit(UiHostImePreeditConstructionDenial),
    TextRevisionExhausted,
    PointerCaptureEpochExhausted,
    CompletedPresentationCountExhausted,
    ObservationSequenceExhausted,
    BatchConstruction(UiHostObservationBatchConstructionDenial),
    Retention(UiHostObservationRetentionDenial),
    MissingPendingPresentationContext,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiNativePointerButtonObservation {
    sequence: u64,
    event_tick: u64,
    x_subpixels: i64,
    y_subpixels: i64,
    coordinate_space: UiHostSurfaceCoordinateSpace,
    coordinate_unit: UiHostSurfaceCoordinateUnit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiNativeScrollDeltaObservation {
    sequence: u64,
    event_tick: u64,
    x_subpixels: i64,
    y_subpixels: i64,
    precision: worth_ui_host_contract::UiHostScrollDeltaPrecision,
}

impl UiNativePointerButtonObservation {
    pub(super) const fn reported(
        sequence: u64,
        event_tick: u64,
        position: worth_ui_host_contract::UiHostSurfacePosition,
    ) -> Self {
        Self {
            sequence,
            event_tick,
            x_subpixels: position.x_subpixels(),
            y_subpixels: position.y_subpixels(),
            coordinate_space: position.basis().coordinate_space(),
            coordinate_unit: position.basis().coordinate_unit(),
        }
    }

    pub const fn sequence(self) -> u64 {
        self.sequence
    }

    pub const fn event_tick(self) -> u64 {
        self.event_tick
    }

    pub const fn x_subpixels(self) -> i64 {
        self.x_subpixels
    }

    pub const fn y_subpixels(self) -> i64 {
        self.y_subpixels
    }

    pub const fn coordinate_space(self) -> UiHostSurfaceCoordinateSpace {
        self.coordinate_space
    }

    pub const fn coordinate_unit(self) -> UiHostSurfaceCoordinateUnit {
        self.coordinate_unit
    }
}

impl UiNativeScrollDeltaObservation {
    pub(super) const fn reported(
        sequence: u64,
        event_tick: u64,
        x_subpixels: i64,
        y_subpixels: i64,
        precision: worth_ui_host_contract::UiHostScrollDeltaPrecision,
    ) -> Self {
        Self {
            sequence,
            event_tick,
            x_subpixels,
            y_subpixels,
            precision,
        }
    }

    /// The platform line count one notch carried, when the delta was line-denominated.
    pub const fn lines_per_notch(self) -> Option<u16> {
        self.precision.lines_per_notch()
    }

    /// Where that line count came from, when the delta was line-denominated.
    pub const fn line_count_basis(
        self,
    ) -> Option<worth_ui_host_contract::UiHostScrollLineCountBasis> {
        self.precision.line_count_basis()
    }

    pub const fn sequence(self) -> u64 {
        self.sequence
    }

    pub const fn event_tick(self) -> u64 {
        self.event_tick
    }

    pub const fn x_subpixels(self) -> i64 {
        self.x_subpixels
    }

    pub const fn y_subpixels(self) -> i64 {
        self.y_subpixels
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiNativeInputObservationReport {
    pub(super) last_completed_presentation: Option<UiHostObservationPresentationBasis>,
    pub(super) completed_presentation_count: usize,
    pub(super) stops: Box<[UiNativeInputObservationStop]>,
    pub(super) terminal_stop: Option<UiNativeInputObservationStop>,
    pub(super) stop_history_complete: bool,
    pub(super) retained_batch_count: u64,
    pub(super) coalesced_motion_batch_count: u64,
    pub(super) retained_event_count: u64,
    pub(super) first_retained_sequence: Option<u64>,
    pub(super) last_retained_sequence: Option<u64>,
    pub(super) family_counts: [u64; 11],
    pub(super) last_pointer_button: Option<UiNativePointerButtonObservation>,
    pub(super) last_vertical_scroll: Option<UiNativeScrollDeltaObservation>,
    pub(super) last_horizontal_scroll: Option<UiNativeScrollDeltaObservation>,
    pub(super) profile_transition_count: u64,
}

impl UiNativeInputObservationReport {
    pub const fn last_completed_presentation(&self) -> Option<UiHostObservationPresentationBasis> {
        self.last_completed_presentation
    }

    pub const fn completed_presentation_count(&self) -> usize {
        self.completed_presentation_count
    }

    pub fn stops(&self) -> &[UiNativeInputObservationStop] {
        &self.stops
    }

    pub const fn terminal_stop(&self) -> Option<UiNativeInputObservationStop> {
        self.terminal_stop
    }

    pub const fn stop_history_complete(&self) -> bool {
        self.stop_history_complete
    }

    pub const fn retained_batch_count(&self) -> u64 {
        self.retained_batch_count
    }

    /// Retained pointer-motion batches that replaced their retained predecessor
    /// instead of appending. The drain hands over `retained_batch_count` minus
    /// this many batches, so ingress accounting must add it back.
    pub const fn coalesced_motion_batch_count(&self) -> u64 {
        self.coalesced_motion_batch_count
    }

    pub const fn retained_event_count(&self) -> u64 {
        self.retained_event_count
    }

    pub const fn first_retained_sequence(&self) -> Option<u64> {
        self.first_retained_sequence
    }

    pub const fn last_retained_sequence(&self) -> Option<u64> {
        self.last_retained_sequence
    }

    pub const fn family_counts(&self) -> [u64; 11] {
        self.family_counts
    }

    pub const fn family_count(&self, family: UiNativeInputObservationEventFamily) -> u64 {
        match family.report_index() {
            Some(index) => self.family_counts[index],
            None => 0,
        }
    }

    pub const fn last_pointer_button(&self) -> Option<UiNativePointerButtonObservation> {
        self.last_pointer_button
    }

    pub const fn last_vertical_scroll(&self) -> Option<UiNativeScrollDeltaObservation> {
        self.last_vertical_scroll
    }

    pub const fn last_horizontal_scroll(&self) -> Option<UiNativeScrollDeltaObservation> {
        self.last_horizontal_scroll
    }

    pub const fn profile_transition_count(&self) -> u64 {
        self.profile_transition_count
    }
}

impl UiNativeInputObservationEventFamily {
    const fn report_index(self) -> Option<usize> {
        match self {
            Self::Pointer => Some(2),
            Self::Keyboard => Some(4),
            Self::WindowFocus => Some(5),
            Self::Text => Some(9),
            Self::Ime => Some(10),
            Self::Scroll => Some(6),
            Self::Gesture | Self::Touch => None,
        }
    }
}
