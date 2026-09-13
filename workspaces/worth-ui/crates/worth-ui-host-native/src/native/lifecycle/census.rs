macro_rules! native_resource_schema {
    ($( $variant:ident => $field:ident ),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub(crate) enum UiNativeResourceClass { $( $variant ),+ }

        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        pub struct UiNativeResourceCensus { $( pub $field: usize ),+ }

        impl UiNativeResourceClass {
            pub(crate) const fn all() -> &'static [Self] {
                &[$(Self::$variant),+]
            }

            const fn field_name(self) -> &'static str {
                match self { $( Self::$variant => stringify!($field) ),+ }
            }
        }

        impl UiNativeResourceCensus {
            pub fn entries(self) -> impl Iterator<Item = (&'static str, usize)> {
                [$( (stringify!($field), self.$field) ),+].into_iter()
            }

            pub fn field_names() -> impl Iterator<Item = &'static str> {
                UiNativeResourceClass::all().iter().copied().map(UiNativeResourceClass::field_name)
            }

            pub(crate) fn record(&mut self, class: UiNativeResourceClass) {
                match class { $( UiNativeResourceClass::$variant => self.$field += 1 ),+ }
            }

            pub(crate) fn max(self, other: Self) -> Self {
                Self { $( $field: self.$field.max(other.$field) ),+ }
            }
        }
    };
}

native_resource_schema! {
    Window => windows,
    Surface => surfaces,
    Adapter => adapters,
    Device => devices,
    Queue => queues,
    RetainedTarget => retained_targets,
    HostRegistration => registrations,
    ReadbackBuffer => readback_buffers,
    PendingSubmission => pending_submissions,
    EventWakeRegistration => event_wake_registrations,
    ApplicationDriver => application_drivers,
    AlphaAtlasPage => alpha_atlas_pages,
    ColorAtlasPage => color_atlas_pages,
    AtlasStagingBuffer => atlas_staging_buffers,
    TextAtlasPlan => text_atlas_plans,
    TextAtlasReservation => text_atlas_reservations,
    TextAtlasPin => text_atlas_pins,
    TextAtlasRecovery => text_atlas_recoveries,
    TextAtlasAlphaEntry => text_atlas_alpha_entries,
    TextAtlasColorEntry => text_atlas_color_entries,
    TextAtlasUploadSubmission => text_atlas_upload_submissions,
    TextAtlasInFlightTransaction => text_atlas_in_flight_transactions,
    TextAtlasRecoveryAuthority => text_atlas_recovery_authorities,
    PhysicalSignalRuntime => physical_signal_runtimes,
    PhysicalSignalWorker => physical_signal_workers,
    PhysicalSignalPendingWork => physical_signal_pending_work,
    PhysicalSignalWake => physical_signal_wakes,
    PhysicalSignalTransitionObservation => physical_signal_transition_observations,
    PendingPresentation => pending_presentations,
    PendingPresentationSettlement => pending_presentation_settlements,
    RetainedDrawList => retained_draw_lists,
    PresentationEpoch => presentation_epochs,
    ReconstructionRequirement => reconstruction_requirements,
    TextPinBinding => text_pin_bindings,
    PendingTextPresentation => pending_text_presentations,
    RetainedFrameObservation => retained_frame_observations,
    TextPinFrameObservation => text_pin_frame_observations,
    TextAtlasPlanObservation => text_atlas_plan_observations,
    ClientMountedLayout => client_mounted_layouts,
    ClientRasterCacheEntry => client_raster_cache_entries,
}

impl UiNativeResourceCensus {
    pub fn is_zero(self) -> bool {
        self.entries().all(|(_, count)| count == 0)
    }

