//! Retained native member values and relation endpoints derive the complete inverse.
use super::*;
pub struct UndoCreate;
pub struct UndoRetire;
macro_rules! correction {
    ($correction:ident, $source:ty, $operation:ty, $input:ty, $inverse:ty, $id:literal) => {
        const IDENTITY: &'static str = $id;
        type Source = $source;
        type InverseIntent = $inverse;
        type Disclosure = graph_read::source::Binding;
        fn source_definition(
        ) -> ApplicationOperationDefinitionBuilder<Schema, $operation, $input, true, false> {
            <$operation>::reference().definition().no_external_effect()
        }
        fn disclosure_parameters(
            _: &$input,
        ) -> Result<
            ApplicationQueryParameterSet<graph_read::source::Query>,
            ApplicationCorrectionDerivationDenial,
        > {
            Ok(ApplicationQueryParameterSet::new())
        }
        fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
            Ok(format!("undo:{key}"))
        }
        fn visit<V: ApplicationCorrectionVisitor<Schema>>(
            dispatch: ApplicationCorrectionDispatch<Schema, Self, V>,
        ) -> V::Output {
            dispatch.conventional()
        }
    };
}
impl ApplicationCorrectionBinding<Schema> for UndoCreate {
    correction!(
        UndoCreate,
        CreateBinding,
        CreateGraph,
        Create,
        Retire,
        "source-subgraph.undo-roster-create.v1"
    );
    fn recover_source_input(
        bytes: &[u8],
        expected: worth_query_decl::facade::application_schema::CanonicalDigestId,
        bounds: worth_query_decl::facade::application_schema::ValueDecodeBounds,
    ) -> Result<
        worth_query_decl::facade::application_operation::ApplicationEncodedInput<
            <Self::Source as worth_query_decl::facade::application_operation::ApplicationMutationBinding<Schema>>::InputBinding,
        >,
        worth_query_decl::facade::application_operation::ApplicationRetainedInputDenial,
    >{
        worth_query_decl::facade::application_operation::ApplicationEncodedInput::decode_retained_with_seed(
            bytes, expected, bounds,
            worth_query_decl::facade::application_schema::CanonicalValueSeed::<<Self::Source as worth_query_decl::facade::application_operation::ApplicationMutationBinding<Schema>>::Input>::new(bounds),
        )
    }

    fn recover_inverse_input(
        bytes: &[u8],
        expected: worth_query_decl::facade::application_schema::CanonicalDigestId,
        bounds: worth_query_decl::facade::application_schema::ValueDecodeBounds,
    ) -> Result<
        worth_query_decl::facade::application_operation::ApplicationEncodedInput<
            <worth_query_decl::facade::application_aftermath::ApplicationInverseBinding<
                Schema,
                Self,
            > as worth_query_decl::facade::application_operation::ApplicationMutationBinding<
                Schema,
            >>::InputBinding,
        >,
        worth_query_decl::facade::application_operation::ApplicationRetainedInputDenial,
    > {
        worth_query_decl::facade::application_operation::ApplicationEncodedInput::decode_retained_with_seed(
            bytes, expected, bounds,
            worth_query_decl::facade::application_schema::CanonicalValueSeed::<<worth_query_decl::facade::application_aftermath::ApplicationInverseBinding<Schema, Self> as worth_query_decl::facade::application_operation::ApplicationMutationBinding<Schema>>::Input>::new(bounds),
        )
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::recorded_create_inverse_binding::<Self>()
    }
    fn authored_identity() -> Option<ApplicationAuthoredCorrection<Schema>> {
        Some(
            ApplicationAuthoredCorrection::retire_created::<Self, AnchorFamily, Created, Retired>()
                .owned_members::<Self, MemberFamily, CreatedMembers, RetiredMembers, _, _>(
                    OwnedBy::reference(),
                ),
        )
    }
    fn derive_correction(
        original: &Create,
        _: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<Retire, ApplicationCorrectionDerivationDenial> {
        Ok(Retire {
            key: original.0.key.clone(),
            members: original
                .0
                .members
                .iter()
                .map(|row| row.key.clone())
                .collect(),
            replacement: None,
        })
    }
}
impl ApplicationCorrectionBinding<Schema> for UndoRetire {
    correction!(
        UndoRetire,
        RetireBinding,
        RetireGraph,
        Retire,
        Restore,
        "source-subgraph.undo-roster-retire.v1"
    );
    fn recover_source_input(
        bytes: &[u8],
        expected: worth_query_decl::facade::application_schema::CanonicalDigestId,
        bounds: worth_query_decl::facade::application_schema::ValueDecodeBounds,
    ) -> Result<
        worth_query_decl::facade::application_operation::ApplicationEncodedInput<
            <Self::Source as worth_query_decl::facade::application_operation::ApplicationMutationBinding<Schema>>::InputBinding,
        >,
        worth_query_decl::facade::application_operation::ApplicationRetainedInputDenial,
    >{
        worth_query_decl::facade::application_operation::ApplicationEncodedInput::decode_retained_with_seed(
            bytes, expected, bounds,
            worth_query_decl::facade::application_schema::CanonicalValueSeed::<<Self::Source as worth_query_decl::facade::application_operation::ApplicationMutationBinding<Schema>>::Input>::new(bounds),
        )
    }

