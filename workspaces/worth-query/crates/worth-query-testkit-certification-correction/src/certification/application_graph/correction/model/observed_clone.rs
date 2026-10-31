//! Clone consumes parent fields; its recorded inverse retires the created child.
use super::{
    authored_identity_correction::Family,
    authored_identity_model::{
        Completion, Created, CreatedOutputs, Restore, Retire, Retired, Target,
    },
    edit::{DenialBinding, ResultBinding},
    query,
    schema::*,
};
use worth_query_decl::facade::worth_query_mutation_binding;
use worth_query_host::facade::{
    declaration::{
        application_aftermath::*, application_operation::*, application_program::*,
        application_query::*, application_schema::*,
    },
    primary_graph::*,
    *,
};

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct CloneNumber {
    pub key: String,
    pub name: String,
}
impl CloneNumber {
    fn scope_key(input: &Self) -> String {
        input.key.clone()
    }
}
worth_query_structured_value_binding!(pub InputBinding for CloneNumber { identity: "numeric-observed-clone.input.v1" });
worth_query_operation!(pub CloneOperation for NumericSchema, input InputBinding);
worth_query_operation_reads!(CloneOperation => [Number, Key, Primary, Secondary, ReadEnabled, Ceiling, Calculated]);
worth_query_operation_writes!(CloneOperation => [Key, Primary, Secondary, ReadEnabled, Ceiling, Calculated]);
impl OperationCreates<CloneOperation> for Number {}

pub struct Snapshot {
    pub(super) parent: Target,
    pub(super) image: Restore,
}
worth_query_mutation_binding!(
    pub Binding for CloneNumber, schema NumericSchema,
    identity "numeric-observed-clone.binding.v1", input InputBinding,
    operation CloneOperation, result ResultBinding,
    idempotency String, identity "numeric-observed-clone.command.v1",
    decision Snapshot, denial DenialBinding,
    handler identity "numeric-observed-clone.handler.v1", outputs CreatedOutputs,
    principal ActorBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Number, NumberFacts, Key, String, ReadOnly, NoApplicationUnit,
    field Key::reference(), value CloneNumber::scope_key,
);
impl ApplicationDeclaredOperation<NumericSchema> for CloneOperation {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
        schema
            .operation_projection_work_budget(Self::reference(), 16)
            .operation_read_entity(Self::reference(), Number::reference())
            .operation_read_field(Self::reference(), Key::reference())
            .operation_read_field(Self::reference(), Primary::reference())
            .operation_read_field(Self::reference(), Secondary::reference())
            .operation_read_field(Self::reference(), ReadEnabled::reference())
            .operation_read_field(Self::reference(), Ceiling::reference())
            .operation_read_field(Self::reference(), Calculated::reference())
            .operation_write(Self::reference(), Key::reference())
            .operation_write(Self::reference(), Primary::reference())
            .operation_write(Self::reference(), Secondary::reference())
            .operation_write(Self::reference(), ReadEnabled::reference())
            .operation_write(Self::reference(), Ceiling::reference())
            .operation_write(Self::reference(), Calculated::reference())
            .operation_create(Self::reference(), Number::reference())
    }
}
impl ApplicationActionDelivery<NumericSchema, Binding> for Completion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "numeric-observed-clone.completion.v1";
}

pub struct UndoClone;
impl ApplicationCorrectionBinding<NumericSchema> for UndoClone {
    const IDENTITY: &'static str = "numeric-observed-clone.undo.v1";
    type Source = Binding;
    type InverseIntent = Retire;
    type Disclosure = query::ReadBinding;

    fn source_definition() -> ApplicationOperationDefinitionBuilder<
        NumericSchema,
        CloneOperation,
        CloneNumber,
        true,
        false,
    > {
        CloneOperation::reference()
            .definition()
            .no_external_effect()
    }
    fn disclosure_parameters(
        _: &CloneNumber,
    ) -> Result<ApplicationQueryParameterSet<query::ReadQuery>, ApplicationCorrectionDerivationDenial>
    {
        Ok(query::parameters())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<NumericSchema> {
        DeclaredCorrectionMechanism::recorded_create_inverse_binding::<Self>()
    }
    fn authored_identity() -> Option<ApplicationAuthoredCorrection<NumericSchema>> {
        Some(ApplicationAuthoredCorrection::retire_created::<
            Self,
            Family,
            Created,
            Retired,
        >())
    }
    fn source_output_fields() -> Vec<ApplicationCorrectionOutputField<NumericSchema>> {
        vec![ApplicationCorrectionOutput::<Binding, Created>::field(
            Key::reference(),
        )]
    }
    fn derive_correction(
        original: &CloneNumber,
        material: ApplicationCorrectionSourceMaterial<'_, NumericSchema>,
    ) -> Result<Retire, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning;
        let ApplicationCorrectionSourceMaterial::CreatedEntity(fields) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let target = StringApplicationValueBinding::decode(
            fields
                .field(&ApplicationCorrectionOutput::<Binding, Created>::field(
                    Key::reference(),
                ))
                .map_err(|_| UnsupportedSourceMeaning)?,
        )
        .map_err(|_| UnsupportedSourceMeaning)?;
        Ok(Retire {
            key: original.key.clone(),
            target,
            wrong_restore_key: false,
            collateral: None,
        })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("undo-observed-clone:{key}"))
    }
    fn visit<V: ApplicationCorrectionVisitor<NumericSchema>>(
        dispatch: ApplicationCorrectionDispatch<NumericSchema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
