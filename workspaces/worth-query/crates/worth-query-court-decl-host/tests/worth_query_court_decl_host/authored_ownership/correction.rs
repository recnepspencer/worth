//! Authored correction supplies the distinguished source anchor in both directions.
use super::{lifecycle::*, *};
pub struct Family;
impl<S: Scenario> ApplicationAuthoredCorrectionFamily<S> for Family {
    const IDENTITY: &'static str = "ownership.authored.family.v1";
    type Entity = Node;
    fn logical_key() -> ApplicationAuthoredLogicalKey<S, Node> {
        ApplicationAuthoredLogicalKey::from_field(Id::reference())
    }
}
pub struct UndoCreate<S>(PhantomData<fn() -> S>);
pub struct UndoRetire<S>(PhantomData<fn() -> S>);
impl<S: Scenario> ApplicationCorrectionBinding<S> for UndoCreate<S> {
    const IDENTITY: &'static str = "ownership.undo-create.v1";
    type Source = CreateBinding<S>;
    type InverseIntent = Retire;
    type Disclosure = query::Binding<S>;

    fn recover_source_input(
        bytes: &[u8],
        expected: worth_query_decl::facade::application_schema::CanonicalDigestId,
        bounds: worth_query_decl::facade::application_schema::ValueDecodeBounds,
    ) -> Result<
        worth_query_decl::facade::application_operation::ApplicationEncodedInput<
            <Self::Source as worth_query_decl::facade::application_operation::ApplicationMutationBinding<S>>::InputBinding,
        >,
        worth_query_decl::facade::application_operation::ApplicationRetainedInputDenial,
    >{
        worth_query_decl::facade::application_operation::ApplicationEncodedInput::decode_retained_with_seed(
            bytes, expected, bounds,
            worth_query_decl::facade::application_schema::CanonicalValueSeed::<<Self::Source as worth_query_decl::facade::application_operation::ApplicationMutationBinding<S>>::Input>::new(bounds),
        )
    }
    fn source_definition() -> ApplicationOperationDefinitionBuilder<S, CreateOp, Create, true, false>
    {
        CreateOp::reference().definition().no_external_effect()
    }
    fn disclosure_parameters(
        _: &Create,
    ) -> Result<ApplicationQueryParameterSet<query::Query>, ApplicationCorrectionDerivationDenial>
    {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn authored_identity() -> Option<ApplicationAuthoredCorrection<S>> {
        Some(ApplicationAuthoredCorrection::retire_created::<
            Self,
            Family,
            Created<S>,
            Retired<S>,
        >())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<S> {
        DeclaredCorrectionMechanism::recorded_create_inverse_binding::<Self>()
    }
    fn source_output_fields() -> Vec<ApplicationCorrectionOutputField<S>> {
        vec![ApplicationCorrectionOutput::<CreateBinding<S>, Created<S>>::field(Id::reference())]
    }
    fn derive_correction(
        original: &Create,
        material: ApplicationCorrectionSourceMaterial<'_, S>,
    ) -> Result<Retire, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning;
        let ApplicationCorrectionSourceMaterial::CreatedEntity(fields) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let id = U64ApplicationValueBinding::decode(
            fields
                .field(
                    &ApplicationCorrectionOutput::<CreateBinding<S>, Created<S>>::field(
                        Id::reference(),
                    ),
                )
                .map_err(|_| UnsupportedSourceMeaning)?,
        )
        .map_err(|_| UnsupportedSourceMeaning)?;
        let mut descendants = Vec::new();
        let mut source = id;
        for _ in 0..original.depth {
            descendants.extend([source * 10 + 1, source * 10 + 2]);
            source = source * 10 + 2;
        }
        Ok(Retire {
            scope: original.scope,
            id,
            descendants,
        })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("retire:{key}"))
    }
    fn visit<V: ApplicationCorrectionVisitor<S>>(
        dispatch: ApplicationCorrectionDispatch<S, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
impl<S: Scenario> ApplicationCorrectionBinding<S> for UndoRetire<S> {
    const IDENTITY: &'static str = "ownership.undo-retire.v1";
    type Source = RetireBinding<S>;
    type InverseIntent = Restore;
    type Disclosure = query::Binding<S>;

    fn recover_source_input(
        bytes: &[u8],
        expected: worth_query_decl::facade::application_schema::CanonicalDigestId,
        bounds: worth_query_decl::facade::application_schema::ValueDecodeBounds,
    ) -> Result<
        worth_query_decl::facade::application_operation::ApplicationEncodedInput<
            <Self::Source as worth_query_decl::facade::application_operation::ApplicationMutationBinding<S>>::InputBinding,
        >,
        worth_query_decl::facade::application_operation::ApplicationRetainedInputDenial,
    >{
        worth_query_decl::facade::application_operation::ApplicationEncodedInput::decode_retained_with_seed(
            bytes, expected, bounds,
            worth_query_decl::facade::application_schema::CanonicalValueSeed::<<Self::Source as worth_query_decl::facade::application_operation::ApplicationMutationBinding<S>>::Input>::new(bounds),
        )
    }
    fn source_definition() -> ApplicationOperationDefinitionBuilder<S, RetireOp, Retire, true, false>
    {
        RetireOp::reference().definition().no_external_effect()
    }
    fn disclosure_parameters(
        _: &Retire,
    ) -> Result<ApplicationQueryParameterSet<query::Query>, ApplicationCorrectionDerivationDenial>
    {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn authored_identity() -> Option<ApplicationAuthoredCorrection<S>> {
        Some(ApplicationAuthoredCorrection::restore_retired::<
            Self,
            Family,
            Retired<S>,
            Created<S>,
        >())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<S> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::new(
                [
                    DeclaredPreImageLocus::from_field(Id::reference::<S>()),
                    DeclaredPreImageLocus::from_field(Value::reference::<S>()),
                ],
                18,
            )
            .unwrap(),
        )
    }
    fn derive_correction(
        original: &Retire,
        material: ApplicationCorrectionSourceMaterial<'_, S>,
    ) -> Result<Restore, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::{
            InvalidPreImage, MissingPreImage, UnsupportedSourceMeaning,
        };
        let ApplicationCorrectionSourceMaterial::RecordedInverse(fields) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let id = U64ApplicationValueBinding::decode(
            fields
                .field(&DeclaredPreImageLocus::from_field(Id::reference::<S>()))
                .ok_or(MissingPreImage)?,
        )
        .map_err(|_| InvalidPreImage)?;
        let value = U64ApplicationValueBinding::decode(
            fields
                .field(&DeclaredPreImageLocus::from_field(Value::reference::<S>()))
                .ok_or(MissingPreImage)?,
        )
        .map_err(|_| InvalidPreImage)?;
        Ok(Restore {
            scope: original.scope,
            id,
            value,
        })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("restore:{key}"))
    }
    fn visit<V: ApplicationCorrectionVisitor<S>>(
        dispatch: ApplicationCorrectionDispatch<S, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
