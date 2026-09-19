use super::index::{compiled_owner_regions, CompiledOwnerIndex, OccurrenceIndex, RegionIndex};
use crate::facade::mounted::{
    UiMountedOccurrenceGeometryDenial as Denial, UiMountedSurfaceGeometryBatch,
};

pub(crate) struct UiMountedOccurrenceGeometryValidationAuthority<'authority> {
    application:
        &'authority crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
    plan: &'authority crate::runtime::WorthUiActiveExecutionPlan,
    graph: crate::graph::UiGraphAuthority<'authority>,
    identity: crate::mounting::UiMountedIdentityView,
    bindings: &'authority crate::runtime::portal::UiPortalOverlayBindingLifecycle,
}

pub(crate) struct UiValidatedMountedSurfaceGeometry {
    batch: UiMountedSurfaceGeometryBatch,
    work: UiMountedGeometryValidationWork,
}

pub(crate) struct UiMountedGeometryValidationWork {
    pub(crate) region_plan_rows_visited: usize,
    pub(crate) occurrence_index_rows: usize,
    pub(crate) occurrence_ancestry_steps: usize,
    pub(crate) region_index_rows: usize,
    pub(crate) region_lookup_steps: usize,
}

impl<'authority> UiMountedOccurrenceGeometryValidationAuthority<'authority> {
    pub(crate) fn active(
        session: &'authority crate::facade::WorthUiActiveApplicationSession,
    ) -> Self {
        Self {
            application: session.application.prepared_authority(),
            plan: session.application.mounted_geometry_plan(),
            graph: session.application.graph(),
            identity: session.mounted.view(),
            bindings: &session.authored_overlay_bindings,
        }
    }

    pub(crate) fn candidate(
        application: &'authority super::super::application_replacement::WorthUiPreparedApplicationActivation,
        successor: &crate::mounting::UiMountedGraphReplacementSuccessor,
        bindings: &'authority crate::runtime::portal::UiPortalOverlayBindingLifecycle,
    ) -> Self {
        Self {
            application: application.candidate_replacement_authority(),
            plan: application.candidate_plan(),
            graph: application.candidate_graph(),
            identity: successor.identity_view(),
            bindings,
        }
    }

    pub(crate) fn validate(
        &self,
        mut batch: UiMountedSurfaceGeometryBatch,
        scroll: Option<&mut crate::runtime::scroll::UiScrollRuntimeState>,
    ) -> Result<UiValidatedMountedSurfaceGeometry, Denial> {
        if batch.basis().generation() != self.application.generation_identity() {
            return Err(Denial::StaleLayoutBasis);
        }
        let declaration = self.surface_declaration(batch.surface());
        let mut owners = CompiledOwnerIndex::new();
        let mut work = UiMountedGeometryValidationWork {
            region_plan_rows_visited: 0,
            occurrence_index_rows: batch.occurrences().len(),
            occurrence_ancestry_steps: 0,
            region_index_rows: batch.regions().len(),
            region_lookup_steps: 0,
        };
        let region_kinds =
            self.validate_declared_regions(&batch, declaration, &mut owners, &mut work)?;
        let occurrences = batch
            .occurrences()
            .iter()
            .copied()
            .map(|row| (row.instance(), row))
            .collect::<OccurrenceIndex>();
        let regions = batch
            .regions()
            .iter()
            .map(|row| (row.owner(), row.declaration()))
            .collect::<RegionIndex>();
        if let Some(contract) = self
            .application
            .capabilities()
            .mosaic_regions()
            .seam_paint()
            .cloned()
        {
            self.validate_complete_seam_geometry(
                declaration,
                &occurrences,
                &regions,
                &mut owners,
                &mut work,
            )?;
            batch = batch.with_mosaic_seam_paint(contract, region_kinds);
        }
        let mosaic = self.resolve_mosaic_clips(
            &batch,
            &occurrences,
            &regions,
            declaration,
            &mut owners,
            &mut work,
        )?;
        batch = batch.with_mosaic_clips(mosaic);
        let scroll = self.resolve_scroll_clips(
            &batch,
            &occurrences,
            &regions,
            declaration,
            scroll,
            &mut work,
        );
        Ok(UiValidatedMountedSurfaceGeometry {
            batch: batch.with_scroll_clips(scroll),
            work,
        })
    }

