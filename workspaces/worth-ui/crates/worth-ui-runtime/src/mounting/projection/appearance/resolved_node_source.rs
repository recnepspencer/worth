pub(crate) struct UiResolvedAppearanceNodeSource<'source> {
    pub(crate) issuer: worth_ui_host_contract::UiMountedNodeReceiptIssuer,
    pub(crate) semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    pub(crate) node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    pub(crate) graph_node: crate::graph::UiGraphNodeIdentity,
    pub(crate) plan_digest: u64,
    pub(crate) allocation: worth_ui_host_contract::UiMountedAllocationProjection,
    pub(crate) clip: super::UiMountedAppearanceClip,
    pub(crate) surface_paint_order: Option<u32>,
    pub(crate) geometry_input: Option<super::UiMountedAppearanceGeometryInput>,
    pub(crate) text_foreground_spans: &'source [super::UiMountedAppearanceTextSpanInput],
    pub(crate) projection: &'source crate::runtime::appearance::UiAppearanceProjection,
    pub(crate) outline_fringe: Result<
        worth_ui_host_contract::UiAppearanceLogicalLength,
        super::UiMountedAppearanceLoweringDenial,
    >,
}
