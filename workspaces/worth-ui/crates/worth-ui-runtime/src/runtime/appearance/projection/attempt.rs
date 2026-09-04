#[derive(Clone)]
pub(crate) struct UiAppearanceAttemptContext {
    target: super::super::state::UiAppearanceTarget,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    issuer: worth_ui_host_contract::UiMountedNodeReceiptIssuer,
    plan_digest: u64,
    allocation: worth_ui_host_contract::UiMountedAllocationProjection,
    generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    role: Option<worth_ui_dsl::UiAppearanceRoleDeclaration>,
    theme_identity: Option<Box<str>>,
    theme_revision: u64,
    catalog_revision: u64,
    state: Option<super::super::state::UiAppearanceStateVector>,
    aspects: Box<[super::UiResolvedAppearanceAspect]>,
    aspect_hints: Box<[worth_ui_dsl::UiAppearanceAspect]>,
    consumers_selected: u32,
    owner_evidence: u64,
    invalidation_batch: Option<super::super::invalidation::UiAppearanceInvalidationBatch>,
}

impl UiAppearanceAttemptContext {
    pub(crate) fn new(
        target: super::super::state::UiAppearanceTarget,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        issuer: worth_ui_host_contract::UiMountedNodeReceiptIssuer,
        plan_digest: u64,
        allocation: worth_ui_host_contract::UiMountedAllocationProjection,
        generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        consumers_selected: u32,
        owner_evidence: u64,
    ) -> Self {
        Self {
            target,
            frame,
            issuer,
            plan_digest,
            allocation,
            generation,
            role: None,
            theme_identity: None,
            theme_revision: 0,
            catalog_revision: 0,
            state: None,
            aspects: Box::new([]),
            aspect_hints: Box::new([]),
            consumers_selected,
            owner_evidence,
            invalidation_batch: None,
        }
    }

    pub(crate) fn set_role(&mut self, role: &worth_ui_dsl::UiAppearanceRoleDeclaration) {
        self.aspect_hints = role
            .partitions()
            .iter()
            .map(|(aspect, _)| *aspect)
            .collect();
        self.role = Some(role.clone());
    }

    pub(crate) fn set_theme(&mut self, theme: &super::super::theme::UiThemeResolutionView) {
        self.theme_identity = Some(theme.definition_identity().into());
        self.theme_revision = theme.definition_revision();
        self.catalog_revision = theme.catalog_revision();
    }

    pub(crate) fn set_state(&mut self, state: &super::super::state::UiAppearanceStateVector) {
        self.owner_evidence = state.evidence_digest();
        self.generation = state.basis().generation().clone();
        self.state = Some(state.clone());
    }

    pub(crate) fn set_invalidation_batch(
        &mut self,
        batch: &super::super::invalidation::UiAppearanceInvalidationBatch,
    ) {
        self.invalidation_batch = Some(batch.clone());
    }

    pub(crate) fn set_projection(&mut self, projection: &super::UiAppearanceProjection) {
        self.aspects = projection.aspects().to_vec().into_boxed_slice();
    }

    pub(crate) const fn target(&self) -> &super::super::state::UiAppearanceTarget {
        &self.target
    }

    pub(crate) const fn frame(&self) -> worth_ui_host_contract::UiMountedFrameIdentity {
        self.frame
    }

    pub(crate) const fn semantic_surface(
        &self,
    ) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.target.surface()
    }

    pub(crate) const fn mounted_instance(
        &self,
    ) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.target.mounted_instance()
    }

    pub(crate) const fn graph_node(&self) -> crate::graph::UiGraphNodeIdentity {
        self.target.graph_node()
    }

    pub(crate) const fn incarnation(&self) -> worth_ui_host_contract::UiMountIncarnation {
        self.target.incarnation()
    }

    pub(crate) const fn node_receipt(
        &self,
    ) -> worth_ui_host_contract::UiMountedNodeReceiptIdentity {
        self.target.node_receipt()
    }

    pub(crate) const fn issuer(&self) -> worth_ui_host_contract::UiMountedNodeReceiptIssuer {
        self.issuer
    }

    pub(crate) fn lower_resolved(
        &self,
        projection: &super::UiAppearanceProjection,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    ) -> Result<
        crate::mounting::UiMountedAppearanceLoweringInput,
        crate::mounting::UiMountedAppearanceLoweringDenial,
    > {
        let input = crate::mounting::UiMountedAppearanceNodeInput::from_resolved_projection(
            self.issuer,
            self.semantic_surface(),
            self.node_receipt(),
            self.graph_node(),
            self.plan_digest,
            self.allocation,
            projection,
        )?;
        Ok(
            crate::mounting::UiMountedAppearanceLoweringInput::for_single_node(
                self.frame,
                self.semantic_surface(),
                presentation,
                input,
            ),
        )
    }

    pub(crate) const fn generation(
        &self,
    ) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.generation
    }

    pub(crate) const fn role(&self) -> Option<&worth_ui_dsl::UiAppearanceRoleDeclaration> {
        self.role.as_ref()
    }

    pub(crate) fn theme_identity(&self) -> Option<&str> {
        self.theme_identity.as_deref()
    }

    pub(crate) const fn theme_revision(&self) -> u64 {
        self.theme_revision
    }

    pub(crate) const fn catalog_revision(&self) -> u64 {
        self.catalog_revision
    }

    pub(crate) const fn state(&self) -> Option<&super::super::state::UiAppearanceStateVector> {
        self.state.as_ref()
    }

    pub(crate) fn aspects(&self) -> &[super::UiResolvedAppearanceAspect] {
        &self.aspects
    }

    pub(crate) fn aspect_hints(&self) -> &[worth_ui_dsl::UiAppearanceAspect] {
        &self.aspect_hints
    }

    pub(crate) const fn consumers_selected(&self) -> u32 {
        self.consumers_selected
    }

    pub(crate) const fn owner_evidence(&self) -> u64 {
        self.owner_evidence
    }

    pub(crate) fn invalidation_batch(
        &self,
    ) -> Option<&super::super::invalidation::UiAppearanceInvalidationBatch> {
        self.invalidation_batch.as_ref()
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
