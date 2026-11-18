pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::release_envelope::*;

#[test]
fn current_release_envelope_is_deterministic_frozen_and_decodable() {
    golden::assert_current_envelope_is_deterministic_frozen_and_decodable();
}
