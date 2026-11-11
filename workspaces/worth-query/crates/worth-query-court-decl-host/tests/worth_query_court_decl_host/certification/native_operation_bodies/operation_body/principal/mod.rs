pub(crate) use worth_query_testkit_certification_operation::facade::certification::native_operation_bodies::operation_body::principal::*;

#[test]
fn admitted_actor_survives_reconstruction_and_cannot_be_chosen_by_input() {
    let old = install::<36>();
    let checkpoint = old
        .with_host_advancement(|phase| {
            old.runtime()
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    drop(old);
    let schema = program::schema::<36>();
    let restored = WorthQueryInterimCheckpointRestore::program(
        program::validated::<36>(&schema),
        schema,
        (program::Installation::Native,),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        checkpoint,
    )
    .unwrap();
    // invoke supplies retention_days=99. The admitted native principal is 1.
    let Outcome::Committed {
        receipt: original, ..
    } = invoke(&restored, 360)
    else {
        panic!("the admitted actor commits the original operation");
    };
    let Outcome::RecoveredCommitted(recorded) = invoke(&restored, 360) else {
        panic!("the actor's retry observes the recorded original");
    };
    worth_query_testkit_certification_canonical::facade::recorded_receipt_comparison::assert_original(
        &recorded, &original,
    );
    let scope = request_scope();
    let principal = authenticate_operator(restored.runtime().installed_schema(), &scope);
    let result = restored
        .request(&principal, &scope)
        .query(audit_query::AuditRead {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(result.rows().len(), 1);
    let row = &result.rows()[0];
    assert_eq!(
        (row.actor, row.retention_days, row.prior_retention_days),
        (Some(1), 36, 7)
    );
    assert_eq!(read_retention(&restored), 36);
}

#[test]
fn portable_actor_contract_refuses_missing_read_unit_codec_and_binding() {
    let schema = program::schema::<36>();
    let portable = WorthQueryPortableApplicationSchemaRecord::project(schema.erased());
    assert_eq!(
        validate_portable_application_schema_freshly(portable.clone()).unwrap(),
        *schema.erased()
    );
    for defect in ["read", "unit", "codec", "binding", "principal", "source"] {
        let mut parts = portable.clone().into_parts();
        if defect == "unit"
            && !parts.members.iter().any(
                |member| matches!(member, ApplicationSchemaMember::Unit { unit } if unit == "days"),
            )
        {
            parts.members.push(ApplicationSchemaMember::Unit {
                unit: "days".to_owned(),
            });
        }
        for member in &mut parts.members {
            match member {
                ApplicationSchemaMember::OperationDecisionRead { operation, target }
                    if defect == "read"
                        && operation == "SetRetention"
                        && matches!(&*target, ApplicationOperationDecisionReadTarget::Field {field, ..} if field == "PrincipalIdentityField") =>
                {
                    *target = ApplicationOperationDecisionReadTarget::Entity {
                        entity: "Principal".to_owned(),
                    };
                }
                ApplicationSchemaMember::Field { field, unit, .. }
                    if defect == "unit" && field == "AuditActorField" =>
                {
                    *unit = Some("days".to_owned())
                }
                ApplicationSchemaMember::OperationBody { operation, body }
                    if operation == "SetRetention" =>
                {
                    let creation = body.creation().unwrap();
                    let mut fields = creation.fields().to_vec();
                    for field in &mut fields {
                        if let ApplicationOperationBodyCreatedField::PrincipalIdentity {
                            binding,
                            mutation_binding,
                            principal_binding,
                            source,
                            ..
                        } = field
                        {
                            match defect {
                                "codec" => {
                                    *binding = WorthQueryPortableTypeIdentity::from_untrusted(
                                        "wrong.codec".to_owned(),
                                    )
                                }
                                "binding" => {
                                    *mutation_binding =
                                        "worth.query.body-court.source-binding.v1".to_owned()
                                }
                                "principal" => *principal_binding = "missing-principal".to_owned(),
                                "source" => {
                                    *source = ApplicationFieldBindingLocus::new(
                                        "Document",
                                        DocumentRetentionField::at("DocumentRetentionField")
                                            .aspect(),
                                        "DocumentRetentionField",
                                    )
                                }
                                _ => {}
                            }
                        }
                    }
                    let creation = ApplicationOperationBodyCreation::from_untrusted(
                        creation.entity().to_owned(),
                        creation.key().to_owned(),
                        fields,
                    )
                    .with_untrusted_links(creation.links().to_vec());
                    *body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
                        body.scope_entity().unwrap().to_owned(),
                        body.scope_fields().to_vec(),
                        Some(creation),
                        body.requirements().to_vec(),
                    );
                }
                _ => {}
            }
        }
        let inserted_unit = parts.members.iter().position(
            |member| matches!(member, ApplicationSchemaMember::Unit { unit } if unit == "days"),
        );
        let unit_owner = parts.members.iter().position(|member| matches!(member, ApplicationSchemaMember::Field { field, .. } if field == "AuditActorField")).unwrap() as u32;
        let mut order: Vec<_> = (0..parts.members.len()).collect();
        order.sort_by(|left, right| parts.members[*left].cmp(&parts.members[*right]));
        for contribution in &mut parts.contributions {
            let mut ordinals: Vec<_> = contribution
                .member_ordinals()
                .iter()
                .map(|old| {
                    order
                        .iter()
                        .position(|ordinal| *ordinal == *old as usize)
                        .unwrap() as u32
                })
                .collect();
            if defect == "unit"
                && contribution.member_ordinals().contains(&unit_owner)
                && let Some(unit) = inserted_unit
            {
                let index = order.iter().position(|ordinal| *ordinal == unit).unwrap() as u32;
                if !ordinals.contains(&index) {
                    ordinals.push(index);
                }
            }
            ordinals.sort_unstable();
            *contribution = worth_query_host::facade::declaration::application_schema::ApplicationSchemaContributionProvenance::from_untrusted_parts(contribution.identity().as_str().to_owned(), ordinals);
        }
        parts.members.sort();
        let denial = validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts),
        )
        .unwrap_err();
        if defect == "read" {
            assert!(matches!(
                denial,
                ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency { .. }
            ));
        }
        if defect == "unit" {
            assert!(
                matches!(
                    denial,
                    ApplicationSchemaDeclarationDenial::MissingOperationProgramDependency { .. }
                ),
                "{defect}: {denial:?}"
            );
        }
    }
}
