use crate::capability::{CommandId, ComponentId, ThemeTokenId};

use super::{
    ComponentAccessibilitySupport, ComponentChildPolicy, ComponentExecutionLane,
    ComponentFocusSupport, ComponentPropSchema, ComponentStateOwnership,
};

/// Declarative renderable component capability supplied by an application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentDescriptor {
    id: ComponentId,
    prop_schema: Option<ComponentPropSchema>,
    child_policy: ComponentChildPolicy,
    state_ownership: Option<ComponentStateOwnership>,
    accessibility: ComponentAccessibilitySupport,
    focus: ComponentFocusSupport,
    theme_token_dependencies: Vec<ThemeTokenId>,
    command_binding_slots: Vec<CommandId>,
    execution_lane: ComponentExecutionLane,
    canvas_spatial_contract: Option<super::ComponentCanvasSpatialContract>,
    realtime_overlay_contract: Option<super::ComponentRealtimeOverlayContract>,
    allocation_contracts:
        super::component_allocation_contract_state::ComponentAllocationContractState,
    static_paint_contract: Option<super::ComponentStaticPaintContract>,
    surface_paint_order: Option<u32>,
    semantic_text_contract: Option<super::ComponentSemanticTextContract>,
    hit_test_contract: Option<super::ComponentHitTestContract>,
    portal_child_contract: Option<super::ComponentPortalChildContract>,
    appearance_aspect_contract: Option<worth_ui_dsl::UiAppearanceAspectContract>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "Gate 0 freezes component appearance contract denials"
)]
pub(crate) enum ComponentAppearanceAspectContractDenial {
    BackdropContractOnComponent,
}

impl ComponentDescriptor {
    pub fn new(
        id: ComponentId,
        prop_schema: ComponentPropSchema,
        child_policy: ComponentChildPolicy,
        state_ownership: ComponentStateOwnership,
    ) -> Self {
        Self {
            id,
            prop_schema: Some(prop_schema),
            child_policy,
            state_ownership: Some(state_ownership),
            accessibility: ComponentAccessibilitySupport::semantic(),
            focus: ComponentFocusSupport::not_focusable(),
            theme_token_dependencies: Vec::new(),
            command_binding_slots: Vec::new(),
            execution_lane: ComponentExecutionLane::Passive,
            canvas_spatial_contract: None,
            realtime_overlay_contract: None,
            allocation_contracts:
                super::component_allocation_contract_state::ComponentAllocationContractState::empty(
                ),
            static_paint_contract: None,
            surface_paint_order: None,
            semantic_text_contract: None,
            hit_test_contract: None,
            portal_child_contract: None,
            appearance_aspect_contract: None,
        }
    }

    pub fn without_prop_schema_for_diagnostics(
        id: ComponentId,
        child_policy: ComponentChildPolicy,
        state_ownership: ComponentStateOwnership,
    ) -> Self {
        Self {
            id,
            prop_schema: None,
            child_policy,
            state_ownership: Some(state_ownership),
            accessibility: ComponentAccessibilitySupport::semantic(),
            focus: ComponentFocusSupport::not_focusable(),
            theme_token_dependencies: Vec::new(),
            command_binding_slots: Vec::new(),
            execution_lane: ComponentExecutionLane::Passive,
            canvas_spatial_contract: None,
            realtime_overlay_contract: None,
            allocation_contracts:
                super::component_allocation_contract_state::ComponentAllocationContractState::empty(
                ),
            static_paint_contract: None,
            surface_paint_order: None,
            semantic_text_contract: None,
            hit_test_contract: None,
            portal_child_contract: None,
            appearance_aspect_contract: None,
        }
    }

    pub fn without_state_ownership_for_diagnostics(
        id: ComponentId,
        prop_schema: ComponentPropSchema,
        child_policy: ComponentChildPolicy,
    ) -> Self {
        Self {
            id,
            prop_schema: Some(prop_schema),
            child_policy,
            state_ownership: None,
            accessibility: ComponentAccessibilitySupport::semantic(),
            focus: ComponentFocusSupport::not_focusable(),
            theme_token_dependencies: Vec::new(),
            command_binding_slots: Vec::new(),
            execution_lane: ComponentExecutionLane::Passive,
            canvas_spatial_contract: None,
            realtime_overlay_contract: None,
            allocation_contracts:
                super::component_allocation_contract_state::ComponentAllocationContractState::empty(
                ),
            static_paint_contract: None,
            surface_paint_order: None,
            semantic_text_contract: None,
            hit_test_contract: None,
            portal_child_contract: None,
            appearance_aspect_contract: None,
        }
    }

    pub fn with_accessibility(mut self, accessibility: ComponentAccessibilitySupport) -> Self {
        self.accessibility = accessibility;
        self
    }

    pub fn with_focus(mut self, focus: ComponentFocusSupport) -> Self {
        self.focus = focus;
        self
    }

    pub fn with_theme_token_dependency(mut self, token_id: ThemeTokenId) -> Self {
        self.theme_token_dependencies.push(token_id);
        self
    }

    pub fn with_command_binding_slot(mut self, command_id: CommandId) -> Self {
        self.command_binding_slots.push(command_id);
        self
    }

    pub fn with_execution_lane(mut self, execution_lane: ComponentExecutionLane) -> Self {
        self.execution_lane = execution_lane;
        if execution_lane != ComponentExecutionLane::CanvasSpatial {
            self.canvas_spatial_contract = None;
        }
        if execution_lane != ComponentExecutionLane::RealtimeOverlay {
            self.realtime_overlay_contract = None;
        }
        self
    }

