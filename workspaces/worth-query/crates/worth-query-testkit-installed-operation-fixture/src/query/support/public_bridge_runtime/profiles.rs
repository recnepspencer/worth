pub use worth_query::facade::runtime::{
    WorthQueryAuthorityLane, WorthQueryRuntimeFacadeFamily, WorthQueryRuntimeFamilySupport,
    WorthQueryRuntimeSupportProfile,
};

pub fn public_graph_support_profile() -> WorthQueryRuntimeSupportProfile {
    WorthQueryRuntimeSupportProfile::bridge_backed(
        "public-graph-subscription-activation",
        "public-graph-preview-basis",
        "public-graph-inspector-evidence",
    )
    .with_family_support(WorthQueryRuntimeFamilySupport::supported(
        WorthQueryRuntimeFacadeFamily::Write,
        [WorthQueryAuthorityLane::AuthoritativeTruth],
        [],
        ["public-graph-write-authority"],
    ))
}
