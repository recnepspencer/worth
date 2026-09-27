use crate::capability::{
    AdmittedCapability, CapabilityRegistrationDiagnostic, CapabilitySnapshot, ComponentDescriptor,
    ComponentId, FrozenThemeTokenEntry, FrozenViewBindingEntry, SupportRequirement,
    SurfaceDescriptor, SurfaceId, ThemeTokenId, ViewBindingId, COMPONENT_FAMILY_NAME,
    SURFACE_FAMILY_NAME, THEME_TOKEN_FAMILY_NAME, VIEW_BINDING_FAMILY_NAME,
};
use crate::source::{
    WorthUiResolutionDiagnostic, WorthUiResolutionDiagnosticCode, WorthUiResolutionMetrics,
};
use worth_ui_dsl::{WorthUiArtifactInputProvenance, WorthUiSourceModuleId};

pub(crate) struct WorthUiSnapshotResolutionContext<'snapshot> {
    snapshot: &'snapshot CapabilitySnapshot,
    metrics: WorthUiResolutionMetrics,
}

pub(crate) type ComponentResolution = (AdmittedCapability<ComponentId>, ComponentDescriptor);
pub(crate) type SurfaceResolution = (AdmittedCapability<SurfaceId>, SurfaceDescriptor);
pub(crate) type ViewBindingResolution = (AdmittedCapability<ViewBindingId>, FrozenViewBindingEntry);
pub(crate) type ThemeTokenResolution = (AdmittedCapability<ThemeTokenId>, FrozenThemeTokenEntry);

impl<'snapshot> WorthUiSnapshotResolutionContext<'snapshot> {
    pub(crate) fn new(snapshot: &'snapshot CapabilitySnapshot) -> Self {
        Self {
            snapshot,
            metrics: WorthUiResolutionMetrics::default(),
        }
    }

    pub(crate) fn finish_metrics(self) -> WorthUiResolutionMetrics {
        self.metrics
    }

