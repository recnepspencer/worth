use worth_query_host::facade::{
    application_entry::{WorthQueryApplicationRequestExt, WorthQueryApplicationRequestQueryDenial},
    application_installation::WorthQueryInMemoryApplicationDenial,
    domain::WorthQueryInstalledApplicationSchema,
    primary_graph::{
        WorthQueryApplicationOneShotDenialKind, WorthQueryPrimaryGraphInstallationDenialKind,
    },
};

use super::{authentication, installation, ConsumerSchema};
use worth_query_topology_entry::PlanarRead;

pub(super) fn candidate_bytes_beyond_host_limit_are_denied(
    foreign: &WorthQueryInstalledApplicationSchema<ConsumerSchema>,
) {
    // The program's producers run the adjustment with its 8192-byte candidate
    // requirement, so a host one byte short publishes no application.
    let Err(denial) = installation::install_with_candidate_bytes(foreign, 8191) else {
        panic!("8192 declared candidate bytes must exceed the 8191-byte host")
    };
    let WorthQueryInMemoryApplicationDenial::Contributions(denial) = denial else {
        panic!("the producer's graph work must deny the installation: {denial:?}")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryPrimaryGraphInstallationDenialKind::ProducerGraphWorkRejected,
        "{denial:?}"
    );
}

pub(super) fn source_footprint_bytes_beyond_host_limit_are_denied(
    foreign: &WorthQueryInstalledApplicationSchema<ConsumerSchema>,
) {
    let world = installation::install_with_query_bytes(foreign, 1400);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the bounded host authenticates the same declared principal");
    let request = world.application.request(&principal, &scope);
    let outcome = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute();
    let Err(denial) = outcome else {
        panic!("the retained source footprint must obey the result-byte reservation")
    };
    let WorthQueryApplicationRequestQueryDenial::Execution(denial) = denial else {
        panic!("source footprint bytes must deny at bounded execution: {denial:?}")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationOneShotDenialKind::ResultBufferLimitExceeded
    );
    // The complete root-selection source now reserves its bounded negative
    // evidence before relation materialization reaches the first child row.
    assert_eq!(denial.subject(), "root");
    assert_eq!(
        world
            .application
            .result_buffer_observer()
            .observe()
            .retained_bytes(),
        0
    );
}
