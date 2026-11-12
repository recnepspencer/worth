//! Public-entry requests with genuine Query admission; no runtime is built to inspect binding.
pub(super) fn request(location: &std::path::Path) -> crate::ApplicationHomeRequest {
    request_with_build(
        location,
        worth_foundational::facade::ApplicationBuildIdentity::parse(
            "11111111-1111-4111-8111-111111111111",
        )
        .unwrap(),
    )
}

pub(super) fn request_with_build(
    location: &std::path::Path,
    build: worth_foundational::facade::ApplicationBuildIdentity,
) -> crate::ApplicationHomeRequest {
    let binding = worth_query_testkit_installation::facade::installation::application_home_binding::admitted_home_binding();
    crate::ApplicationHomeRequest::new(
        location.into(),
        super::native_schema::definition(),
        binding,
        build,
    )
}
