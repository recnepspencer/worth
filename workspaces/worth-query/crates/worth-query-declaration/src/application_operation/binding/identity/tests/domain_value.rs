use serde::Serialize;

use crate::facade::application_operation::{
    application_value_identity, ApplicationValueIdentityDomain,
};

#[derive(Serialize)]
struct GroupIdentity<'a> {
    product: &'a str,
    axis: Option<u8>,
}

#[test]
fn application_values_keep_optional_values_and_namespaces_distinct() {
    let derive = |scope, axis| {
        application_value_identity(
            ApplicationValueIdentityDomain::DomainValue,
            scope,
            &GroupIdentity {
                product: "catalog-item",
                axis,
            },
        )
        .expect("the value serializes")
    };
    let absent = derive("example.group.v1", None);
    assert_eq!(absent, derive("example.group.v1", None));
    assert_ne!(
        absent.identity(),
        derive("example.group.v1", Some(2)).identity()
    );
    assert_ne!(
        absent.identity(),
        derive("example.group.v2", None).identity()
    );
    assert_ne!(
        absent.identity(),
        derive("example.other.v1", None).identity()
    );
    let host_intent = application_value_identity(
        ApplicationValueIdentityDomain::HostCommitIntent,
        "example.group.v1",
        &GroupIdentity {
            product: "catalog-item",
            axis: None,
        },
    )
    .expect("the host intent serializes");
    assert_ne!(absent.identity(), host_intent.identity());
}

#[test]
fn an_application_value_serializer_rejection_returns_no_digest() {
    let result = application_value_identity(
        ApplicationValueIdentityDomain::DomainValue,
        "example.group.v1",
        &super::Refuses,
    );
    assert!(matches!(
        result,
        Err(crate::application_schema::ApplicationValueEncodeDenial::CodecRejected { .. })
    ));
}
