use std::rc::Rc;

/// An attempt's context, shared: a retained entry and each inspection record
/// hold it without copying its role, state and aspects.
#[derive(Clone)]
pub(crate) struct UiAppearanceAttemptContext(Rc<UiAppearanceAttemptFields>);

#[derive(Clone)]
struct UiAppearanceAttemptFields {
    target: super::super::state::UiAppearanceTarget,
    mounted: crate::mounting::UiMountedAppearanceNodeInputContext,
    generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    role: Option<worth_ui_dsl::UiAppearanceRoleDeclaration>,
    theme_identity: Option<Box<str>>,
    theme_revision: u64,
    catalog_revision: u64,
    state: Option<super::super::state::UiAppearanceStateVector>,
    aspects: Box<[super::UiResolvedAppearanceAspect]>,
    aspect_hints: Box<[worth_ui_dsl::UiAppearanceAspect]>,
    theme_slots_compared: u32,
    consumers_selected: u32,
    owner_evidence: u64,
    invalidation_batch: Option<super::super::invalidation::UiAppearanceInvalidationBatch>,
}

impl UiAppearanceAttemptContext {
    #[cfg(test)]
    pub(crate) fn with_clip_for_test(
        mut self,
        clip: crate::mounting::UiMountedAppearanceClip,
    ) -> Self {
        let fields = Rc::make_mut(&mut self.0);
        fields.mounted = fields.mounted.clone().with_clip_for_test(clip);
        self
    }

    pub(crate) fn new(
        target: super::super::state::UiAppearanceTarget,
        mounted: crate::mounting::UiMountedAppearanceNodeInputContext,
        generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        consumers_selected: u32,
        owner_evidence: u64,
    ) -> Self {
        Self(Rc::new(UiAppearanceAttemptFields {
            target,
            mounted,
            generation,
            role: None,
            theme_identity: None,
            theme_revision: 0,
            catalog_revision: 0,
            state: None,
            aspects: Box::new([]),
            aspect_hints: Box::new([]),
            theme_slots_compared: 0,
            consumers_selected,
            owner_evidence,
            invalidation_batch: None,
        }))
    }

    pub(crate) fn set_role(&mut self, role: &worth_ui_dsl::UiAppearanceRoleDeclaration) {
        let fields = Rc::make_mut(&mut self.0);
        fields.aspect_hints = role
            .partitions()
            .iter()
            .map(|(aspect, _)| *aspect)
            .collect();
        fields.role = Some(role.clone());
    }

    pub(crate) fn set_theme(&mut self, theme: &super::super::theme::UiThemeResolutionView) {
        let fields = Rc::make_mut(&mut self.0);
        fields.theme_identity = Some(theme.definition_identity().into());
        fields.theme_revision = theme.definition_revision();
        fields.catalog_revision = theme.catalog_revision();
    }

    pub(crate) fn set_state(&mut self, state: &super::super::state::UiAppearanceStateVector) {
        let fields = Rc::make_mut(&mut self.0);
        fields.owner_evidence = state.evidence_digest();
        fields.generation = state.basis().generation().clone();
        fields.state = Some(state.clone());
    }

    pub(crate) fn set_invalidation_batch(
        &mut self,
        batch: &super::super::invalidation::UiAppearanceInvalidationBatch,
    ) {
        let fields = Rc::make_mut(&mut self.0);
        fields.invalidation_batch = Some(batch.clone());
    }

    pub(crate) fn set_projection(&mut self, projection: &super::UiAppearanceProjection) {
        let fields = Rc::make_mut(&mut self.0);
        fields.aspects = projection.aspects().to_vec().into_boxed_slice();
        fields.theme_slots_compared = projection
            .aspects()
            .iter()
            .map(|aspect| aspect.theme_slots_compared())
            .try_fold(0_u32, |total, compared| total.checked_add(compared))
            .expect("appearance theme traversal count fits its bounded catalog");
    }

    pub(crate) fn set_theme_slots_compared(&mut self, compared: u32) {
        let fields = Rc::make_mut(&mut self.0);
        fields.theme_slots_compared = compared;
    }

