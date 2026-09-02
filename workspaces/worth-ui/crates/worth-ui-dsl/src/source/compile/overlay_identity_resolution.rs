use std::collections::{BTreeMap, BTreeSet};

use crate::source::{
    WorthUiArtifactInputModule, WorthUiArtifactInputNode, WorthUiArtifactInputProvenance,
    WorthUiDslCompileDiagnostic, WorthUiDslCompileDiagnosticCode, WorthUiDslCompileReport,
    WorthUiDslCompileStopClass, WorthUiDslSourceSpan, WorthUiFileAuthoredLoweredDeclaration,
    WorthUiSourceModuleId,
};

struct OverlayIdentityTables {
    backdrops: BTreeMap<String, crate::UiBackdropIdentity>,
    portals: BTreeMap<String, crate::UiPortalDeclarationId>,
    surfaces: BTreeMap<String, crate::UiSemanticSurfaceDeclarationIdentity>,
    regions: BTreeMap<String, crate::UiMosaicRegionDeclarationIdentity>,
}

#[derive(Debug)]
enum ResolutionDenial {
    Duplicate(&'static str, String),
    Capacity(&'static str),
    Missing(&'static str, String),
    MissingRole(String),
    Backdrop(crate::UiBackdropDeclarationDenial),
}

pub(crate) fn resolve_file_authored_overlay_declarations(
    modules: BTreeMap<WorthUiSourceModuleId, Vec<WorthUiFileAuthoredLoweredDeclaration>>,
) -> Result<BTreeMap<WorthUiSourceModuleId, WorthUiArtifactInputModule>, WorthUiDslCompileReport> {
    let tables = collect_tables(&modules)
        .map_err(|denial| resolver_report(denial, first_provenance(&modules)))?;
    let roles = collect_roles(&modules);
    let mut resolved_modules = BTreeMap::new();
    for (module_id, declarations) in modules {
        let mut nodes = Vec::with_capacity(declarations.len());
        for declaration in declarations {
            match declaration {
                WorthUiFileAuthoredLoweredDeclaration::Artifact(node) => nodes.push(node),
                WorthUiFileAuthoredLoweredDeclaration::Backdrop { source, provenance } => {
                    let node = resolve_backdrop(&source, &tables, &roles)
                        .map_err(|denial| resolver_report(denial, Some(&provenance)))?;
                    nodes.push(WorthUiArtifactInputNode::Backdrop(
                        crate::WorthUiArtifactInputBackdropNode::new(node, provenance),
                    ));
                }
            }
        }
        resolved_modules.insert(
            module_id.clone(),
            WorthUiArtifactInputModule::new(module_id, nodes),
        );
    }
    Ok(resolved_modules)
}

pub(crate) fn resolve_portal_identity_names(
    names: impl IntoIterator<Item = String>,
) -> Result<Vec<crate::UiPortalDeclarationId>, ()> {
    let names = names.into_iter().collect::<Vec<_>>();
    let unique = names.iter().cloned().collect::<BTreeSet<_>>();
    if unique.len() != names.len() {
        return Err(());
    }
    allocate(unique, "portal", crate::UiPortalDeclarationId::new)
        .map(|identities| identities.into_values().collect())
        .map_err(|_| ())
}

fn collect_tables(
    modules: &BTreeMap<WorthUiSourceModuleId, Vec<WorthUiFileAuthoredLoweredDeclaration>>,
) -> Result<OverlayIdentityTables, ResolutionDenial> {
    let mut backdrop_names = BTreeSet::new();
    let mut portal_names = BTreeSet::new();
    let mut surface_names = BTreeSet::new();
    let mut region_names = BTreeSet::new();
    for declarations in modules.values() {
        for declaration in declarations {
            match declaration {
                WorthUiFileAuthoredLoweredDeclaration::Artifact(node) => match node {
                    WorthUiArtifactInputNode::Surface(surface) => {
                        insert_name(&mut surface_names, "surface", surface.name_text())?;
                    }
                    WorthUiArtifactInputNode::SemanticArtifact(artifact) => {
                        if let Some(crate::WorthUiServiceDeclarationMeaning::Portal(portal)) =
                            artifact.declaration().service_declaration()
                        {
                            insert_name(&mut portal_names, "portal", portal.identity())?;
                        }
                    }
                    _ => {}
                },
                WorthUiFileAuthoredLoweredDeclaration::Backdrop { source, .. } => {
                    insert_name(&mut backdrop_names, "backdrop", source.identity())?;
                    if let crate::source::lower::WorthUiBackdropExtentSource::PresentedMosaicRegion {
                        surface,
                        region,
                    } = source.extent()
                    {
                        region_names.insert(region_key(surface, region));
                    }
                }
            }
        }
    }
    Ok(OverlayIdentityTables {
        backdrops: allocate(backdrop_names, "backdrop", crate::UiBackdropIdentity::new)?,
        portals: allocate(portal_names, "portal", crate::UiPortalDeclarationId::new)?,
        surfaces: allocate(
            surface_names,
            "surface",
            crate::UiSemanticSurfaceDeclarationIdentity::new,
        )?,
        regions: allocate(
            region_names,
            "mosaic region",
            crate::UiMosaicRegionDeclarationIdentity::new,
        )?,
    })
}

fn collect_roles(
    modules: &BTreeMap<WorthUiSourceModuleId, Vec<WorthUiFileAuthoredLoweredDeclaration>>,
) -> BTreeMap<String, crate::UiAppearanceRoleDeclaration> {
    modules
        .values()
        .flat_map(|declarations| declarations.iter())
        .filter_map(|declaration| match declaration {
            WorthUiFileAuthoredLoweredDeclaration::Artifact(
                WorthUiArtifactInputNode::AppearanceRole(role),
            ) => Some((role.role().role().as_str().to_owned(), role.role().clone())),
            _ => None,
        })
        .collect()
}

fn resolve_backdrop(
    source: &crate::source::lower::WorthUiBackdropSource,
    tables: &OverlayIdentityTables,
    roles: &BTreeMap<String, crate::UiAppearanceRoleDeclaration>,
) -> Result<crate::UiBackdropDeclaration, ResolutionDenial> {
    let identity = lookup(&tables.backdrops, "backdrop", source.identity())?;
    let surface = lookup(&tables.surfaces, "surface", source.surface())?;
    let scope = match source.scope() {
        crate::source::lower::WorthUiBackdropScopeSource::SurfaceSingleton => {
            crate::UiBackdropScope::SurfaceSingleton
        }
        crate::source::lower::WorthUiBackdropScopeSource::PerPortalInstance(portal) => {
            crate::UiBackdropScope::PerPortalInstance(lookup(&tables.portals, "portal", portal)?)
        }
    };
    let extent = match source.extent() {
        crate::source::lower::WorthUiBackdropExtentSource::SurfaceViewport(surface) => {
            crate::UiBackdropExtentBasis::SurfaceViewport(lookup(
                &tables.surfaces,
                "surface",
                surface,
            )?)
        }
        crate::source::lower::WorthUiBackdropExtentSource::PresentedMosaicRegion {
            surface,
            region,
        } => crate::UiBackdropExtentBasis::PresentedMosaicRegion {
            surface: lookup(&tables.surfaces, "surface", surface)?,
            region: lookup(
                &tables.regions,
                "mosaic region",
                &region_key(surface, region),
            )?,
        },
    };
    let presence = match source.presence() {
        crate::source::lower::WorthUiBackdropPresenceSource::Always => {
            crate::UiBackdropPresenceBasis::Always
        }
        crate::source::lower::WorthUiBackdropPresenceSource::WhilePortalPresented(portal) => {
            crate::UiBackdropPresenceBasis::WhilePortalPresented(lookup(
                &tables.portals,
                "portal",
                portal,
            )?)
        }
    };
    let motion = match source.motion() {
        crate::source::lower::WorthUiBackdropMotionSource::None => {
            crate::UiBackdropMotionBasis::None
        }
        crate::source::lower::WorthUiBackdropMotionSource::PortalPresentation(portal) => {
            crate::UiBackdropMotionBasis::PortalPresentation(lookup(
                &tables.portals,
                "portal",
                portal,
            )?)
        }
    };
    let placement = match source.placement() {
        crate::source::lower::WorthUiBackdropPlacementSource::AboveSurfaceContent => {
            crate::UiBackdropPlacement::AboveSurfaceContent
        }
        crate::source::lower::WorthUiBackdropPlacementSource::ImmediatelyBeforePortal(portal) => {
            crate::UiBackdropPlacement::ImmediatelyBeforePortal(lookup(
                &tables.portals,
                "portal",
                portal,
            )?)
        }
        crate::source::lower::WorthUiBackdropPlacementSource::ImmediatelyAfterPortal(portal) => {
            crate::UiBackdropPlacement::ImmediatelyAfterPortal(lookup(
                &tables.portals,
                "portal",
                portal,
            )?)
        }
        crate::source::lower::WorthUiBackdropPlacementSource::ImmediatelyBeforeBackdrop(
            backdrop,
        ) => crate::UiBackdropPlacement::ImmediatelyBeforeBackdrop(lookup(
            &tables.backdrops,
            "backdrop",
            backdrop,
        )?),
        crate::source::lower::WorthUiBackdropPlacementSource::ImmediatelyAfterBackdrop(
            backdrop,
        ) => crate::UiBackdropPlacement::ImmediatelyAfterBackdrop(lookup(
            &tables.backdrops,
            "backdrop",
            backdrop,
        )?),
    };
    let role = roles
        .get(source.role().as_str())
        .ok_or_else(|| ResolutionDenial::MissingRole(source.role().as_str().to_owned()))?;
    crate::UiBackdropDeclaration::admit_with_role_revision(
        identity,
        surface,
        scope,
        extent,
        presence,
        motion,
        placement,
        source.role_revision(),
        role,
    )
    .map_err(ResolutionDenial::Backdrop)
}

fn insert_name(
    names: &mut BTreeSet<String>,
    namespace: &'static str,
    name: &str,
) -> Result<(), ResolutionDenial> {
    if names.insert(name.to_owned()) {
        Ok(())
    } else {
        Err(ResolutionDenial::Duplicate(namespace, name.to_owned()))
    }
}

fn allocate<T>(
    names: BTreeSet<String>,
    namespace: &'static str,
    constructor: impl Fn(u64) -> Option<T>,
) -> Result<BTreeMap<String, T>, ResolutionDenial> {
    names
        .into_iter()
        .enumerate()
        .map(|(index, name)| {
            let value = index
                .checked_add(1)
                .and_then(|value| u64::try_from(value).ok())
                .ok_or(ResolutionDenial::Capacity(namespace))?;
            let identity = constructor(value).ok_or(ResolutionDenial::Capacity(namespace))?;
            Ok((name, identity))
        })
        .collect()
}

fn lookup<T: Copy>(
    table: &BTreeMap<String, T>,
    namespace: &'static str,
    name: &str,
) -> Result<T, ResolutionDenial> {
    table
        .get(name)
        .copied()
        .ok_or_else(|| ResolutionDenial::Missing(namespace, name.to_owned()))
}

fn region_key(surface: &str, region: &str) -> String {
    let mut key = surface.to_owned();
    key.push(char::from(0));
    key.push_str(region);
    key
}

fn first_provenance(
    modules: &BTreeMap<WorthUiSourceModuleId, Vec<WorthUiFileAuthoredLoweredDeclaration>>,
) -> Option<&WorthUiArtifactInputProvenance> {
    modules
        .values()
        .flat_map(|declarations| declarations.iter())
        .find_map(|declaration| match declaration {
            WorthUiFileAuthoredLoweredDeclaration::Artifact(node) => Some(provenance(node)),
            WorthUiFileAuthoredLoweredDeclaration::Backdrop { provenance, .. } => Some(provenance),
        })
}

fn provenance(node: &WorthUiArtifactInputNode) -> &WorthUiArtifactInputProvenance {
    match node {
        WorthUiArtifactInputNode::Import(node) => node.provenance(),
        WorthUiArtifactInputNode::Component(node)
        | WorthUiArtifactInputNode::Surface(node)
        | WorthUiArtifactInputNode::Binding(node)
        | WorthUiArtifactInputNode::QueryScalar(node)
        | WorthUiArtifactInputNode::QueryCollection(node) => node.provenance(),
        WorthUiArtifactInputNode::Token(node) => node.provenance(),
        WorthUiArtifactInputNode::SemanticArtifact(node) => node.provenance(),
        WorthUiArtifactInputNode::AppearanceRole(node) => node.provenance(),
        WorthUiArtifactInputNode::Backdrop(node) => node.provenance(),
    }
}

fn resolver_report(
    denial: ResolutionDenial,
    provenance: Option<&WorthUiArtifactInputProvenance>,
) -> WorthUiDslCompileReport {
    let (code, message) = match denial {
        ResolutionDenial::Duplicate(namespace, name) => (
            WorthUiDslCompileDiagnosticCode::DuplicateAppearanceDeclaration,
            format!("{namespace} identity '{name}' is declared more than once"),
        ),
        ResolutionDenial::Capacity(namespace) => (
            WorthUiDslCompileDiagnosticCode::OverlayCapacityDenied,
            format!("{namespace} identity namespace capacity denied"),
        ),
        ResolutionDenial::Missing(namespace, name) => (
            WorthUiDslCompileDiagnosticCode::MissingOverlayAnchor,
            format!("{namespace} identity '{name}' is not declared"),
        ),
        ResolutionDenial::MissingRole(name) => (
            WorthUiDslCompileDiagnosticCode::MissingAppearanceDeclaration,
            format!("backdrop references missing appearance role '{name}'"),
        ),
        ResolutionDenial::Backdrop(denial) => (
            match denial {
                crate::UiBackdropDeclarationDenial::ForeignSurfaceExtent => {
                    WorthUiDslCompileDiagnosticCode::ForeignOverlaySurface
                }
                crate::UiBackdropDeclarationDenial::PerPortalScopeMismatch
                | crate::UiBackdropDeclarationDenial::ForeignPortalPlacement => {
                    WorthUiDslCompileDiagnosticCode::InvalidBackdropDeclaration
                }
                crate::UiBackdropDeclarationDenial::IncompatibleAppearanceRole => {
                    WorthUiDslCompileDiagnosticCode::WrongAppearanceDeclarationKind
                }
            },
            format!("backdrop declaration admission denied: {denial:?}"),
        ),
    };
    let (module_id, span) = provenance
        .map(diagnostic_location)
        .unwrap_or_else(|| (String::new(), None));
    WorthUiDslCompileReport::new(vec![WorthUiDslCompileDiagnostic::new(
        code,
        WorthUiDslCompileStopClass::SemanticNormalization,
        message,
        (!module_id.is_empty()).then_some(module_id),
        span,
    )])
}

fn diagnostic_location(
    provenance: &WorthUiArtifactInputProvenance,
) -> (String, Option<WorthUiDslSourceSpan>) {
    match provenance {
        WorthUiArtifactInputProvenance::ParsedSourceDeclaration {
            declaration_span, ..
        } => (
            declaration_span.module_id().as_str().to_owned(),
            Some(WorthUiDslSourceSpan::new(
                declaration_span.module_id().as_str(),
                declaration_span.start_byte(),
                declaration_span.end_byte(),
            )),
        ),
        WorthUiArtifactInputProvenance::RustAuthoredDeclaration {
            authored_module_path,
            ..
        } => (authored_module_path.clone(), None),
    }
}
