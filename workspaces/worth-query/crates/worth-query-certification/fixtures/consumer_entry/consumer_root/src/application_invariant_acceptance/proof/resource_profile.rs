use worth_query_host::facade::{
    application_entry::{WorthQueryApplicationRequestExt, WorthQueryApplicationRequestQueryDenial},
    domain::WorthQueryInstalledApplicationSchema,
    primary_graph::WorthQueryApplicationOneShotDenialKind,
};

use super::{authentication, installation, ConsumerSchema};
use worth_query_topology_entry::PlanarRead;

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