    pub(crate) fn target(&self) -> &super::super::state::UiAppearanceTarget {
        &self.0.target
    }

    pub(crate) fn frame(&self) -> worth_ui_host_contract::UiMountedFrameIdentity {
        self.0.mounted.frame
    }

    pub(crate) fn semantic_surface(&self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.0.target.surface()
    }

    pub(crate) fn mounted_instance(&self) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.0.target.mounted_instance()
    }

    pub(crate) fn graph_node(&self) -> crate::graph::UiGraphNodeIdentity {
        self.0.target.graph_node()
    }

    pub(crate) fn incarnation(&self) -> worth_ui_host_contract::UiMountIncarnation {
        self.0.target.incarnation()
    }

    pub(crate) fn node_receipt(&self) -> worth_ui_host_contract::UiMountedNodeReceiptIdentity {
        self.0.target.node_receipt()
    }

    pub(crate) fn issuer(&self) -> worth_ui_host_contract::UiMountedNodeReceiptIssuer {
        self.0.mounted.issuer()
    }

    pub(crate) fn lower_resolved(
        &self,
        projection: &super::UiAppearanceProjection,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        outline_fringe: Result<
            worth_ui_host_contract::UiAppearanceLogicalLength,
            crate::mounting::UiMountedAppearanceLoweringDenial,
        >,
    ) -> Result<
        crate::mounting::UiMountedAppearanceLoweringInput,
        crate::mounting::UiMountedAppearanceLoweringDenial,
    > {
        self.0
            .mounted
            .lower_resolved_projection(projection, presentation, outline_fringe)
    }

    pub(crate) fn generation(&self) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.0.generation
    }

    pub(crate) fn role(&self) -> Option<&worth_ui_dsl::UiAppearanceRoleDeclaration> {
        self.0.role.as_ref()
    }

    pub(crate) fn theme_identity(&self) -> Option<&str> {
        self.0.theme_identity.as_deref()
    }

    pub(crate) fn theme_revision(&self) -> u64 {
        self.0.theme_revision
    }

    pub(crate) fn state(&self) -> Option<&super::super::state::UiAppearanceStateVector> {
        self.0.state.as_ref()
    }

    pub(crate) fn aspects(&self) -> &[super::UiResolvedAppearanceAspect] {
        &self.0.aspects
    }

    pub(crate) fn aspect_hints(&self) -> &[worth_ui_dsl::UiAppearanceAspect] {
        &self.0.aspect_hints
    }

    pub(crate) fn consumers_selected(&self) -> u32 {
        self.0.consumers_selected
    }

    pub(crate) fn theme_slots_compared(&self) -> u32 {
        self.0.theme_slots_compared
    }

    pub(crate) fn owner_evidence(&self) -> u64 {
        self.0.owner_evidence
    }
}

#[derive(Clone)]
pub(crate) struct UiAppearanceProjectionAttempt {
    context: UiAppearanceAttemptContext,
    projection: Option<super::UiAppearanceProjection>,
    denial: Option<super::super::inspection::UiAppearanceInspectionDenial>,
}

impl UiAppearanceProjectionAttempt {
    pub(crate) fn resolved(
        mut context: UiAppearanceAttemptContext,
        projection: super::UiAppearanceProjection,
    ) -> Self {
        context.set_projection(&projection);
        Self {
            context,
            projection: Some(projection),
            denial: None,
        }
    }

    pub(crate) fn denied(
        context: UiAppearanceAttemptContext,
        denial: super::super::inspection::UiAppearanceInspectionDenial,
    ) -> Self {
        Self {
            context,
            projection: None,
            denial: Some(denial),
        }
    }

    pub(crate) const fn context(&self) -> &UiAppearanceAttemptContext {
        &self.context
    }

    pub(crate) const fn projection(&self) -> Option<&super::UiAppearanceProjection> {
        self.projection.as_ref()
    }

    pub(crate) const fn denial(
        &self,
    ) -> Option<super::super::inspection::UiAppearanceInspectionDenial> {
        self.denial
    }
}