    fn recover_inverse_input(
        bytes: &[u8],
        expected: worth_query_decl::facade::application_schema::CanonicalDigestId,
        bounds: worth_query_decl::facade::application_schema::ValueDecodeBounds,
    ) -> Result<
        worth_query_decl::facade::application_operation::ApplicationEncodedInput<
            <worth_query_decl::facade::application_aftermath::ApplicationInverseBinding<
                Schema,
                Self,
            > as worth_query_decl::facade::application_operation::ApplicationMutationBinding<
                Schema,
            >>::InputBinding,
        >,
        worth_query_decl::facade::application_operation::ApplicationRetainedInputDenial,
    > {
        worth_query_decl::facade::application_operation::ApplicationEncodedInput::decode_retained_with_seed(
            bytes, expected, bounds,
            worth_query_decl::facade::application_schema::CanonicalValueSeed::<<worth_query_decl::facade::application_aftermath::ApplicationInverseBinding<Schema, Self> as worth_query_decl::facade::application_operation::ApplicationMutationBinding<Schema>>::Input>::new(bounds),
        )
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::new(
                [
                    DeclaredPreImageLocus::from_field(SourceKey::at("SourceKey")),
                    DeclaredPreImageLocus::from_field(SourceValue::at("SourceValue")),
                    DeclaredPreImageLocus::from_field(MemberKey::at("MemberKey")),
                    DeclaredPreImageLocus::from_field(Value::at("MemberValue")),
                ],
                1024,
            )
            .with_relations([
                DeclaredPreImageRelationLocus::from_relation(OwnedBy::reference()),
                DeclaredPreImageRelationLocus::from_relation(Next::reference()),
                DeclaredPreImageRelationLocus::from_relation(Operand::reference()),
                DeclaredPreImageRelationLocus::from_relation(Head::reference()),
            ]),
        )
    }
    fn authored_identity() -> Option<ApplicationAuthoredCorrection<Schema>> {
        Some(ApplicationAuthoredCorrection::restore_retired::<Self, AnchorFamily, Retired, Restored>()
            .co_retire_created::<Self, AnchorFamily, ReplacementCreated, ReplacementRetired>()
            .owned_members::<Self, MemberFamily, RetiredMembers, RestoredMembers, _, _>(OwnedBy::reference())
            .owned_members::<Self, MemberFamily, ReplacementMembers, RemovedMembers, _, _>(OwnedBy::reference()))
    }
    fn derive_correction(
        original: &Retire,
        material: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<Restore, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::*;
        let ApplicationCorrectionSourceMaterial::RecordedInverse(prior) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let key = StringApplicationValueBinding::decode(
            prior
                .field_for_output(
                    &Retired::DESCRIPTOR,
                    &DeclaredPreImageLocus::from_field(SourceKey::at("SourceKey")),
                )
                .ok_or(MissingPreImage)?,
        )
        .map_err(|_| InvalidPreImage)?;
        let value = U64ApplicationValueBinding::decode(
            prior
                .field_for_output(
                    &Retired::DESCRIPTOR,
                    &DeclaredPreImageLocus::from_field(SourceValue::at("SourceValue")),
                )
                .ok_or(MissingPreImage)?,
        )
        .map_err(|_| InvalidPreImage)?;
        let mut members = Vec::new();
        let mut ids = Vec::new();
        let mut index = 0;
        while let Some(suffix) = prior.family_member_suffix(&RetiredMembers::DESCRIPTOR, index) {
            let key = StringApplicationValueBinding::decode(
                prior
                    .field_for_family_member(
                        &RetiredMembers::DESCRIPTOR,
                        suffix,
                        &DeclaredPreImageLocus::from_field(MemberKey::at("MemberKey")),
                    )
                    .ok_or(MissingPreImage)?,
            )
            .map_err(|_| InvalidPreImage)?;
            let value = U64ApplicationValueBinding::decode(
                prior
                    .field_for_family_member(
                        &RetiredMembers::DESCRIPTOR,
                        suffix,
                        &DeclaredPreImageLocus::from_field(Value::at("MemberValue")),
                    )
                    .ok_or(MissingPreImage)?,
            )
            .map_err(|_| InvalidPreImage)?;
            ids.push((
                prior
                    .family_member_entity(&RetiredMembers::DESCRIPTOR, suffix)
                    .ok_or(MissingPreImage)?,
                key.clone(),
            ));
            members.push(Row {
                key,
                value,
                next: None,
                operand: None,
            });
            index += 1;
        }
        let links = DeclaredPreImageRelationLocus::from_relation(Next::reference());
        for index in 0..prior.relation_count(&links).ok_or(MissingPreImage)? {
            let relation = prior.relation_at(&links, index).ok_or(MissingPreImage)?;
            if let Some((from, to)) = relation.before() {
                let from = &ids
                    .iter()
                    .find(|(id, _)| *id == from)
                    .ok_or(InvalidPreImage)?
                    .1;
                let to = &ids
                    .iter()
                    .find(|(id, _)| *id == to)
                    .ok_or(InvalidPreImage)?
                    .1;
                members
                    .iter_mut()
                    .find(|row| &row.key == from)
                    .ok_or(InvalidPreImage)?
                    .next = Some(to.clone());
            }
        }
        let links = DeclaredPreImageRelationLocus::from_relation(Operand::reference());
        for index in 0..prior.relation_count(&links).ok_or(MissingPreImage)? {
            let relation = prior.relation_at(&links, index).ok_or(MissingPreImage)?;
            if let Some((from, to)) = relation.before() {
                let from = &ids
                    .iter()
                    .find(|(id, _)| *id == from)
                    .ok_or(InvalidPreImage)?
                    .1;
                let to = &ids
                    .iter()
                    .find(|(id, _)| *id == to)
                    .ok_or(InvalidPreImage)?
                    .1;
                members
                    .iter_mut()
                    .find(|row| &row.key == from)
                    .ok_or(InvalidPreImage)?
                    .operand = Some(to.clone());
            }
        }
        Ok(Restore {
            image: Image {
                key,
                value,
                members,
            },
            remove: original.replacement.as_ref().map(|image| Retire {
                key: image.key.clone(),
                members: image.members.iter().map(|row| row.key.clone()).collect(),
                replacement: None,
            }),
        })
    }
}
