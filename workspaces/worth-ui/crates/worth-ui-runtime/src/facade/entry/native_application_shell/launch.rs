use super::{
    NativeMountedRow, WorthUiNativeApplicationCleanup, WorthUiNativeApplicationShell,
    WorthUiNativeApplicationShellLaunchDenial,
};
use crate::facade::entry::WorthUiApp;
use crate::facade::mounted::{
    UiHostSurfacePresentationMode, UiSurfaceBindingCoordinatePosture, UiSurfaceBindingGeneration,
    UiSurfaceBindingProfile,
};
use std::collections::HashMap;

struct ConfiguredNativeSurface {
    binding: UiSurfaceBindingGeneration,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    mounted_rows: Vec<NativeMountedRow>,
    mounted_row_indices: HashMap<Box<str>, usize>,
}

struct NativeSurfaceConfigurationFailure {
    cause: WorthUiNativeApplicationShellLaunchDenial,
    expected_released_surface_count: usize,
}

impl WorthUiApp {
    /// Launch one native surface without exposing mounted construction authority.
    pub fn launch_native_surface(
        self,
    ) -> Result<WorthUiNativeApplicationShell, WorthUiNativeApplicationShellLaunchDenial> {
        self.launch_native_surface_at_scale(1_000)
    }

    #[doc(hidden)]
    pub fn launch_native_surface_at_scale(
        self,
        scale_factor_milli: u32,
    ) -> Result<WorthUiNativeApplicationShell, WorthUiNativeApplicationShellLaunchDenial> {
        self.launch_native_surface_at_scale_for(scale_factor_milli, None)
    }

    pub(crate) fn launch_native_surface_at_scale_for(
        self,
        scale_factor_milli: u32,
        native_surface_declaration: Option<&str>,
    ) -> Result<WorthUiNativeApplicationShell, WorthUiNativeApplicationShellLaunchDenial> {
        let mut session = self.launch().map_err(|denial| match denial {
            crate::runtime::WorthUiRuntimeLaunchDenial::HostSessionReleaseIndeterminate {
                ..
            }
            | crate::runtime::WorthUiRuntimeLaunchDenial::HostSessionReleaseMismatch { .. } => {
                WorthUiNativeApplicationShellLaunchDenial::RuntimeLaunchCleanup(Box::new(denial))
            }
            _ => WorthUiNativeApplicationShellLaunchDenial::RuntimeLaunch,
        })?;
        let configured = match configure_native_surface(
            &mut session,
            scale_factor_milli,
            native_surface_declaration,
        ) {
            Ok(configured) => configured,
            Err(failure) => {
                let client_resource_peaks = session.mounted.native_client_resource_peaks();
                let mut cleanup = session.shutdown();
                return Err(
                    if launch_cleanup_complete(&cleanup, failure.expected_released_surface_count) {
                        failure.cause
                    } else {
                        WorthUiNativeApplicationShellLaunchDenial::ApplicationCleanup(Box::new(
                            WorthUiNativeApplicationCleanup {
                                host_cleanup: cleanup.take_host_session_recovery(),
                                presentation_async_cleanup: cleanup
                                    .take_presentation_async_cleanup(),
                                query_close: Box::new(
                                    super::query_close::UiNativeApplicationQueryCloseInput {
                                        closed_resources: cleanup
                                            .mounted_presentation()
                                            .closed_query_resources(),
                                        transitions: cleanup
                                            .mounted_presentation()
                                            .query_transitions()
                                            .to_vec()
                                            .into_boxed_slice(),
                                        semantic_frontiers: cleanup
                                            .mounted_presentation()
                                            .query_semantic_frontiers()
                                            .to_vec()
                                            .into_boxed_slice(),
                                        semantic_frontier_trace_complete: cleanup
                                            .mounted_presentation()
                                            .query_semantic_frontier_trace_complete(),
                                        text_work: cleanup
                                            .mounted_presentation()
                                            .text_presentation_work()
                                            .to_vec()
                                            .into_boxed_slice(),
                                        text_work_trace_complete: cleanup
                                            .mounted_presentation()
                                            .text_presentation_work_trace_complete(),
                                        authored_mounted_instances: Box::new([]),
                                        client_resource_peaks,
                                        mounted_shutdown_attempts: cleanup
                                            .mounted_presentation()
                                            .attempts()
                                            .to_vec()
                                            .into_boxed_slice(),
                                        intent_resources_empty: cleanup
                                            .intent_resource_census()
                                            .is_empty(),
                                        query_close_complete: cleanup
                                            .mounted_presentation()
                                            .query_close_complete(),
                                        transition_trace_complete: cleanup
                                            .mounted_presentation()
                                            .query_transition_trace_complete(),
                                    },
                                ),
                            },
                        ))
                    },
                );
            }
        };
        Ok(WorthUiNativeApplicationShell {
            session: Box::new(session),
            binding: configured.binding,
            surface: configured.surface,
            scale_factor_milli,
            mounted_rows: configured.mounted_rows,
            mounted_row_indices: configured.mounted_row_indices,
            observed_viewport_basis: None,
            pending_viewport_basis: None,
            pending_surface_reconciliation: None,
            runtime_derived_state_reconstruction: None,
            pending_managed_rebind: None,
            retained_portal_dismissal: None,
            pending_component_presence: None,
            managed_rebind_completion_tick: 0,
            reduced_motion_posture: super::WorthUiNativeReducedMotionPosture::Unavailable,
        })
    }
}

