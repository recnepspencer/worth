//! Real filesystem Fresh permission and the native installation owner.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{runtime::*, schema::*};
use worth_store_application::*;

pub(super) fn open(
    path: &std::path::Path,
    request: &ExecutionRequest<'_, '_>,
) -> UnrecoveredApplicationHome {
    let inputs = worth_store_testkit_application_oracle::home_opening::request(
        path,
        RelationalRuntimeApi::builder(),
    );
    let admitted = admit_home(recover_home(inputs, request).unwrap(), request).unwrap();
    attach_relational(admitted, request)
        .unwrap_or_else(|_| panic!("the actual fresh Native owner must attach"))
}
pub(super) struct Attempt {
    pub(super) result: Result<
        (
            RelationalInitialSchemaInstallationReceipt,
            RelationalRuntime,
        ),
        RelationalInitialSchemaInstallationDenial,
    >,
    pub(super) before: SchemaAuthoritySnapshot,
    pub(super) after: SchemaAuthoritySnapshot,
}
pub(super) fn install(
    home: &mut UnrecoveredApplicationHome,
    setup: &ExecutionRequest<'_, '_>,
    request: &ExecutionRequest<'_, '_>,
) -> Attempt {
    let preparation = home.prepare_native_schema_for_home(setup).unwrap();
    let ApplicationHomeNativeSchemaPreparation::Fresh {
        native,
        initialization,
    } = preparation
    else {
        panic!("the filesystem owner must prove actual absence");
    };
    let before = native.config().schema.registry.authority_snapshot();
    let additions = super::source_metadata::native_registry(&crate::schema_transition::registry(
        crate::schema_transition::SOURCE,
    ));
    let result = native
        .prepare_fresh_home_schema_installation(request, initialization)
        .and_then(|installation| {
            installation.install_with_managed_sources(request, additions, Vec::new(), Vec::new())
        });
    let after = native.config().schema.registry.authority_snapshot();
    let result = result.map(|receipt| {
        let fork = native
            .fork()
            .expect("the installed, settled Native owner permits its ordinary runtime fork");
        (receipt, fork)
    });
    Attempt {
        result,
        before,
        after,
    }
}

pub(super) fn release(
    home: UnrecoveredApplicationHome,
    receipt: RelationalInitialSchemaInstallationReceipt,
    fork: RelationalRuntime,
    memory: &worth_execution::SerialMemoryBudget,
) {
    let second = fork
        .fork()
        .expect("a real runtime fork shares the installed native inventory");
    drop(receipt);
    // Abandon the actual pre-service Home. No close acknowledgement is claimed
    // and no retained capability is fabricated or manually stripped of ports.
    drop(home);
    let retained = memory.charged_memory_bytes();
    assert!(retained > 0);
    drop(fork);
    assert_eq!(memory.charged_memory_bytes(), retained);
    drop(second);
    assert_eq!(memory.charged_memory_bytes(), 0);
}
