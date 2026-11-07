use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::facade::application_aftermath::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectProtocol, ApplicationSchemaMember,
    WorthQueryExternalEffectCorrelationFamily,
};

use super::{text, type_id};

pub(super) fn external_effect() -> ApplicationSchemaMember {
    ApplicationSchemaMember::OperationExternalEffect {
        operation: text("Apply"),
        effect: text("Changed"),
        rust_payload_type: type_id("effect-payload"),
        protocol: ApplicationExternalEffectProtocol::new(
            BoundaryProtocolIdentity::parse("archive.tests.changed").unwrap(),
            BoundaryProtocolVersion::new(2),
        ),
        maximum_payload_bytes: 4096,
        correlation_family: WorthQueryExternalEffectCorrelationFamily::new("changed-family")
            .unwrap(),
    }
}

pub(super) fn aftermath_contract() -> PortableApplicationAftermathContract {
    PortableApplicationAftermathContract::from_untrusted_fields(
        DeclaredCorrectionAuthority::RuntimeWithExternalOwner,
        Some(PortableCorrectionMechanism::Compensation(
            DeclaredCompensation::new(
                "Compensate",
                DeclaredAftermathPostcondition::BusinessPostcondition {
                    identity: text("neutralized"),
                },
            ),
        )),
        Some(DeclaredReconciliationProcedure::new("reconcile")),
    )
}

pub(super) fn inverse_aftermath_contract() -> PortableApplicationAftermathContract {
    let demand = PortablePreImageDemand::from_untrusted_fields(
        vec![PortablePreImageLocus::from_untrusted_fields(
            text("Entity"),
            text("Aspect"),
            text("field"),
        )],
        512,
    );
    let inverse = PortableRecordedInverse::from_untrusted_fields(
        text("Reverse"),
        DeclaredLoweringCorrespondenceRef::new("reverse-correspondence"),
        DeclaredAftermathPostcondition::ExactPriorTruth,
        demand,
    );
    PortableApplicationAftermathContract::from_untrusted_fields(
        DeclaredCorrectionAuthority::RuntimeAlone,
        Some(PortableCorrectionMechanism::RecordedInverse(inverse)),
        None,
    )
}
