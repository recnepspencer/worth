mod support;

mod ordinary_reads {
    #[cfg(feature = "test-request-lifetime-probes")]
    mod advancement_custody;
    mod application_query;
    mod authority;
    mod canonical_scale_fixture;
    mod canonical_work_scale;
    mod estate;
    mod estate_fixture;
    mod fixture;
    mod locality;
    mod pending_payments;
}
