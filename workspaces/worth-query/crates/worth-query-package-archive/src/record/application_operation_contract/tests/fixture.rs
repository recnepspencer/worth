use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, AspectMask, BoundaryProtocolIdentity, BoundaryProtocolVersion, CanonicalFieldPath,
    FieldDeclaration, FieldKey, FieldRequirement, ProjectionMask, ScalarAspectType,
    StructAspectShape,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectProtocol, WorthQueryExternalEffectCorrelationFamily,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;
use worth_query_installation::facade::{
    WorthQueryPortableApplicationOperationContractParts,
    WorthQueryPortableApplicationOperationContractRecord,
    WorthQueryPortableExternalEffectContractParts, WorthQueryPortableExternalEffectContractRecord,
    WorthQueryPortableInstalledReconciliationProcedureRecord,
    WorthQueryPortableOperationGraphReadScope as GraphRead,
    WorthQueryPortableOperationTouchScope as Touch, WorthQueryPortablePackageRecord,
};

pub(super) fn complete_record() -> WorthQueryPortableApplicationOperationContractRecord {
    WorthQueryPortableApplicationOperationContractRecord::from_untrusted_parts(complete_parts())
}

pub(super) fn wrapped_complete_record() -> WorthQueryPortablePackageRecord {
    wrapped(complete_record())
}

pub(super) fn wrapped(
    record: WorthQueryPortableApplicationOperationContractRecord,
) -> WorthQueryPortablePackageRecord {
    WorthQueryPortablePackageRecord::ApplicationOperationContract(record)
}

pub(super) fn complete_parts() -> WorthQueryPortableApplicationOperationContractParts {
    let contract = contract();
    WorthQueryPortableApplicationOperationContractParts {
        schema: "ArchiveOperationSchema".to_owned(),
        operation: "settle".to_owned(),
        input_type: WorthQueryPortableTypeIdentity::from_untrusted(
            "archive.operation.input".to_owned(),
        ),
        graph_reads: vec![
            GraphRead::Entity {
                schema: "ArchiveOperationSchema".to_owned(),
                entity: "Account".to_owned(),
            },
            GraphRead::NativeProjection {
                schema: "ArchiveOperationSchema".to_owned(),
                entity: "Account".to_owned(),
                aspect: AspectKey::new("Profile").unwrap(),
                contract: contract.clone(),
                mask: AspectMask::<ProjectionMask>::new([field_path()]),
            },
            GraphRead::Relation {
                schema: "ArchiveOperationSchema".to_owned(),
                relation: "Owner".to_owned(),
                from: "Account".to_owned(),
                to: "Principal".to_owned(),
            },
        ],
        touches: vec![
            Touch::CreateEntity {
                schema: "ArchiveOperationSchema".to_owned(),
                entity: "Audit".to_owned(),
            },
            Touch::DeleteEntity {
                schema: "ArchiveOperationSchema".to_owned(),
                entity: "Draft".to_owned(),
            },
            Touch::WriteField {
                schema: "ArchiveOperationSchema".to_owned(),
                entity: "Account".to_owned(),
                contract,
                field_path: field_path(),
            },
            Touch::LinkRelation {
                schema: "ArchiveOperationSchema".to_owned(),
                relation: "Owner".to_owned(),
                from: "Account".to_owned(),
                to: "Principal".to_owned(),
            },
            Touch::UnlinkRelation {
                schema: "ArchiveOperationSchema".to_owned(),
                relation: "PreviousOwner".to_owned(),
                from: "Account".to_owned(),
                to: "Principal".to_owned(),
            },
        ],
        emissions: vec!["audit".to_owned(), "notify".to_owned()],
        body: None,
        external_effect: Some(external_effect()),
        reconciliation: Some(
            WorthQueryPortableInstalledReconciliationProcedureRecord::from_untrusted_procedure_slot(
                "settle-reconciliation".to_owned(),
            ),
        ),
    }
}

fn contract() -> AspectContract {
    let field = FieldDeclaration::new(
        FieldKey::new("name").unwrap(),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap();
    AspectContract::struct_aspect(
        AspectKey::new("Profile").unwrap(),
        AspectIdentity(0x9162_1200),
        AspectContractRevision(2),
        StructAspectShape::new([field]).unwrap(),
    )
}

fn field_path() -> CanonicalFieldPath {
    CanonicalFieldPath::new([FieldKey::new("name").unwrap()]).unwrap()
}

fn external_effect() -> WorthQueryPortableExternalEffectContractRecord {
    WorthQueryPortableExternalEffectContractRecord::from_untrusted_parts(
        WorthQueryPortableExternalEffectContractParts {
            correlation_family: WorthQueryExternalEffectCorrelationFamily::new("dispatch-rail")
                .unwrap(),
            effect: "payment".to_owned(),
            payload_type: WorthQueryPortableTypeIdentity::from_untrusted(
                "archive.effect.payload".to_owned(),
            ),
            protocol: ApplicationExternalEffectProtocol::new(
                BoundaryProtocolIdentity::parse("archive.payment").unwrap(),
                BoundaryProtocolVersion::try_new(2).unwrap(),
            ),
            maximum_payload_bytes: 1_024,
        },
    )
}
