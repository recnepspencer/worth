use std::collections::BTreeSet;

/// Public exposure decision for a registry family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegistryFamilyFacadeExposure {
    /// The registry family is intentionally available through `worth_ui::facade`.
    PublicFacade,
    /// The registry family is intentionally kept behind internal boundaries.
    InternalOnly,
}

/// Lifecycle propagation contract for a registry family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegistryFamilyLifecyclePropagation {
    /// The family must propagate through builder, diagnostics, and snapshot freeze.
    FullRegistryLifecycle,
}

/// Report produced when auditing reported family names against the registry inventory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryFamilyInventoryAudit {
    omitted_families: Vec<RegistryFamily>,
    unknown_family_names: Vec<String>,
    duplicate_family_names: Vec<String>,
}

impl RegistryFamilyInventoryAudit {
    pub fn from_reported_family_names(
        family_names: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Self {
        let reported_family_names = family_names
            .into_iter()
            .map(|family_name| family_name.as_ref().to_owned())
            .collect::<Vec<_>>();
        let omitted_families = omitted_registry_families(&reported_family_names);
        let unknown_family_names = unknown_registry_family_names(&reported_family_names);
        let duplicate_family_names = duplicate_registry_family_names(&reported_family_names);

        Self {
            omitted_families,
            unknown_family_names,
            duplicate_family_names,
        }
    }

    pub fn omitted_families(&self) -> &[RegistryFamily] {
        &self.omitted_families
    }

    pub fn unknown_family_names(&self) -> &[String] {
        &self.unknown_family_names
    }

    pub fn duplicate_family_names(&self) -> &[String] {
        &self.duplicate_family_names
    }

    pub fn is_complete(&self) -> bool {
        self.omitted_families.is_empty()
            && self.unknown_family_names.is_empty()
            && self.duplicate_family_names.is_empty()
    }
}

macro_rules! define_registry_families {
    ($(
        $variant:ident => {
            name: $name:literal,
            facade_exposure: $facade_exposure:ident,
            lifecycle: $lifecycle:ident,
        },
    )*) => {
        /// Typed inventory of capability registry families known to Worth UI.
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub enum RegistryFamily {
            $($variant,)*
        }

        impl RegistryFamily {
            /// Every registry family that must propagate through lifecycle boundaries.
            pub const ALL: &'static [Self] = &[
                $(Self::$variant,)*
            ];

            /// Every registry family that must propagate through lifecycle boundaries.
            pub fn all() -> &'static [Self] {
                Self::ALL
            }

            /// Stable family name used by registration, diagnostics, and reports.
            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)*
                }
            }

            /// Explicit facade exposure decision for this registry family.
            pub const fn facade_exposure(self) -> RegistryFamilyFacadeExposure {
                match self {
                    $(
                        Self::$variant => RegistryFamilyFacadeExposure::$facade_exposure,
                    )*
                }
            }

            /// Explicit lifecycle propagation contract for this registry family.
            pub const fn lifecycle_propagation(self) -> RegistryFamilyLifecyclePropagation {
                match self {
                    $(
                        Self::$variant => RegistryFamilyLifecyclePropagation::$lifecycle,
                    )*
                }
            }

            /// Resolve an external family name into the typed inventory, if known.
            pub fn from_name(family_name: &str) -> Option<Self> {
                match family_name {
                    $($name => Some(Self::$variant),)*
                    _ => None,
                }
            }

            /// Whether the builder must own initialized storage for this family.
            pub const fn requires_builder_initialization(self) -> bool {
                match self.lifecycle_propagation() {
                    RegistryFamilyLifecyclePropagation::FullRegistryLifecycle => true,
                }
            }

            /// Whether snapshot freeze must report this family.
            pub const fn requires_snapshot_freeze(self) -> bool {
                match self.lifecycle_propagation() {
                    RegistryFamilyLifecyclePropagation::FullRegistryLifecycle => true,
                }
            }

            /// Whether registration diagnostics must aggregate this family.
            pub const fn requires_diagnostics_aggregation(self) -> bool {
                match self.lifecycle_propagation() {
                    RegistryFamilyLifecyclePropagation::FullRegistryLifecycle => true,
                }
            }
        }
    };
}

