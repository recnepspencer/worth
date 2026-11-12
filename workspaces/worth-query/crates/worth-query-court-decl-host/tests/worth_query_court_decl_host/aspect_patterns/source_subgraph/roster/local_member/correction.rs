//! The inverse returns every retained member and note of the original typed local intent.
use super::*;
pub struct UndoDelete;
impl ApplicationCorrectionBinding<Schema> for UndoDelete {
    const IDENTITY: &'static str = "source-subgraph.undo-member-delete.v1";
    type Source = DeleteBinding;
    type InverseIntent = Restore;
    type Disclosure = graph_read::source::Binding;
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
    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<Schema, DeleteMember, Delete, true, false> {
        DeleteMember::reference().definition().no_external_effect()
    }
    fn disclosure_parameters(
        _: &Delete,
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
    // Two families under the one owner that stays live; each names its own
    // owner relation, and either may move no member at all.
    fn authored_identity() -> Option<ApplicationAuthoredCorrection<Schema>> {
        Some(
            ApplicationAuthoredCorrection::live_owner_members::<
                Self,
                MemberFamily,
                Deleted,
                Restored,
                _,
                _,
            >(OwnedBy::reference())
            .and_live_owner_members::<Self, NoteFamily, DeletedNotes, RestoredNotes, _, _>(
                NotedBy::reference(),
            ),
        )
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::new(
                [
                    DeclaredPreImageLocus::from_field(SourceValue::at("SourceValue")),
                    DeclaredPreImageLocus::from_field(MemberKey::at("MemberKey")),
                    DeclaredPreImageLocus::from_field(Value::at("MemberValue")),
                    DeclaredPreImageLocus::from_field(NoteKey::at("NoteKey")),
                    DeclaredPreImageLocus::from_field(NoteValue::at("NoteValue")),
                    DeclaredPreImageLocus::from_field(Mark::at("Mark")),
                ],
                1024,
            )
            .with_relations([
                DeclaredPreImageRelationLocus::from_relation(OwnedBy::reference()),
                DeclaredPreImageRelationLocus::from_relation(Head::reference()),
                DeclaredPreImageRelationLocus::from_relation(Operand::reference()),
                DeclaredPreImageRelationLocus::from_relation(NotedBy::reference()),
            ]),
        )
    }
    fn derive_correction(
        original: &Delete,
        material: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<Restore, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::*;
        let ApplicationCorrectionSourceMaterial::RecordedInverse(prior) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        // The original intent orders the members; every value comes from the
        // retained pre-image of the exact member its family role names.
        let returned = |family: &ApplicationMutationOutputRoleFamilyDescriptor,
                        suffixes: &[String],
                        key: DeclaredPreImageLocus<Schema>,
                        value: DeclaredPreImageLocus<Schema>| {
            suffixes
                .iter()
                .map(|suffix| {
                    Ok(Returned {
                        key: StringApplicationValueBinding::decode(
                            prior
                                .field_for_family_member(family, suffix, &key)
                                .ok_or(MissingPreImage)?,
                        )
                        .map_err(|_| InvalidPreImage)?,
                        value: U64ApplicationValueBinding::decode(
                            prior
                                .field_for_family_member(family, suffix, &value)
                                .ok_or(MissingPreImage)?,
                        )
                        .map_err(|_| InvalidPreImage)?,
                    })
                })
                .collect::<Result<Vec<_>, ApplicationCorrectionDerivationDenial>>()
        };
        Ok(Restore {
            owner: original.owner.clone(),
            owner_value: prior
                .field(&DeclaredPreImageLocus::from_field(SourceValue::at(
                    "SourceValue",
                )))
                .map(U64ApplicationValueBinding::decode)
                .transpose()
                .map_err(|_| InvalidPreImage)?,
            members: returned(
                &Deleted::DESCRIPTOR,
                &original.members,
                DeclaredPreImageLocus::from_field(MemberKey::at("MemberKey")),
                DeclaredPreImageLocus::from_field(Value::at("MemberValue")),
            )?,
            operand: original.operand.clone(),
            notes: returned(
                &DeletedNotes::DESCRIPTOR,
                &original.notes,
                DeclaredPreImageLocus::from_field(NoteKey::at("NoteKey")),
                DeclaredPreImageLocus::from_field(NoteValue::at("NoteValue")),
            )?,
            // A note the source read without a mark retained none and returns bare.
            marks: original
                .notes
                .iter()
                .filter_map(|suffix| {
                    let mark = prior.field_for_family_member(
                        &DeletedNotes::DESCRIPTOR,
                        suffix,
                        &DeclaredPreImageLocus::from_field(Mark::at("Mark")),
                    )?;
                    Some(
                        U64ApplicationValueBinding::decode(mark)
                            .map(|value| Returned {
                                key: suffix.clone(),
                                value,
                            })
                            .map_err(|_| InvalidPreImage),
                    )
                })
                .collect::<Result<Vec<_>, ApplicationCorrectionDerivationDenial>>()?,
            // A member that stays retains only the field the source changed.
            revalue: original
                .revalue
                .as_ref()
                .map(|row| {
                    Ok(Returned {
                        key: row.key.clone(),
                        value: U64ApplicationValueBinding::decode(
                            prior
                                .field_for_family_member(
                                    &Kept::DESCRIPTOR,
                                    &row.key,
                                    &DeclaredPreImageLocus::from_field(Value::at("MemberValue")),
                                )
                                .ok_or(MissingPreImage)?,
                        )
                        .map_err(|_| InvalidPreImage)?,
                    })
                })
                .transpose()?,
        })
    }
}
