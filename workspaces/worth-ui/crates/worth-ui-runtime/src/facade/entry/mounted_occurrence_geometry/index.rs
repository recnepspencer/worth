use std::collections::{BTreeMap, BTreeSet};

use crate::facade::mounted::UiMountedOccurrenceGeometry;

pub(super) type OccurrenceIndex =
    BTreeMap<worth_ui_host_contract::UiMountedInstanceIdentity, UiMountedOccurrenceGeometry>;
pub(super) type RegionIndex = BTreeSet<(
    worth_ui_host_contract::UiMountedInstanceIdentity,
    worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
)>;
pub(super) type CompiledOwnerIndex =
    BTreeMap<crate::graph::UiGraphNodeIdentity, CompiledOwnerRegions>;

pub(super) struct CompiledOwnerRegions {
    bindings: Vec<crate::runtime::session::UiMountedRegionDeclarationBinding>,
    executed_by_declaration:
        BTreeMap<worth_ui_dsl::UiMosaicRegionDeclarationIdentity, BTreeSet<Box<str>>>,
}

impl CompiledOwnerRegions {
    pub(super) fn new(
        bindings: Vec<crate::runtime::session::UiMountedRegionDeclarationBinding>,
    ) -> Self {
        let mut executed_by_declaration = BTreeMap::<_, BTreeSet<Box<str>>>::new();
        for binding in &bindings {
            executed_by_declaration
                .entry(binding.declaration())
                .or_default()
                .insert(binding.executed_region().into());
        }
        Self {
            bindings,
            executed_by_declaration,
        }
    }

    pub(super) fn contains(
        &self,
        executed_region: &str,
        declaration: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
    ) -> bool {
        self.executed_by_declaration
            .get(&declaration)
            .is_some_and(|executed| executed.contains(executed_region))
    }

    pub(super) fn region_kind(
        &self,
        executed_region: &str,
        declaration: worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
    ) -> Option<&str> {
        self.bindings
            .iter()
            .find(|binding| {
                binding.executed_region() == executed_region && binding.declaration() == declaration
            })
            .map(crate::runtime::session::UiMountedRegionDeclarationBinding::region_kind)
    }

    pub(super) fn bindings(&self) -> &[crate::runtime::session::UiMountedRegionDeclarationBinding] {
        &self.bindings
    }
}

pub(super) fn compiled_owner_regions<'index>(
    application: &crate::runtime::session::WorthUiApplicationSessionState,
    index: &'index mut CompiledOwnerIndex,
    surface: worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity,
    owner: crate::graph::UiGraphNodeIdentity,
    plan_rows_visited: &mut usize,
) -> &'index CompiledOwnerRegions {
    index.entry(owner).or_insert_with(|| {
        let (bindings, visited) = application.mounted_region_declarations(surface, owner);
        *plan_rows_visited += visited;
        CompiledOwnerRegions::new(bindings)
    })
}
