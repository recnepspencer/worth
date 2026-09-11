use crate::facade::application_schema::{ApplicationSchemaMember, OperationRequiresAbility};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransferInput;
worth_query_portable_type!(TransferInput => "worth.query.test.transfer-input");
worth_query_structured_value_binding!(
    pub TransferInputBinding for TransferInput {
        identity: <TransferInput as crate::facade::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_NAME
    }
);

worth_query_entity!(pub Account for AbilitySchema);
worth_query_operation!(
    pub TransferOperation for AbilitySchema,
    input TransferInputBinding
);
worth_query_ability!(pub SendMoney scoped_to Account, in AbilitySchema);
worth_query_operation_requires!(TransferOperation => [SendMoney]);

worth_query_application_schema! {
    pub schema AbilitySchema {
        owner: "WORTH.tests.ability",
        version: (1, 0),
        members: |schema| {
            schema
                .entity(Account::reference())
                .operation(
                    TransferOperation::reference()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish(),
                )
                .ability(SendMoney::reference())
                .operation_requires_ability(
                    TransferOperation::reference(),
                    SendMoney::reference(),
                )
        }
    }
}

#[test]
fn typed_ability_and_operation_requirement_enter_canonical_schema_meaning() {
    let declaration = AbilitySchema::declaration().expect("ability schema must declare");
    assert!(declaration.erased().members().iter().any(|member| matches!(
        member,
        ApplicationSchemaMember::Ability {
            ability,
            scope_entity,
        } if ability == "SendMoney" && scope_entity == "Account"
    )));
    assert!(declaration.erased().members().iter().any(|member| matches!(
        member,
        ApplicationSchemaMember::OperationAbility {
            operation,
            ability,
            scope_entity,
        } if operation == "TransferOperation"
            && ability == "SendMoney"
            && scope_entity == "Account"
    )));
}

#[test]
fn operation_requirement_is_a_compile_time_relationship() {
    fn requires<Operation, Ability: OperationRequiresAbility<Operation>>() {}
    requires::<TransferOperation, SendMoney>();
}