fn configure_native_surface(
    session: &mut crate::facade::entry::WorthUiActiveApplicationSession,
    scale_factor_milli: u32,
    native_surface_declaration: Option<&str>,
) -> Result<ConfiguredNativeSurface, NativeSurfaceConfigurationFailure> {
    let surface = match native_surface_declaration {
        Some(authored_name) => {
            let declaration = session
                .application
                .authored_overlay_material()
                .overlay_declaration_bindings()
                .surface_named(authored_name)
                .ok_or_else(|| {
                    configuration_failure(
                        WorthUiNativeApplicationShellLaunchDenial::SemanticSurfaceCreation,
                        0,
                    )
                })?;
            session
                .create_declared_semantic_surface(declaration)
                .map_err(|_| {
                    configuration_failure(
                        WorthUiNativeApplicationShellLaunchDenial::SemanticSurfaceCreation,
                        0,
                    )
                })?
        }
        None => session.create_semantic_surface().map_err(|_| {
            configuration_failure(
                WorthUiNativeApplicationShellLaunchDenial::SemanticSurfaceCreation,
                0,
            )
        })?,
    };
    let profile = UiSurfaceBindingProfile::new(
        scale_factor_milli,
        UiSurfaceBindingCoordinatePosture::LogicalPoints,
        1,
    )
    .map_err(|_| {
        configuration_failure(
            WorthUiNativeApplicationShellLaunchDenial::HostSurfaceRegistration,
            0,
        )
    })?;
    let binding = session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::NativeDisplay,
            profile,
        )
        .map_err(|_| {
            configuration_failure(
                WorthUiNativeApplicationShellLaunchDenial::HostSurfaceRegistration,
                0,
            )
        })?;
    let graph_nodes = {
        let graph = session.graph();
        graph
            .node_identities()
            .filter_map(|identity| {
                let lookup = graph.lookup().graph_node(identity)?;
                let semantic = lookup
                    .value()
                    .declaration_identity()
                    .authored_semantic_name()
                    .to_owned();
                semantic
                    .starts_with("component:")
                    .then(|| (identity, Box::<str>::from(semantic)))
            })
            .collect::<Vec<_>>()
    };
    let mut mounted_rows = Vec::with_capacity(graph_nodes.len());
    let mut mounted_row_indices = HashMap::with_capacity(graph_nodes.len());
    for (graph_node, authored_semantic_identity) in graph_nodes {
        session
            .register_application_semantic_text(authored_semantic_identity.clone(), graph_node)
            .map_err(|_| {
                configuration_failure(
                    WorthUiNativeApplicationShellLaunchDenial::MountedInstanceCreation,
                    1,
                )
            })?;
        let handle = session.mounted_graph_node(graph_node).map_err(|_| {
            configuration_failure(
                WorthUiNativeApplicationShellLaunchDenial::MountedInstanceCreation,
                1,
            )
        })?;
        let mounted = session.mount_instance(handle, surface).map_err(|_| {
            configuration_failure(
                WorthUiNativeApplicationShellLaunchDenial::MountedInstanceCreation,
                1,
            )
        })?;
        let index = mounted_rows.len();
        if mounted_row_indices
            .insert(authored_semantic_identity.clone(), index)
            .is_some()
        {
            return Err(configuration_failure(
                WorthUiNativeApplicationShellLaunchDenial::MountedInstanceCreation,
                1,
            ));
        }
        mounted_rows.push(NativeMountedRow {
            authored_semantic_identity,
            graph_node,
            mounted: Some(mounted),
            latest_mounted: mounted,
        });
    }
    session
        .establish_native_viewport_allocation()
        .map_err(|denial| {
            configuration_failure(
                WorthUiNativeApplicationShellLaunchDenial::ViewportAllocation(Box::new(denial)),
                1,
            )
        })?;
    Ok(ConfiguredNativeSurface {
        binding: binding.binding_generation(),
        surface,
        mounted_rows,
        mounted_row_indices,
    })
}

fn configuration_failure(
    cause: WorthUiNativeApplicationShellLaunchDenial,
    expected_released_surface_count: usize,
) -> NativeSurfaceConfigurationFailure {
    NativeSurfaceConfigurationFailure {
        cause,
        expected_released_surface_count,
    }
}

fn launch_cleanup_complete(
    receipt: &crate::runtime::WorthUiRuntimeShutdownReceipt,
    expected_released_surface_count: usize,
) -> bool {
    matches!(
        receipt.host_session_release(),
        Some(worth_ui_host_contract::UiHostSessionReleaseOutcome::Released(released))
            if released.released_surface_count() == expected_released_surface_count
    ) && receipt.mounted_presentation().query_close_complete()
}
