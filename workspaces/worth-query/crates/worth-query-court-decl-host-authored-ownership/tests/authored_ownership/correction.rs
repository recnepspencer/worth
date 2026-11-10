//! Authored correction supplies the distinguished source anchor in both directions.
use super::{co_roles::*, lifecycle::*, *};
pub struct Family;
impl<S: Scenario> ApplicationAuthoredCorrectionFamily<S> for Family {
    const IDENTITY: &'static str = "ownership.authored.family.v1";
    type Entity = Node<S>;
    fn logical_key() -> ApplicationAuthoredLogicalKey<S, Node<S>> {
        ApplicationAuthoredLogicalKey::from_field(schema::id::<S>())
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

    fn recover_inverse_input(
        bytes: &[u8],
        expected: worth_query_decl::facade::application_schema::CanonicalDigestId,
        bounds: worth_query_decl::facade::application_schema::ValueDecodeBounds,
    ) -> Result<
        worth_query_decl::facade::application_operation::ApplicationEncodedInput<
            <worth_query_decl::facade::application_aftermath::ApplicationInverseBinding<S, Self> as worth_query_decl::facade::application_operation::ApplicationMutationBinding<S>>::InputBinding,
        >,
        worth_query_decl::facade::application_operation::ApplicationRetainedInputDenial,
    >{
        worth_query_decl::facade::application_operation::ApplicationEncodedInput::decode_retained_with_seed(
            bytes, expected, bounds,
            worth_query_decl::facade::application_schema::CanonicalValueSeed::<<worth_query_decl::facade::application_aftermath::ApplicationInverseBinding<S, Self> as worth_query_decl::facade::application_operation::ApplicationMutationBinding<S>>::Input>::new(bounds),
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
        let identity =
            ApplicationAuthoredCorrection::retire_created::<Self, Family, Created<S>, Retired<S>>();
        Some(if <S::Heterogeneous as heterogeneous::Profile>::PRESENT {
            <S::Heterogeneous as heterogeneous::Profile>::created_identity(identity)
        } else if matches!(S::MODE, 5 | 19 | 20) {
            identity
                .co_retire_created::<Self, Family, CoCreated<S, 0>, CoRetired<S, 0>>()
                .co_retire_created::<Self, Family, CoCreated<S, 1>, CoRetired<S, 1>>()
        } else {
            identity
        })
    }
    fn visit_coupled_outputs<V: ApplicationCoupledOutputVisitor<S, Self::Source>>(visitor: &mut V) {
        if S::MODE == 60 {
            visitor.required::<program::Start>();
        }
    }
    fn mechanism() -> DeclaredCorrectionMechanism<S> {
        DeclaredCorrectionMechanism::recorded_create_inverse_binding::<Self>()
    }
    fn source_output_fields() -> Vec<ApplicationCorrectionOutputField<S>> {
        vec![ApplicationCorrectionOutput::<CreateBinding<S>, Created<S>>::field(schema::id::<S>())]
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
                        schema::id::<S>(),
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
        for &source in &original.co_anchors {
            for _ in 0..original.depth {
                descendants.extend([source * 10 + 1, source * 10 + 2]);
            }
        }
        Ok(Retire {
            relation_fault: 0,
            corrupt_restoration: false,
            co_anchors: original.co_anchors.clone(),
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

    fn recover_inverse_input(
        bytes: &[u8],
        expected: worth_query_decl::facade::application_schema::CanonicalDigestId,
        bounds: worth_query_decl::facade::application_schema::ValueDecodeBounds,
    ) -> Result<
        worth_query_decl::facade::application_operation::ApplicationEncodedInput<
            <worth_query_decl::facade::application_aftermath::ApplicationInverseBinding<S, Self> as worth_query_decl::facade::application_operation::ApplicationMutationBinding<S>>::InputBinding,
        >,
        worth_query_decl::facade::application_operation::ApplicationRetainedInputDenial,
    >{
        worth_query_decl::facade::application_operation::ApplicationEncodedInput::decode_retained_with_seed(
            bytes, expected, bounds,
            worth_query_decl::facade::application_schema::CanonicalValueSeed::<<worth_query_decl::facade::application_aftermath::ApplicationInverseBinding<S, Self> as worth_query_decl::facade::application_operation::ApplicationMutationBinding<S>>::Input>::new(bounds),
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
        let identity =
            ApplicationAuthoredCorrection::restore_retired::<Self, Family, Retired<S>, Created<S>>(
            );
        Some(if <S::Heterogeneous as heterogeneous::Profile>::PRESENT {
            <S::Heterogeneous as heterogeneous::Profile>::retired_identity(identity)
        } else if matches!(S::MODE, 5 | 19 | 20) {
            identity
                .co_restore_retired::<Self, Family, CoRetired<S, 0>, CoCreated<S, 0>>()
                .co_restore_retired::<Self, Family, CoRetired<S, 1>, CoCreated<S, 1>>()
        } else {
            identity
        })
    }
    fn visit_coupled_outputs<V: ApplicationCoupledOutputVisitor<S, Self::Source>>(visitor: &mut V) {
        if S::MODE == 24 {
            visitor.required::<coupled::multi::SecondStart>();
        } else if S::MODE != 11 {
            <Self::Source as ApplicationMutationBinding<S>>::visit_coupled_outputs(visitor);
        }
    }
    fn mechanism() -> DeclaredCorrectionMechanism<S> {
        let mut loci = vec![
            DeclaredPreImageLocus::from_field(schema::id::<S>()),
            DeclaredPreImageLocus::from_field(schema::value::<S>()),
        ];
        if heterogeneous::active::<S>() {
            loci.push(DeclaredPreImageLocus::from_field(
                heterogeneous::BodyKey::at("BodyKey"),
            ));
        }
        let demand = DeclaredPreImageDemand::new(
            loci,
            if heterogeneous::active::<S>() || matches!(S::MODE, 30 | 39 | 40) {
                4096
            } else if matches!(S::MODE, 5 | 19 | 20) {
                54
            } else {
                18
            },
        );
        let demand = if matches!(S::MODE, 30 | 39 | 40 | 41) {
            demand.with_relations([DeclaredPreImageRelationLocus::from_relation(
                PreviousStage::reference(),
            )])
        } else {
            demand
        };
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(demand)
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
                .field(&DeclaredPreImageLocus::from_field(schema::id::<S>()))
                .ok_or(MissingPreImage)?,
        )
        .map_err(|_| InvalidPreImage)?;
        let value = U64ApplicationValueBinding::decode(
            fields
                .field(&DeclaredPreImageLocus::from_field(schema::value::<S>()))
                .ok_or(MissingPreImage)?,
        )
        .map_err(|_| InvalidPreImage)?;
        let roles = [
            <CoRetired<S, 0> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
            <CoRetired<S, 1> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        ];
        let mut co_values = original
            .co_anchors
            .iter()
            .enumerate()
            .map(|(index, _)| {
                let value = fields
                    .field_for_output(
                        &roles[index],
                        &DeclaredPreImageLocus::from_field(schema::value::<S>()),
                    )
                    .ok_or(MissingPreImage)?;
                U64ApplicationValueBinding::decode(value).map_err(|_| InvalidPreImage)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(key) = <S::Heterogeneous as heterogeneous::Profile>::retained_key::<S>(fields)?
        {
            co_values.push(key);
        }
        if original.corrupt_restoration {
            co_values[0] += 1;
        }
        let (retired_entity, relation_material) =
            relation_splice::derive::<S>(fields, original.relation_fault)?;
        Ok(Restore {
            retired_entity,
            relation_material,
            co_values,
            co_anchors: original.co_anchors.clone(),
            scope: original.scope,
            id,
            value,
        })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("restore:{key}"))
    }
    fn reapplication_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("reapply:{key}"))
    }
    fn visit_reapplication<V: ApplicationReapplicationVisitor<S>>(
        dispatch: ApplicationReapplicationDispatch<S, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
    fn visit<V: ApplicationCorrectionVisitor<S>>(
        dispatch: ApplicationCorrectionDispatch<S, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