define_registry_families! {
    AppearanceRole => {
        name: "appearance_role",
        facade_exposure: PublicFacade,
        lifecycle: FullRegistryLifecycle,
    },
    AppearanceTheme => {
        name: "appearance_theme",
        facade_exposure: InternalOnly,
        lifecycle: FullRegistryLifecycle,
    },
    MosaicSeamPaint => {
        name: "mosaic_seam_paint",
        facade_exposure: InternalOnly,
        lifecycle: FullRegistryLifecycle,
    },
    Command => {
        name: "command",
        facade_exposure: PublicFacade,
        lifecycle: FullRegistryLifecycle,
    },
    CommandProjection => {
        name: "command_projection",
        facade_exposure: PublicFacade,
        lifecycle: FullRegistryLifecycle,
    },
    Component => {
        name: "component",
        facade_exposure: PublicFacade,
        lifecycle: FullRegistryLifecycle,
    },
    Icon => {
        name: "icon",
        facade_exposure: PublicFacade,
        lifecycle: FullRegistryLifecycle,
    },
    IntentDefinition => {
        name: "intent_definition",
        facade_exposure: PublicFacade,
        lifecycle: FullRegistryLifecycle,
    },
    MosaicPlacementPolicy => {
        name: "mosaic_placement_policy",
        facade_exposure: PublicFacade,
        lifecycle: FullRegistryLifecycle,
    },
    MosaicRegionKind => {
        name: "mosaic_region_kind",
        facade_exposure: PublicFacade,
        lifecycle: FullRegistryLifecycle,
    },
    MosaicSizingContract => {
        name: "mosaic_sizing_contract",
        facade_exposure: PublicFacade,
        lifecycle: FullRegistryLifecycle,
    },
    MosaicStateSlot => {
        name: "mosaic_state_slot",
        facade_exposure: PublicFacade,
        lifecycle: FullRegistryLifecycle,
    },
    NativeCapability => {
        name: "native_capability",
        facade_exposure: PublicFacade,
        lifecycle: FullRegistryLifecycle,
    },
    PluginSlot => {
        name: "plugin_slot",
        facade_exposure: PublicFacade,
        lifecycle: FullRegistryLifecycle,
    },
    RuntimeOutcomeProjection => {
        name: "runtime_outcome_projection",
        facade_exposure: PublicFacade,
        lifecycle: FullRegistryLifecycle,
    },
    Setting => {
        name: "setting",
        facade_exposure: PublicFacade,
        lifecycle: FullRegistryLifecycle,
    },
    Surface => {
        name: "surface",
        facade_exposure: PublicFacade,
        lifecycle: FullRegistryLifecycle,
    },
    TaskPresentation => {
        name: "task_presentation",
        facade_exposure: PublicFacade,
        lifecycle: FullRegistryLifecycle,
    },
    ThemeToken => {
        name: "theme_token",
        facade_exposure: PublicFacade,
        lifecycle: FullRegistryLifecycle,
    },
    ViewBinding => {
        name: "view_binding",
        facade_exposure: PublicFacade,
        lifecycle: FullRegistryLifecycle,
    },
}

fn omitted_registry_families(reported_family_names: &[String]) -> Vec<RegistryFamily> {
    RegistryFamily::all()
        .iter()
        .copied()
        .filter(|registry_family| {
            !reported_family_names
                .iter()
                .any(|family_name| family_name == registry_family.name())
        })
        .collect()
}

fn unknown_registry_family_names(reported_family_names: &[String]) -> Vec<String> {
    reported_family_names
        .iter()
        .filter(|family_name| RegistryFamily::from_name(family_name).is_none())
        .cloned()
        .collect()
}

fn duplicate_registry_family_names(reported_family_names: &[String]) -> Vec<String> {
    let mut sorted_family_names = reported_family_names.to_vec();
    sorted_family_names.sort();
    let duplicate_names = sorted_family_names
        .windows(2)
        .filter(|family_name_pair| family_name_pair[0] == family_name_pair[1])
        .map(|family_name_pair| family_name_pair[0].clone())
        .collect::<BTreeSet<_>>();

    duplicate_names.into_iter().collect()
}
