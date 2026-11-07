mod declaration_dx;
mod drift_localization;
mod evaluation_success;
mod hostile_terminal_document;
mod rejection;

use crate::consumer_kit::support_snapshot::{project_support_snapshot, WorthQuerySupportSnapshot};
use crate::runtime::{
    WorthQueryRuntimeFamilySupport, WorthQueryRuntimePublicApiContract,
    WorthQueryRuntimePublicSupportMatrix, WorthQueryRuntimeSupportProfile,
};

fn scaffold_snapshot() -> WorthQuerySupportSnapshot {
    snapshot_from_profile(WorthQueryRuntimeSupportProfile::scaffold_backend_profile())
}

fn snapshot_from_profile(profile: WorthQueryRuntimeSupportProfile) -> WorthQuerySupportSnapshot {
    let contract = WorthQueryRuntimePublicApiContract::from_support_profile(&profile)
        .expect("fixture digest must fit");
    let matrix = WorthQueryRuntimePublicSupportMatrix::from_public_api_contract(&contract)
        .expect("fixture digest must fit");
    project_support_snapshot(&matrix).expect("fixture digest must fit")
}

fn write_deferred_snapshot() -> WorthQuerySupportSnapshot {
    snapshot_from_profile(
        WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
            WorthQueryRuntimeFamilySupport::deferred(
                crate::runtime::WorthQueryRuntimeFacadeFamily::Write,
                "phase five hostile drift fixture",
            ),
        ),
    )
}

fn empty_family_snapshot() -> WorthQuerySupportSnapshot {
    snapshot_from_profile(WorthQueryRuntimeSupportProfile::new(std::iter::empty::<
        WorthQueryRuntimeFamilySupport,
    >()))
}
