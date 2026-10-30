//! Read scopes install each schema's exact owner-declared operation and fields.
use super::*;
use worth_query_declaration::facade::application_operation::*;

worth_query_declaration::worth_query_structured_value_binding!(ReadInputBinding for () { identity: "worth.rust.unit" });

mod scalar {
    use super::*;
    worth_query_declaration::worth_query_application_schema! {
        schema Schema {
            owner: ScalarRead, version: (1, 0),
            members: |schema| { schema.entity(Record::reference())
                .aspect(Record::reference(), Facts::reference())
                .scalar_aspect_field(Record::reference(), Value::reference())
                .operation(Read::reference().definition().no_external_effect().no_aftermath().finish())
                .operation_read_field(Read::reference(), Value::reference()) }
        }
    }
    worth_query_declaration::worth_query_entity!(Record for Schema);
    worth_query_declaration::worth_query_aspect!(Facts for Schema, Record; identity = AspectIdentity(0x971002), revision = AspectContractRevision(1));
    worth_query_declaration::worth_query_field!(Value for Schema, Record, Facts: u64 => U64ApplicationValueBinding, read_write, no_equality);
    worth_query_declaration::worth_query_operation!(Read for Schema, input ReadInputBinding);
    worth_query_declaration::worth_query_operation_reads!(Read => [Value]);

    pub(super) fn assert_scope() {
        assert_read(Read::reference(), true);
    }
}

mod structure {
    use super::*;
    worth_query_declaration::worth_query_application_schema! {
        schema Schema {
            owner: StructRead, version: (1, 0),
            members: |schema| { schema.entity(Record::reference())
                .aspect(Record::reference(), Facts::reference())
                .field(Record::reference(), Value::reference())
                .operation(Read::reference().definition().no_external_effect().no_aftermath().finish())
                .operation_read_field(Read::reference(), Value::reference()) }
        }
    }
    worth_query_declaration::worth_query_entity!(Record for Schema);
    worth_query_declaration::worth_query_aspect!(Facts for Schema, Record; identity = AspectIdentity(0x971003), revision = AspectContractRevision(1));
    worth_query_declaration::worth_query_field!(Value for Schema, Record, Facts: u64 => U64ApplicationValueBinding, read_write, no_equality);
    worth_query_declaration::worth_query_operation!(Read for Schema, input ReadInputBinding);
    worth_query_declaration::worth_query_operation_reads!(Read => [Value]);

    pub(super) fn assert_scope() {
        assert_read(Read::reference(), false);
    }
}

pub(super) fn assert_declared_scopes() {
    scalar::assert_scope();
    structure::assert_scope();
}

fn assert_read<Schema, Operation>(
    operation: ApplicationOperationRef<Schema, Operation, ()>,
    whole: bool,
) where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = ()>,
{
    let declaration = Schema::declaration().unwrap();
    assert_portable_mask(declaration.erased(), whole);
    let package = crate::facade::WorthQueryPortableDomainPackage::new(
        crate::facade::WorthQueryPortableDomainIdentity::new(
            Schema::OWNER,
            Schema::MAJOR,
            Schema::MINOR,
        ),
    )
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted =
        crate::facade::WorthQueryInstallationAdmissionProfile::new("support", "configuration")
            .admit(package)
            .unwrap();
    let installed = crate::facade::WorthQueryInstalledPackageIndex::build(
        crate::facade::WorthQueryInstallationRuntimeIdentity::fresh(),
        crate::facade::WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(declaration)
    .unwrap();
    let operation = installed.installed_operation(operation).unwrap();
    let [crate::domain_operation::WorthQueryOperationGraphReadScope::NativeProjection(scope)] =
        operation.contracts().graph_reads().roles()[0].read_scopes()
    else {
        panic!("installed Value read retains its exact Native scope");
    };
    assert!(scope.admits_field(&FieldKey::new("Value").unwrap()));
    assert!(!scope.admits_field(&FieldKey::new("Sibling").unwrap()));
    assert!(!scope.admits_field(&FieldKey::new("Facts").unwrap()));
    assert_eq!(scope.atomic_field_count(), Some(1));
    if !whole {
        let native = installed
            .native_contracts()
            .aspect("Record", "Facts")
            .unwrap();
        let projection =
            crate::domain_operation::WorthQueryOperationNativeProjectionContract::from_installed(
                native,
                aspects().projection_mask().whole_aspect(),
            )
            .unwrap();
        let scope = crate::domain_operation::WorthQueryOperationApplicationProjectionScope::new(
            installed.binding_identity(),
            "Record".to_owned(),
            native.contract().key().clone(),
            projection,
            native.binding().clone(),
        );
        assert!(!scope.admits_field(&FieldKey::new("Value").unwrap()));
        assert_eq!(scope.atomic_field_count(), None);
    }
}

fn assert_portable_mask(declaration: &ErasedApplicationSchemaDeclaration, whole: bool) {
    let native = compile_portable_native_contract_records(declaration).unwrap();
    let operations = crate::application_operation::compile_portable_operation_contract_records(
        declaration,
        &native,
    )
    .unwrap();
    assert_eq!(operations.len(), 1);
    let [crate::package::WorthQueryPortableOperationGraphReadScope::NativeProjection {
        mask, ..
    }] = operations[0].graph_reads()
    else {
        panic!("the declared Value read has one actual Native projection");
    };
    assert_eq!(mask.is_whole_aspect(), whole);
    if !whole {
        assert_eq!(
            mask,
            &aspects().projection_mask().fields(["Value"]).unwrap()
        );
    }
}
