//! External business compensation retains its explicit reconciliation requirement.
use super::*;

#[test]
fn external_correction_requires_compensation_reconciliation_and_exact_action() {
    let source = MutationBinding::descriptor();
    let inventory = BTreeMap::from([(source.identity().to_owned(), &source)]);
    let queries = [disclosure::HistoryBinding::descriptor()];
    for (authority, reconciliation, compensation, slot, valid) in [
        (
            DeclaredCorrectionAuthority::RuntimeWithExternalOwner,
            true,
            true,
            source.operation_name(),
            true,
        ),
        (
            DeclaredCorrectionAuthority::RuntimeWithExternalOwner,
            false,
            true,
            source.operation_name(),
            false,
        ),
        (
            DeclaredCorrectionAuthority::RuntimeWithExternalOwner,
            true,
            true,
            "foreign-inverse",
            false,
        ),
        (
            DeclaredCorrectionAuthority::RuntimeWithExternalOwner,
            true,
            false,
            source.operation_name(),
            false,
        ),
        (
            DeclaredCorrectionAuthority::RuntimeAlone,
            true,
            true,
            source.operation_name(),
            false,
        ),
        (
            DeclaredCorrectionAuthority::NotCorrectable,
            true,
            true,
            source.operation_name(),
            false,
        ),
    ] {
        let mechanism = if compensation {
            PortableCorrectionMechanism::Compensation(
                DeclaredCompensation::new(
                    slot,
                    DeclaredAftermathPostcondition::BusinessPostcondition {
                        identity: "completed-journal-compensated".to_owned(),
                    },
                )
                .unwrap(),
            )
        } else {
            PortableCorrectionMechanism::RecordedInverse(
                PortableRecordedInverse::from_untrusted_fields(
                    slot.to_owned(),
                    DeclaredLoweringCorrespondenceRef::new("fixture.restore.v1").unwrap(),
                    DeclaredAftermathPostcondition::ExactPriorTruth,
                    PortablePreImageDemand::from_untrusted_fields(Vec::new(), 0),
                ),
            )
        };
        for opt_in in [
            None,
            Some("confirm-original-settlement"),
            Some("different-owner-procedure"),
        ] {
            let mut association = parts(&source);
            association.completed_external_reconciliation =
                opt_in.map(|procedure| DeclaredReconciliationProcedure::new(procedure).unwrap());
            let contract = PortableApplicationAftermathContract::from_untrusted_fields(
                authority,
                Some(mechanism.clone()),
                reconciliation.then(|| {
                    DeclaredReconciliationProcedure::new("confirm-original-settlement").unwrap()
                }),
            )
            .with_untrusted_correction_binding(
                DeclaredCorrectionBinding::from_untrusted_parts(association),
            );
            let members = [ApplicationSchemaMember::OperationAftermath {
                operation: source.operation_name().to_owned(),
                contract,
            }];
            let result = resolve_correction_binding(&members, &source, &inventory, &queries);
            if valid && opt_in == Some("confirm-original-settlement") {
                assert_eq!(
                    result.unwrap().unwrap().inverse_type(),
                    TypeId::of::<MutationBinding>()
                );
            } else {
                assert_eq!(result.unwrap_err().kind(), crate::application_operation::WorthQueryApplicationOperationInstallationDenialKind::AftermathInstallationDenied);
            }
        }
    }
}
