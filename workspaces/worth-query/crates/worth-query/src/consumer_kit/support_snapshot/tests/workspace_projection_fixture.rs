use crate::runtime::{
    WorthQueryRuntimeBackendPosture, WorthQueryRuntimePublicApiContract,
    WorthQueryRuntimePublicSupportMatrix, WorthQueryRuntimeSupportProfile, WorthQueryWorkspace,
};

pub(in super::super::tests) fn live_support_matrix() -> WorthQueryRuntimePublicSupportMatrix {
    WorthQueryRuntimePublicSupportMatrix::from_public_api_contract(
        &WorthQueryRuntimePublicApiContract::from_support_profile(&primary_support_profile()),
    )
}

pub(in super::super::tests) fn support_snapshot_workspace() -> WorthQueryWorkspace {
    crate::consumer_kit::test_backend::in_memory_test_runtime()
        .with_schema(crate::consumer_kit::test_backend::task_schema())
        .support_profile(primary_support_profile())
        .workspace("support-snapshot-real-workspace")
        .expect("support snapshot workspace should build from the real in-memory Product owner")
}

pub(in super::super::tests) fn scaffold_support_matrix() -> WorthQueryRuntimePublicSupportMatrix {
    WorthQueryRuntimePublicSupportMatrix::from_public_api_contract(
        &WorthQueryRuntimePublicApiContract::from_support_profile(
            &WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ),
    )
}

fn primary_support_profile() -> WorthQueryRuntimeSupportProfile {
    WorthQueryRuntimeSupportProfile::scaffold_backend_profile()
        .with_posture(WorthQueryRuntimeBackendPosture::Primary)
}
