mod declaration_basis;
mod fingerprint;

use crate::source::{
    WorthUiAuthoredMount, WorthUiAuthoredRegion, WorthUiAuthoredStructuralBody,
    WorthUiSemanticModule, WorthUiSourceModuleId,
};
use declaration_basis::WorthUiSemanticDeclarationExactBasis;
use fingerprint::Fingerprint;

#[derive(Debug, Eq, PartialEq)]
pub(super) struct WorthUiSemanticPackageExactBasis {
    modules: Box<[WorthUiSemanticModuleExactBasis]>,
}

#[derive(Debug, Eq, PartialEq)]
struct WorthUiSemanticModuleExactBasis {
    module_id: String,
    declarations: Box<[WorthUiSemanticDeclarationExactBasis]>,
}

#[derive(Debug, Eq, PartialEq)]
struct WorthUiSemanticBlockExactBasis {
    name: String,
    authored_identity: Option<String>,
    appearance_role_attachment: Option<(String, u64)>,
    structure: WorthUiStructuralBodyExactBasis,
}

#[derive(Debug, Eq, PartialEq)]
struct WorthUiStructuralBodyExactBasis {
    root_regions: Box<[WorthUiRegionExactBasis]>,
    projection_contents: Box<[String]>,
    interaction_routes: Box<[WorthUiInteractionRouteExactBasis]>,
}

#[derive(Debug, Eq, PartialEq)]
struct WorthUiInteractionRouteExactBasis {
    family: String,
    declaration: String,
    kind: String,
}

#[derive(Debug, Eq, PartialEq)]
struct WorthUiRegionExactBasis {
    id: String,
    sizing_contract: Option<String>,
    state_slot: Option<String>,
    child_regions: Box<[WorthUiRegionExactBasis]>,
    mounts: Box<[WorthUiMountExactBasis]>,
}

#[derive(Debug, Eq, PartialEq)]
struct WorthUiMountExactBasis {
    surface: String,
    placement_policy: Option<String>,
    state_slot: Option<String>,
}

impl WorthUiSemanticPackageExactBasis {
    pub(super) fn from_modules<'module>(
        modules: impl IntoIterator<
            Item = (
                &'module WorthUiSourceModuleId,
                &'module WorthUiSemanticModule,
            ),
        >,
    ) -> Self {
        Self {
            modules: modules
                .into_iter()
                .map(|(module_id, module)| WorthUiSemanticModuleExactBasis {
                    module_id: module_id.as_str().to_owned(),
                    declarations: module
                        .declarations()
                        .iter()
                        .map(WorthUiSemanticDeclarationExactBasis::from_declaration)
                        .collect(),
                })
                .collect(),
        }
    }

    pub(super) fn narrowing_fingerprint(&self) -> u64 {
        let mut fingerprint = Fingerprint::new("worth-ui:semantic-package:v1");
        fingerprint.fold_usize(self.modules.len());
        for module in &self.modules {
            module.fold_into(&mut fingerprint);
        }
        fingerprint.finish()
    }
}

impl WorthUiSemanticModuleExactBasis {
    fn fold_into(&self, fingerprint: &mut Fingerprint) {
        fingerprint.fold_text("module");
        fingerprint.fold_text(&self.module_id);
        fingerprint.fold_usize(self.declarations.len());
        for declaration in &self.declarations {
            declaration.fold_into(fingerprint);
        }
    }
}

impl WorthUiSemanticBlockExactBasis {
    fn from_block(block: &crate::source::WorthUiSemanticBlock) -> Self {
        Self {
            name: block.name_text().to_owned(),
            authored_identity: block.authored_identity().map(str::to_owned),
            appearance_role_attachment: block.appearance_role_attachment().map(|attachment| {
                (
                    attachment.role().as_str().to_owned(),
                    attachment.revision().value(),
                )
            }),
            structure: WorthUiStructuralBodyExactBasis::from_structure(block.structure()),
        }
    }

    fn fold_into(&self, fingerprint: &mut Fingerprint) {
        fingerprint.fold_text(&self.name);
        fingerprint.fold_optional_text(self.authored_identity.as_deref());
        match &self.appearance_role_attachment {
            Some((role, revision)) => {
                fingerprint.fold_bool(true);
                fingerprint.fold_text(role);
                fingerprint.fold_u64(*revision);
            }
            None => fingerprint.fold_bool(false),
        }
        self.structure.fold_into(fingerprint);
    }
}

impl WorthUiStructuralBodyExactBasis {
    fn from_structure(structure: &WorthUiAuthoredStructuralBody) -> Self {
        Self {
            root_regions: structure
                .root_regions()
                .iter()
                .map(WorthUiRegionExactBasis::from_region)
                .collect(),
            projection_contents: structure
                .projection_contents()
                .iter()
                .map(|content| content.projection_identity_text().to_owned())
                .collect(),
            interaction_routes: structure
                .interaction_routes()
                .iter()
                .map(|route| WorthUiInteractionRouteExactBasis {
                    family: route.family().as_str().to_owned(),
                    declaration: route.declaration_identity().to_owned(),
                    kind: match route.kind() {
                        crate::WorthUiIntentInteractionRouteKind::Product => "product",
                        crate::WorthUiIntentInteractionRouteKind::Confirmation => "confirmation",
                    }
                    .to_owned(),
                })
                .collect(),
        }
    }

    fn fold_into(&self, fingerprint: &mut Fingerprint) {
        fingerprint.fold_usize(self.root_regions.len());
        for region in &self.root_regions {
            region.fold_into(fingerprint);
        }
        fingerprint.fold_usize(self.projection_contents.len());
        for projection in &self.projection_contents {
            fingerprint.fold_text(projection);
        }
        fingerprint.fold_usize(self.interaction_routes.len());
        for route in &self.interaction_routes {
            fingerprint.fold_text(&route.family);
            fingerprint.fold_text(&route.declaration);
            fingerprint.fold_text(&route.kind);
        }
    }
}

impl WorthUiRegionExactBasis {
    fn from_region(region: &WorthUiAuthoredRegion) -> Self {
        Self {
            id: region.region_id_text().to_owned(),
            sizing_contract: region.sizing_contract_id_text().map(str::to_owned),
            state_slot: region.state_slot_id_text().map(str::to_owned),
            child_regions: region
                .child_regions()
                .iter()
                .map(Self::from_region)
                .collect(),
            mounts: region
                .mounts()
                .iter()
                .map(WorthUiMountExactBasis::from_mount)
                .collect(),
        }
    }

    fn fold_into(&self, fingerprint: &mut Fingerprint) {
        fingerprint.fold_text(&self.id);
        fingerprint.fold_optional_text(self.sizing_contract.as_deref());
        fingerprint.fold_optional_text(self.state_slot.as_deref());
        fingerprint.fold_usize(self.child_regions.len());
        for child in &self.child_regions {
            child.fold_into(fingerprint);
        }
        fingerprint.fold_usize(self.mounts.len());
        for mount in &self.mounts {
            mount.fold_into(fingerprint);
        }
    }
}

impl WorthUiMountExactBasis {
    fn from_mount(mount: &WorthUiAuthoredMount) -> Self {
        Self {
            surface: mount.surface_id_text().to_owned(),
            placement_policy: mount.placement_policy_id_text().map(str::to_owned),
            state_slot: mount.state_slot_id_text().map(str::to_owned),
        }
    }

    fn fold_into(&self, fingerprint: &mut Fingerprint) {
        fingerprint.fold_text(&self.surface);
        fingerprint.fold_optional_text(self.placement_policy.as_deref());
        fingerprint.fold_optional_text(self.state_slot.as_deref());
    }
}