    pub fn with_canvas_spatial_contract(
        mut self,
        contract: super::ComponentCanvasSpatialContract,
    ) -> Self {
        self.execution_lane = ComponentExecutionLane::CanvasSpatial;
        self.canvas_spatial_contract = Some(contract);
        self.realtime_overlay_contract = None;
        self
    }

    pub fn with_realtime_overlay_contract(
        mut self,
        contract: super::ComponentRealtimeOverlayContract,
    ) -> Self {
        self.execution_lane = ComponentExecutionLane::RealtimeOverlay;
        self.canvas_spatial_contract = None;
        self.realtime_overlay_contract = Some(contract);
        self
    }

    pub fn with_allocation_measurement_contract(
        mut self,
        contract: super::ComponentAllocationMeasurementContract,
    ) -> Self {
        self.allocation_contracts = self.allocation_contracts.record(contract);
        self
    }

    pub fn with_static_paint(
        mut self,
        contract: super::ComponentStaticPaintContract,
        allocation: super::ComponentAllocationMeasurementContract,
    ) -> Self {
        if !self
            .theme_token_dependencies
            .contains(contract.theme_token())
        {
            self.theme_token_dependencies
                .push(contract.theme_token().clone());
        }
        self.allocation_contracts = self.allocation_contracts.record(allocation);
        self.static_paint_contract = Some(contract);
        self
    }

    /// Declares back-to-front order for this component's appearance surface and outline.
    /// This is independent of bootstrap static paint and does not order interaction.
    pub fn with_surface_paint_order(mut self, rank: u32) -> Self {
        self.surface_paint_order = Some(rank);
        self
    }

    pub const fn surface_paint_order(&self) -> Option<u32> {
        self.surface_paint_order
    }

    pub fn with_semantic_text(mut self, contract: super::ComponentSemanticTextContract) -> Self {
        for token in contract.foreground_tokens() {
            if !self.theme_token_dependencies.contains(token) {
                self.theme_token_dependencies.push(token.clone());
            }
        }
        self.semantic_text_contract = Some(contract);
        self
    }

    pub fn with_hit_test(mut self, contract: super::ComponentHitTestContract) -> Self {
        self.allocation_contracts = self.allocation_contracts.record(contract.allocation());
        self.hit_test_contract = Some(contract);
        self
    }

    pub fn with_portal_child(mut self, contract: super::ComponentPortalChildContract) -> Self {
        self.portal_child_contract = Some(contract);
        self
    }

    pub fn id(&self) -> &ComponentId {
        &self.id
    }

    #[allow(
        dead_code,
        reason = "Gate 0 stages component aspect support without live appearance"
    )]
    pub(crate) fn with_appearance_aspect_contract(
        mut self,
        contract: worth_ui_dsl::UiAppearanceAspectContract,
    ) -> Result<Self, ComponentAppearanceAspectContractDenial> {
        if contract.applicability() != worth_ui_dsl::UiAppearanceAspectApplicability::Component {
            return Err(ComponentAppearanceAspectContractDenial::BackdropContractOnComponent);
        }
        self.appearance_aspect_contract = Some(contract);
        Ok(self)
    }

    pub(crate) const fn appearance_aspect_contract(
        &self,
    ) -> Option<&worth_ui_dsl::UiAppearanceAspectContract> {
        self.appearance_aspect_contract.as_ref()
    }

    pub fn prop_schema(&self) -> Option<&ComponentPropSchema> {
        self.prop_schema.as_ref()
    }

    pub fn child_policy(&self) -> ComponentChildPolicy {
        self.child_policy
    }

    pub fn state_ownership(&self) -> Option<ComponentStateOwnership> {
        self.state_ownership
    }

    pub fn accessibility(&self) -> ComponentAccessibilitySupport {
        self.accessibility
    }

    pub fn focus(&self) -> ComponentFocusSupport {
        self.focus
    }

    pub fn theme_token_dependencies(&self) -> &[ThemeTokenId] {
        &self.theme_token_dependencies
    }

    pub fn command_binding_slots(&self) -> &[CommandId] {
        &self.command_binding_slots
    }

    pub fn execution_lane(&self) -> ComponentExecutionLane {
        self.execution_lane
    }

    pub fn canvas_spatial_contract(&self) -> Option<super::ComponentCanvasSpatialContract> {
        self.canvas_spatial_contract
    }

    pub fn realtime_overlay_contract(&self) -> Option<super::ComponentRealtimeOverlayContract> {
        self.realtime_overlay_contract
    }

    pub fn allocation_measurement_contract(
        &self,
    ) -> Option<super::ComponentAllocationMeasurementContract> {
        self.allocation_contracts.resolved()
    }

    pub fn static_paint_contract(&self) -> Option<&super::ComponentStaticPaintContract> {
        self.static_paint_contract.as_ref()
    }

    pub fn semantic_text_contract(&self) -> Option<&super::ComponentSemanticTextContract> {
        self.semantic_text_contract.as_ref()
    }

    pub fn hit_test_contract(&self) -> Option<super::ComponentHitTestContract> {
        self.hit_test_contract
    }

    pub fn portal_child_contract(&self) -> Option<&super::ComponentPortalChildContract> {
        self.portal_child_contract.as_ref()
    }

    pub(crate) fn has_conflicting_allocation_contracts(&self) -> bool {
        self.allocation_contracts.is_conflicting()
    }
}
