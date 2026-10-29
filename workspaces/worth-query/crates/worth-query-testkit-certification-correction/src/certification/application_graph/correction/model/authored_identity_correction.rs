//! Both lifecycle directions bind the same family and its authored logical key.
use super::{authored_identity_model::*, query, schema::*};
use worth_query_host::facade::declaration::{
    application_aftermath::*, application_query::*, application_schema::*,
};

pub struct Family;
impl ApplicationAuthoredCorrectionFamily<NumericSchema> for Family {
    const IDENTITY: &'static str = "numeric.authored-number.v1";
    type Entity = Number;
    fn logical_key() -> ApplicationAuthoredLogicalKey<NumericSchema, Number> {
        ApplicationAuthoredLogicalKey::from_field(Key::reference())
    }
}
pub struct UndoCreate;
pub struct UndoRetire;
impl ApplicationCorrectionBinding<NumericSchema> for UndoCreate {
    const IDENTITY: &'static str = "authored-identity.undo-create.v1";
    type Source = CreateBinding;
    type InverseIntent = Retire;
    type Disclosure = query::ReadBinding;
    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<NumericSchema, CreateOperation, Create, true, false>
    {
        CreateOperation::reference()
            .definition()
            .no_external_effect()
    }
    fn disclosure_parameters(
        _: &Create,
    ) -> Result<ApplicationQueryParameterSet<query::ReadQuery>, ApplicationCorrectionDerivationDenial>
    {
        Ok(query::parameters())
    }
    fn authored_identity() -> Option<ApplicationAuthoredCorrection<NumericSchema>> {
        Some(ApplicationAuthoredCorrection::retire_created::<
            Self,
            Family,
            Created,
            Retired,
        >())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<NumericSchema> {
        DeclaredCorrectionMechanism::recorded_create_inverse_binding::<Self>()
    }
    fn source_output_fields() -> Vec<ApplicationCorrectionOutputField<NumericSchema>> {
        vec![ApplicationCorrectionOutput::<CreateBinding, Created>::field(Key::reference())]
    }
    fn derive_correction(
        original: &Create,
        material: ApplicationCorrectionSourceMaterial<'_, NumericSchema>,
    ) -> Result<Retire, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning;
        let ApplicationCorrectionSourceMaterial::CreatedEntity(fields) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let key = StringApplicationValueBinding::decode(
            fields
                .field(
                    &ApplicationCorrectionOutput::<CreateBinding, Created>::field(Key::reference()),
                )
                .map_err(|_| UnsupportedSourceMeaning)?,
        )
        .map_err(|_| UnsupportedSourceMeaning)?;
        Ok(Retire {
            key: original.key.clone(),
            target: original.inverse_target.clone().unwrap_or(key),
            wrong_restore_key: false,
        })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("undo-authored-create:{key}"))
    }
    fn visit<V: ApplicationCorrectionVisitor<NumericSchema>>(
        dispatch: ApplicationCorrectionDispatch<NumericSchema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
impl ApplicationCorrectionBinding<NumericSchema> for UndoRetire {
    const IDENTITY: &'static str = "authored-identity.undo-retire.v1";
    type Source = RetireBinding;
    type InverseIntent = Restore;
    type Disclosure = query::ReadBinding;
    fn source_definition() -> ApplicationOperationDefinitionBuilder<
        NumericSchema,
        RetireAuthoredEntity,
        Retire,
        true,
        false,
    > {
        RetireAuthoredEntity::reference()
            .definition()
            .no_external_effect()
    }
    fn disclosure_parameters(
        _: &Retire,
    ) -> Result<ApplicationQueryParameterSet<query::ReadQuery>, ApplicationCorrectionDerivationDenial>
    {
        Ok(query::parameters())
    }
    fn authored_identity() -> Option<ApplicationAuthoredCorrection<NumericSchema>> {
        Some(ApplicationAuthoredCorrection::restore_retired::<
            Self,
            Family,
            Retired,
            Created,
        >())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<NumericSchema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::new(
                [
                    DeclaredPreImageLocus::from_field(Key::reference()),
                    DeclaredPreImageLocus::from_field(Primary::reference()),
                    DeclaredPreImageLocus::from_field(Secondary::reference()),
                    DeclaredPreImageLocus::from_field(ReadEnabled::reference()),
                    DeclaredPreImageLocus::from_field(Ceiling::reference()),
                    DeclaredPreImageLocus::from_field(Calculated::reference()),
                ],
                usize::MAX,
            )
            .unwrap(),
        )
    }
    fn derive_correction(
        original: &Retire,
        material: ApplicationCorrectionSourceMaterial<'_, NumericSchema>,
    ) -> Result<Restore, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::{
            InvalidPreImage, MissingPreImage, UnsupportedSourceMeaning,
        };
        let ApplicationCorrectionSourceMaterial::RecordedInverse(preimage) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        macro_rules! field {
            ($Field:ty, $Codec:ty) => {
                <$Codec>::decode(
                    preimage
                        .field(&DeclaredPreImageLocus::from_field(<$Field>::reference()))
                        .ok_or(MissingPreImage)?,
                )
                .map_err(|_| InvalidPreImage)?
            };
        }
        let mut name = field!(Key, StringApplicationValueBinding);
        // An intentionally defective inverse supplies the public rejection control.
        if original.wrong_restore_key {
            name.push_str("-wrong");
        }
        Ok(Restore {
            key: original.key.clone(),
            name,
            primary: field!(Primary, U64ApplicationValueBinding),
            secondary: field!(Secondary, U64ApplicationValueBinding),
            read_enabled: field!(ReadEnabled, BoolApplicationValueBinding),
            ceiling: field!(Ceiling, U64ApplicationValueBinding),
            calculated: field!(Calculated, U64ApplicationValueBinding),
        })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("undo-authored-retire:{key}"))
    }
    fn visit<V: ApplicationCorrectionVisitor<NumericSchema>>(
        dispatch: ApplicationCorrectionDispatch<NumericSchema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