    fn refusal(
        &self,
        family: &'static str,
        identity: &str,
    ) -> Option<&'snapshot [CapabilityRegistrationDiagnostic]> {
        self.snapshot
            .registration_rejections()
            .diagnostics(family, identity)
    }

    pub(crate) fn resolve_component(
        &mut self,
        module_id: &WorthUiSourceModuleId,
        authored_text: &str,
        provenance: &WorthUiArtifactInputProvenance,
    ) -> Result<ComponentResolution, WorthUiResolutionDiagnostic> {
        let component_id = parse_component_id(module_id, authored_text, provenance)?;
        let lookup = self.snapshot.index().components().lookup(&component_id);
        self.metrics.record_lookup(lookup.counters());
        if let Some(descriptor) = lookup.into_value() {
            let admitted_component = SupportRequirement::admitted()
                .check(
                    self.snapshot
                        .support_catalog()
                        .component_posture(&component_id)
                        .expect("support catalog should contain admitted component ids"),
                )
                .expect("admitted descriptor should satisfy admitted support requirement");
            return Ok((admitted_component, descriptor.clone()));
        }

        Err(component_diagnostic(
            self.snapshot
                .support_catalog()
                .component_posture(&component_id),
            module_id.clone(),
            authored_text,
            provenance.clone(),
        )
        .refused_at_registration(
            WorthUiResolutionDiagnosticCode::RejectedComponentReference,
            self.refusal(COMPONENT_FAMILY_NAME, component_id.as_str()),
        ))
    }

    pub(crate) fn resolve_surface(
        &mut self,
        module_id: &WorthUiSourceModuleId,
        authored_text: &str,
        provenance: &WorthUiArtifactInputProvenance,
    ) -> Result<SurfaceResolution, WorthUiResolutionDiagnostic> {
        let surface_id = parse_surface_id(module_id, authored_text, provenance)?;
        let lookup = self.snapshot.index().surfaces().lookup(&surface_id);
        self.metrics.record_lookup(lookup.counters());
        if let Some(descriptor) = lookup.into_value() {
            let admitted_surface = SupportRequirement::admitted()
                .check(
                    self.snapshot
                        .support_catalog()
                        .surface_posture(&surface_id)
                        .expect("support catalog should contain admitted surface ids"),
                )
                .expect("admitted descriptor should satisfy admitted support requirement");
            return Ok((admitted_surface, descriptor.clone()));
        }

        Err(surface_diagnostic(
            self.snapshot.support_catalog().surface_posture(&surface_id),
            module_id.clone(),
            authored_text,
            provenance.clone(),
        )
        .refused_at_registration(
            WorthUiResolutionDiagnosticCode::RejectedSurfaceReference,
            self.refusal(SURFACE_FAMILY_NAME, surface_id.as_str()),
        ))
    }

    pub(crate) fn resolve_view_binding(
        &mut self,
        module_id: &WorthUiSourceModuleId,
        authored_text: &str,
        provenance: &WorthUiArtifactInputProvenance,
    ) -> Result<ViewBindingResolution, WorthUiResolutionDiagnostic> {
        let view_binding_id = parse_view_binding_id(module_id, authored_text, provenance)?;
        let lookup = self
            .snapshot
            .index()
            .view_bindings()
            .lookup(&view_binding_id);
        self.metrics.record_lookup(lookup.counters());
        if let Some(descriptor) = lookup.into_value() {
            let admitted_view_binding = SupportRequirement::admitted()
                .check(
                    self.snapshot
                        .support_catalog()
                        .view_binding_posture(&view_binding_id)
                        .expect("support catalog should contain admitted view binding ids"),
                )
                .expect("admitted descriptor should satisfy admitted support requirement");
            let entry = self
                .snapshot
                .view_bindings()
                .get_entry(descriptor.id())
                .expect("frozen view binding entry should exist for descriptor")
                .clone();
            return Ok((admitted_view_binding, entry));
        }

        Err(view_binding_diagnostic(
            self.snapshot
                .support_catalog()
                .view_binding_posture(&view_binding_id),
            module_id.clone(),
            authored_text,
            provenance.clone(),
        )
        .refused_at_registration(
            WorthUiResolutionDiagnosticCode::RejectedViewBindingReference,
            self.refusal(VIEW_BINDING_FAMILY_NAME, view_binding_id.as_str()),
        ))
    }

    pub(crate) fn resolve_theme_token(
        &mut self,
        module_id: &WorthUiSourceModuleId,
        authored_text: &str,
        provenance: &WorthUiArtifactInputProvenance,
    ) -> Result<ThemeTokenResolution, WorthUiResolutionDiagnostic> {
        let theme_token_id = parse_theme_token_id(module_id, authored_text, provenance)?;
        let lookup = self.snapshot.index().theme_tokens().lookup(&theme_token_id);
        self.metrics.record_lookup(lookup.counters());
        if let Some(descriptor) = lookup.into_value() {
            let admitted_theme_token = SupportRequirement::admitted()
                .check(
                    self.snapshot
                        .support_catalog()
                        .theme_token_posture(&theme_token_id)
                        .expect("support catalog should contain admitted theme token ids"),
                )
                .expect("admitted descriptor should satisfy admitted support requirement");
            let entry = self
                .snapshot
                .theme_tokens()
                .get_entry(descriptor.id())
                .expect("frozen theme token entry should exist for descriptor")
                .clone();
            return Ok((admitted_theme_token, entry));
        }

        Err(theme_token_diagnostic(
            self.snapshot
                .support_catalog()
                .theme_token_posture(&theme_token_id),
            module_id.clone(),
            authored_text,
            provenance.clone(),
        )
        .refused_at_registration(
            WorthUiResolutionDiagnosticCode::RejectedThemeTokenReference,
            self.refusal(THEME_TOKEN_FAMILY_NAME, theme_token_id.as_str()),
        ))
    }
}

fn parse_component_id(
    module_id: &WorthUiSourceModuleId,
    authored_text: &str,
    provenance: &WorthUiArtifactInputProvenance,
) -> Result<ComponentId, WorthUiResolutionDiagnostic> {
    ComponentId::new(authored_text).map_err(|_| {
        WorthUiResolutionDiagnostic::new(
            WorthUiResolutionDiagnosticCode::InvalidComponentReferenceId,
            module_id.clone(),
            authored_text,
            provenance.clone(),
        )
    })
}

fn parse_surface_id(
    module_id: &WorthUiSourceModuleId,
    authored_text: &str,
    provenance: &WorthUiArtifactInputProvenance,
) -> Result<SurfaceId, WorthUiResolutionDiagnostic> {
    SurfaceId::new(authored_text).map_err(|_| {
        WorthUiResolutionDiagnostic::new(
            WorthUiResolutionDiagnosticCode::InvalidSurfaceReferenceId,
            module_id.clone(),
            authored_text,
            provenance.clone(),
        )
    })
}

fn parse_view_binding_id(
    module_id: &WorthUiSourceModuleId,
    authored_text: &str,
    provenance: &WorthUiArtifactInputProvenance,
) -> Result<ViewBindingId, WorthUiResolutionDiagnostic> {
    ViewBindingId::new(authored_text).map_err(|_| {
        WorthUiResolutionDiagnostic::new(
            WorthUiResolutionDiagnosticCode::InvalidViewBindingReferenceId,
            module_id.clone(),
            authored_text,
            provenance.clone(),
        )
    })
}

