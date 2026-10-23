//! Resolve descriptive correction associations against actual native inventory.
use super::*;
use crate::application_aftermath::resolve_correction_binding;
mod disclosure;
use std::{any::TypeId, collections::BTreeMap};
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::{
    application_aftermath::*, application_operation::ApplicationMutationBindingDescriptor,
    application_schema::ApplicationSchemaMember, portable_identity::WorthQueryPortableTypeIdentity,
};

#[test]
fn correction_binding_resolves_exact_native_members_and_rejects_changed_meaning() {
    let source = MutationBinding::descriptor();
    let inventory = BTreeMap::from([(source.identity().to_owned(), &source)]);
    let source_members = members(&source, parts(&source));
    let queries = [disclosure::HistoryBinding::descriptor()];
    let installed = resolve_correction_binding(&source_members, &source, &inventory, &queries)
        .unwrap()
        .unwrap();
    assert_eq!(installed.source_type(), TypeId::of::<MutationBinding>());
    assert_eq!(installed.inverse_type(), TypeId::of::<MutationBinding>());

    for axis in [
        "source-codec",
        "inverse-codec",
        "source-handler",
        "inverse-handler",
        "source-query",
        "inverse-query",
        "inverse-membership",
    ] {
        let mut altered = parts(&source);
        match axis {
            "source-codec" => {
                let mut description = altered.source.into_parts();
                description.input_identity = identity("different-source-input");
                altered.source = worth_query_declaration::facade::application_operation::ApplicationMutationDescription::from_untrusted_parts(description);
            }
            "inverse-codec" => {
                let mut description = altered.inverse.into_parts();
                description.result_identity = identity("different-inverse-result");
                altered.inverse = worth_query_declaration::facade::application_operation::ApplicationMutationDescription::from_untrusted_parts(description);
            }
            "source-handler" => altered.source_handler = "foreign-source-handler".to_owned(),
            "inverse-handler" => altered.inverse_handler = "foreign-inverse-handler".to_owned(),
            "source-query" => altered.source_query = Some("foreign-source-query".to_owned()),
            "inverse-query" => altered.inverse_query = Some("foreign-inverse-query".to_owned()),
            "inverse-membership" => {
                let mut description = altered.inverse.into_parts();
                description.binding_identity = identity("uninstalled-inverse");
                altered.inverse = worth_query_declaration::facade::application_operation::ApplicationMutationDescription::from_untrusted_parts(description);
            }
            _ => unreachable!(),
        }
        assert_eq!(resolve_correction_binding(&members(&source, altered), &source, &inventory, &queries).unwrap_err().kind(),
            crate::application_operation::WorthQueryApplicationOperationInstallationDenialKind::AftermathInstallationDenied,
            "{axis}");
    }
}

fn parts(source: &ApplicationMutationBindingDescriptor) -> DeclaredCorrectionBindingParts {
    DeclaredCorrectionBindingParts {
        identity: identity("fixture.restore.v1"),
        source: source.description().clone(),
        inverse: source.description().clone(),
        source_handler: source.handler().identity().to_owned(),
        inverse_handler: source.handler().identity().to_owned(),
        source_query: source.source_query_identifier().map(str::to_owned),
        inverse_query: source.source_query_identifier().map(str::to_owned),
        disclosure: DeclaredCorrectionDisclosure::of::<MutationSchema, disclosure::HistoryBinding>(
        ),
    }
}

#[test]
fn correction_compensation_resolves_its_actual_action_and_refuses_slot_or_authority_drift() {
    let source = MutationBinding::descriptor();
    let inventory = BTreeMap::from([(source.identity().to_owned(), &source)]);
    let queries = [disclosure::HistoryBinding::descriptor()];
    for (slot, authority, valid) in [
        (
            source.operation_name(),
            DeclaredCorrectionAuthority::RuntimeAlone,
            true,
        ),
        (
            "uninstalled-compensation",
            DeclaredCorrectionAuthority::RuntimeAlone,
            false,
        ),
        (
            source.operation_name(),
            DeclaredCorrectionAuthority::RuntimeWithExternalOwner,
            false,
        ),
    ] {
        let contract = PortableApplicationAftermathContract::from_untrusted_fields(
            authority,
            Some(PortableCorrectionMechanism::Compensation(
                DeclaredCompensation::new(
                    slot,
                    DeclaredAftermathPostcondition::BusinessPostcondition {
                        identity: "effect-neutralized".to_owned(),
                    },
                )
                .unwrap(),
            )),
            None,
        )
        .with_untrusted_correction_binding(
            DeclaredCorrectionBinding::from_untrusted_parts(parts(&source)),
        );
        let members = [ApplicationSchemaMember::OperationAftermath {
            operation: source.operation_name().to_owned(),
            contract,
        }];
        let result = resolve_correction_binding(&members, &source, &inventory, &queries);
        if valid {
            let binding = result
                .unwrap()
                .expect("the installed compensating action resolves");
            assert_eq!(binding.inverse_type(), TypeId::of::<MutationBinding>());
        } else {
            assert_eq!(result.unwrap_err().kind(), crate::application_operation::WorthQueryApplicationOperationInstallationDenialKind::AftermathInstallationDenied);
        }
    }
}

fn members(
    source: &ApplicationMutationBindingDescriptor,
    parts: DeclaredCorrectionBindingParts,
) -> Vec<ApplicationSchemaMember> {
    let inverse = PortableRecordedInverse::from_untrusted_fields(
        source.operation_name().to_owned(),
        DeclaredLoweringCorrespondenceRef::new("fixture.restore.v1").unwrap(),
        DeclaredAftermathPostcondition::ExactPriorTruth,
        PortablePreImageDemand::from_untrusted_fields(
            vec![PortablePreImageLocus::from_untrusted_fields(
                "Account".to_owned(),
                "State".to_owned(),
                "amount".to_owned(),
            )],
            64,
        ),
    );
    vec![ApplicationSchemaMember::OperationAftermath {
        operation: source.operation_name().to_owned(),
        contract: PortableApplicationAftermathContract::from_untrusted_fields(
            DeclaredCorrectionAuthority::RuntimeAlone,
            Some(PortableCorrectionMechanism::RecordedInverse(inverse)),
            None,
        )
        .with_untrusted_correction_binding(DeclaredCorrectionBinding::from_untrusted_parts(parts)),
    }]
}

fn identity(value: &str) -> WorthQueryPortableTypeIdentity {
    WorthQueryPortableTypeIdentity::from_untrusted(value.to_owned())
}