    pub(crate) fn with_text_atlas(
        mut self,
        atlas: crate::native::text_atlas::UiNativeTextAtlasCensus,
    ) -> Self {
        use crate::native::text_atlas::UiNativeTextAtlasResourceClass as Class;
        for (class, count) in atlas.classified_entries() {
            match class {
                Class::Plan => self.text_atlas_plans = self.text_atlas_plans.max(count),
                Class::Reservation => {
                    self.text_atlas_reservations = self.text_atlas_reservations.max(count)
                }
                Class::Pin => self.text_atlas_pins = self.text_atlas_pins.max(count),
                Class::Recovery => {
                    self.text_atlas_recoveries = self.text_atlas_recoveries.max(count)
                }
                Class::AlphaPage => self.alpha_atlas_pages = self.alpha_atlas_pages.max(count),
                Class::ColorPage => self.color_atlas_pages = self.color_atlas_pages.max(count),
                Class::AlphaEntry => {
                    self.text_atlas_alpha_entries = self.text_atlas_alpha_entries.max(count)
                }
                Class::ColorEntry => {
                    self.text_atlas_color_entries = self.text_atlas_color_entries.max(count)
                }
                Class::StagingBuffer => {
                    self.atlas_staging_buffers = self.atlas_staging_buffers.max(count)
                }
                Class::UploadSubmission => {
                    self.text_atlas_upload_submissions =
                        self.text_atlas_upload_submissions.max(count)
                }
                Class::InFlightTransaction => {
                    self.text_atlas_in_flight_transactions =
                        self.text_atlas_in_flight_transactions.max(count)
                }
                Class::RecoveryAuthority => {
                    self.text_atlas_recovery_authorities =
                        self.text_atlas_recovery_authorities.max(count)
                }
            }
        }
        self
    }

    pub(crate) fn with_physical_signal(
        mut self,
        signal: crate::native::physical_work_signal::UiNativePhysicalSignalObservation,
    ) -> Self {
        let active = usize::from(signal.runtime_owned);
        self.physical_signal_runtimes = self.physical_signal_runtimes.max(active);
        self.physical_signal_workers = self.physical_signal_workers.max(active);
        self.physical_signal_pending_work = self
            .physical_signal_pending_work
            .max(signal.active_requests);
        self.physical_signal_wakes = self.physical_signal_wakes.max(signal.pending_wakes);
        self.physical_signal_transition_observations = self
            .physical_signal_transition_observations
            .max(signal.retained_transition_observations);
        self
    }

    pub(crate) fn with_host_state(mut self, state: &crate::native::UiNativeHostState) -> Self {
        self.pending_presentations = self
            .pending_presentations
            .max(state.pending_presentations.len());
        self.pending_presentation_settlements = self.pending_presentation_settlements.max(
            state
                .pending_presentations
                .iter()
                .filter(|pending| pending.has_settlement())
                .count(),
        );
        self.retained_draw_lists = self
            .retained_draw_lists
            .max(state.retained_draw_lists.len());
        self.presentation_epochs = self
            .presentation_epochs
            .max(state.presentation_epochs.len());
        self.reconstruction_requirements = self
            .reconstruction_requirements
            .max(state.lifecycle.recovery_count());
        self.text_pin_bindings = self.text_pin_bindings.max(state.text_pins_by_binding.len());
        self.pending_text_presentations = self
            .pending_text_presentations
            .max(state.pending_text_presentations.len());
        self.retained_frame_observations = self.retained_frame_observations.max(
            state.retained_frame_observations.len()
                + usize::from(state.last_retained_frame.is_some()),
        );
        self.text_pin_frame_observations = self
            .text_pin_frame_observations
            .max(state.text_pin_frame_observations.len());
        self.text_atlas_plan_observations = self
            .text_atlas_plan_observations
            .max(state.text_atlas_plan_observations.len());
        self
    }

    pub(crate) fn with_client_peak(
        mut self,
        client: crate::native::UiNativeClientResourceObservation,
    ) -> Self {
        self.client_mounted_layouts = self
            .client_mounted_layouts
            .max(client.peak_mounted_layouts());
        self.client_raster_cache_entries = self
            .client_raster_cache_entries
            .max(client.peak_raster_cache_entries());
        self
    }

    pub(crate) fn with_client_terminal(
        mut self,
        client: crate::native::UiNativeClientResourceObservation,
    ) -> Self {
        self.client_mounted_layouts = client.terminal_mounted_layouts();
        self.client_raster_cache_entries = client.terminal_raster_cache_entries();
        self
    }
}