    fn surface_declaration(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity> {
        self.bindings
            .bound_owners()
            .find_map(|(declaration, runtime, _)| (runtime == surface).then_some(declaration))
    }

    fn validate_declared_regions(
        &self,
        batch: &UiMountedSurfaceGeometryBatch,
        declaration: Option<worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity>,
        owners: &mut CompiledOwnerIndex,
        work: &mut UiMountedGeometryValidationWork,
    ) -> Result<
        std::collections::BTreeMap<
            (
                worth_ui_host_contract::UiMountedInstanceIdentity,
                worth_ui_dsl::UiMosaicRegionDeclarationIdentity,
            ),
            crate::capability::MosaicRegionKindId,
        >,
        Denial,
    > {
        let mut kinds = std::collections::BTreeMap::new();
        for region in batch.regions() {
            let declaration = declaration.ok_or(Denial::ForeignRegionDeclaration)?;
            let basis = self
                .basis(region.owner())
                .ok_or(Denial::UnknownMountedInstance)?;
            if basis.semantic_surface_identity() != batch.surface() {
                return Err(Denial::ForeignSurface);
            }
            let compiled = self.owner_regions(
                owners,
                declaration,
                basis.graph_node_identity(),
                &mut work.region_plan_rows_visited,
            );
            work.region_lookup_steps += 1;
            if !compiled.contains(region.executed_region(), region.declaration()) {
                return Err(Denial::UnknownExecutedRegion);
            }
            let kind = compiled
                .region_kind(region.executed_region(), region.declaration())
                .and_then(|kind| crate::capability::MosaicRegionKindId::new(kind).ok())
                .ok_or(Denial::UnknownExecutedRegion)?;
            kinds.insert((region.owner(), region.declaration()), kind);
        }
        Ok(kinds)
    }

    pub(super) fn owner_regions<'index>(
        &self,
        owners: &'index mut CompiledOwnerIndex,
        surface: worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity,
        owner: crate::graph::UiGraphNodeIdentity,
        rows: &mut usize,
    ) -> &'index super::index::CompiledOwnerRegions {
        compiled_owner_regions(
            self.plan,
            self.graph,
            self.application.authored_overlay_material(),
            owners,
            surface,
            owner,
            rows,
        )
    }

    pub(super) fn basis(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<&crate::mounting::UiMountedIdentityBasis> {
        self.identity
            .mounted_instances()
            .iter()
            .find(|candidate| candidate.identity() == instance)
            .map(|candidate| candidate.basis())
    }

    pub(super) fn region_declaration_for_plan_index(
        &self,
        surface: worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity,
        plan_index: u32,
    ) -> Option<worth_ui_dsl::UiMosaicRegionDeclarationIdentity> {
        use crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning as Meaning;
        let meaning = self.plan.mounted_projection_ordinary_meaning(plan_index)?;
        let Meaning::Layout(layout) = meaning.as_ref() else {
            return None;
        };
        self.application
            .authored_overlay_material()
            .overlay_declaration_bindings()
            .region_on_surface(surface, layout.region_descriptor()?.id().as_str())
    }

    pub(super) fn install_scroll_ownership(
        &self,
        scroll: &mut crate::runtime::scroll::UiScrollRuntimeState,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        basis: &crate::mounting::UiMountedIdentityBasis,
    ) {
        scroll.resolve_and_install_ownership(
            instance,
            crate::runtime::scroll::UiScrollOwnerIncarnation::from_mount_incarnation(
                basis.mount_incarnation(),
            ),
            self.graph,
            crate::mounting::UiMountedPlanProjectionSource::Executed(self.plan),
            basis.graph_node_identity(),
            basis.semantic_surface_identity(),
            basis.repeated_instance_basis().identity_digest(),
        );
    }
}

impl UiValidatedMountedSurfaceGeometry {
    pub(crate) fn into_parts(
        self,
    ) -> (
        UiMountedSurfaceGeometryBatch,
        UiMountedGeometryValidationWork,
    ) {
        (self.batch, self.work)
    }
}
