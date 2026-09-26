impl super::UiMountedNodeLoweringContext<'_, '_> {
    pub(super) fn lower(
        &self,
        instance: &super::super::super::UiMountedInstanceIdentityView,
    ) -> Result<super::UiMountedProjectionNodeDraft, super::UiMountedProjectionDenial> {
        let graph_node = self
            .graph
            .lookup()
            .graph_node(instance.graph_node_identity())
            .ok_or(super::UiMountedProjectionDenial::UnknownGraphNode)?
            .value();
        let provenance = graph_node.authored_provenance_digest();
        let plan_index = self
            .plan
            .plan_index(provenance)
            .map_err(|_| super::UiMountedProjectionDenial::ForeignPlan)?;
        let (appearance_allocation, grid_correction) = self
            .occurrence_geometry
            .projection_on_grid(instance)
            .map_err(super::UiMountedProjectionDenial::OccurrenceGeometry)?
            .ok_or(super::UiMountedProjectionDenial::OccurrenceGeometry(
                super::super::super::UiMountedOccurrenceGeometryDenial::MissingOccurrenceGeometry,
            ))?;
        let surface_paint_posture = self.occurrence_geometry.surface_paint_posture(instance);
        let allocation = super::lower_allocation(
            self.allocation_source
                .projection(instance.graph_node_identity()),
        )?;
        let predecessor = self
            .predecessor
            .and_then(|semantic| semantic.node(instance.identity()))
            .and_then(|node| node.semantic_text.as_ref());
        let (semantic_input, text_source_lookups) = self
            .semantic_content
            .text_for_lowering(instance.graph_node_identity(), predecessor.is_some());
        let semantic_text_formatting = super::super::semantic_text::lower_semantic_text_formatting(
            self.plan,
            self.theme_values,
            instance.graph_node_identity(),
            plan_index,
            semantic_input,
            predecessor,
            self.theme_value_changed(instance.graph_node_identity()),
        )?;
        if semantic_text_formatting.is_none() && (semantic_input.is_some() || predecessor.is_some())
        {
            return Err(
                super::UiMountedProjectionDenial::MissingSemanticTextFormatting {
                    graph_node: instance.graph_node_identity(),
                    plan_index_available: plan_index.is_some(),
                    predecessor_available: predecessor.is_some(),
                    semantic_input_available: semantic_input.is_some(),
                    theme_value_changed: self.theme_value_changed(instance.graph_node_identity()),
                },
            );
        }
        let mut semantic_text = super::super::semantic_text::lower_semantic_text_seed(
            semantic_input,
            predecessor,
            semantic_text_formatting,
        )?;
        if !self.mechanics_predecessor_available {
            if let Some(seed) = semantic_text.as_mut() {
                seed.require_complete_mechanics();
            }
        }
        let hit_test = super::super::hit_test::lower_hit_test_seed(self.plan, plan_index)?;
        let focus_support = plan_index
            .and_then(|index| self.plan.ordinary_meaning(index))
            .and_then(|meaning| match meaning.as_ref() {
                crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning::Component(
                    component,
                ) => Some(component.focus_support()),
                _ => None,
            })
            .unwrap_or_else(crate::capability::ComponentFocusSupport::not_focusable);
        let focus_scope = if focus_support != crate::capability::ComponentFocusSupport::NotFocusable
        {
            super::super::focus_scope::resolve(
                self.graph,
                self.plan,
                instance.graph_node_identity(),
            )?
        } else {
            None
        };
        let focus_container_owner =
            if focus_support == crate::capability::ComponentFocusSupport::Focusable {
                super::super::focus_scope::container_owner(
                    self.graph,
                    self.plan,
                    instance.graph_node_identity(),
                )?
            } else {
                None
            };
        let (component_id, portal_child_owner, surface_paint_order, surface_geometry, portal_surface_appearance) = plan_index
            .and_then(|index| self.plan.ordinary_meaning(index))
            .and_then(|meaning| match meaning.as_ref() {
                crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning::Component(
                    component,
                ) => Some((component.descriptor().id().clone(), component.portal_child_owner().cloned(), component.surface_paint_order(), component.descriptor().surface_geometry().clone(), component.descriptor().portal_surface_appearance())),
                _ => None,
            })
            .map_or((None, None, None, Default::default(), true), |(component, owner, order, geometry, portal_paint)| (Some(component), owner, order, geometry, portal_paint));
        let participation = super::lower_participation(
            graph_node.participation_posture(),
            semantic_text.is_some() || graph_node.has_appearance_attachment(),
            hit_test.is_some(),
        );
        let (appearance_clip, clip_ancestry_entries) =
            super::super::appearance::derive_unbound_ancestry(
                self.graph,
                self.plan,
                instance.graph_node_identity(),
                portal_child_owner.is_some(),
                self.occurrence_geometry.mosaic_clips(instance),
                self.occurrence_geometry.scroll_clips(instance),
            )?;
        Ok(super::UiMountedProjectionNodeDraft {
            mounted_instance: instance.identity(),
            graph_node: instance.graph_node_identity(),
            semantic_surface: instance.basis().semantic_surface_identity(),
            incarnation: instance.mount_incarnation(),
            plan_digest: self.plan_digest,
            role: super::mechanical_role(graph_node.operator_kind()),
            participation,
            allocation,
            appearance_allocation,
            grid_correction,
            appearance_clip,
            surface_paint_posture,
            surface_paint_order,
            surface_geometry,
            portal_surface_appearance,
            has_appearance_attachment: graph_node.has_appearance_attachment(),
            clip_ancestry_entries,
            text_source_lookups,
            plan_index,
            semantic_text,
            hit_test,
            focus_support,
            focus_scope,
            focus_container_owner,
            component_id,
            portal_child_owner,
        })
    }
}