fn parse_theme_token_id(
    module_id: &WorthUiSourceModuleId,
    authored_text: &str,
    provenance: &WorthUiArtifactInputProvenance,
) -> Result<ThemeTokenId, WorthUiResolutionDiagnostic> {
    ThemeTokenId::new(authored_text).map_err(|_| {
        WorthUiResolutionDiagnostic::new(
            WorthUiResolutionDiagnosticCode::InvalidThemeTokenReferenceId,
            module_id.clone(),
            authored_text,
            provenance.clone(),
        )
    })
}

fn component_diagnostic(
    posture: Option<crate::capability::CapabilitySupportPosture<ComponentId>>,
    module_id: WorthUiSourceModuleId,
    authored_text: &str,
    provenance: WorthUiArtifactInputProvenance,
) -> WorthUiResolutionDiagnostic {
    WorthUiResolutionDiagnostic::new(
        match posture {
            Some(posture) if posture.is_deferred() => {
                WorthUiResolutionDiagnosticCode::DeferredComponentReference
            }
            Some(posture) if posture.is_unsupported() => {
                WorthUiResolutionDiagnosticCode::UnsupportedComponentReference
            }
            Some(posture) if posture.is_platform_internal() => {
                WorthUiResolutionDiagnosticCode::PlatformInternalComponentReference
            }
            _ => WorthUiResolutionDiagnosticCode::MissingComponentReference,
        },
        module_id,
        authored_text,
        provenance,
    )
}

fn surface_diagnostic(
    posture: Option<crate::capability::CapabilitySupportPosture<SurfaceId>>,
    module_id: WorthUiSourceModuleId,
    authored_text: &str,
    provenance: WorthUiArtifactInputProvenance,
) -> WorthUiResolutionDiagnostic {
    WorthUiResolutionDiagnostic::new(
        match posture {
            Some(posture) if posture.is_deferred() => {
                WorthUiResolutionDiagnosticCode::DeferredSurfaceReference
            }
            Some(posture) if posture.is_unsupported() => {
                WorthUiResolutionDiagnosticCode::UnsupportedSurfaceReference
            }
            Some(posture) if posture.is_platform_internal() => {
                WorthUiResolutionDiagnosticCode::PlatformInternalSurfaceReference
            }
            _ => WorthUiResolutionDiagnosticCode::MissingSurfaceReference,
        },
        module_id,
        authored_text,
        provenance,
    )
}

fn view_binding_diagnostic(
    posture: Option<crate::capability::CapabilitySupportPosture<ViewBindingId>>,
    module_id: WorthUiSourceModuleId,
    authored_text: &str,
    provenance: WorthUiArtifactInputProvenance,
) -> WorthUiResolutionDiagnostic {
    WorthUiResolutionDiagnostic::new(
        match posture {
            Some(posture) if posture.is_deferred() => {
                WorthUiResolutionDiagnosticCode::DeferredViewBindingReference
            }
            Some(posture) if posture.is_unsupported() => {
                WorthUiResolutionDiagnosticCode::UnsupportedViewBindingReference
            }
            Some(posture) if posture.is_platform_internal() => {
                WorthUiResolutionDiagnosticCode::PlatformInternalViewBindingReference
            }
            _ => WorthUiResolutionDiagnosticCode::MissingViewBindingReference,
        },
        module_id,
        authored_text,
        provenance,
    )
}

fn theme_token_diagnostic(
    posture: Option<crate::capability::CapabilitySupportPosture<ThemeTokenId>>,
    module_id: WorthUiSourceModuleId,
    authored_text: &str,
    provenance: WorthUiArtifactInputProvenance,
) -> WorthUiResolutionDiagnostic {
    WorthUiResolutionDiagnostic::new(
        match posture {
            Some(posture) if posture.is_deferred() => {
                WorthUiResolutionDiagnosticCode::DeferredThemeTokenReference
            }
            Some(posture) if posture.is_unsupported() => {
                WorthUiResolutionDiagnosticCode::UnsupportedThemeTokenReference
            }
            Some(posture) if posture.is_platform_internal() => {
                WorthUiResolutionDiagnosticCode::PlatformInternalThemeTokenReference
            }
            _ => WorthUiResolutionDiagnosticCode::MissingThemeTokenReference,
        },
        module_id,
        authored_text,
        provenance,
    )
}
