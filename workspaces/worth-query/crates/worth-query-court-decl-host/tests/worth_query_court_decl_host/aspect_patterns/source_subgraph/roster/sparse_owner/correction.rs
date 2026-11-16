//! The inverse is the same revision: changed owner fields and moved members swap back.
use super::*;
use local_member::Returned;
pub struct Revision;
impl ApplicationCorrectionBinding<Schema> for Revision {
    const IDENTITY: &'static str = "source-subgraph.owner-revision.v1";
    type Source = ReviseBinding;
    type InverseIntent = Revise;
    type Disclosure = graph_read::source::Binding;
    fn recover_source_input(
        bytes: &[u8],
        expected: CanonicalDigestId,
        bounds: ValueDecodeBounds,
    ) -> Result<ApplicationEncodedInput<ReviseInput>, ApplicationRetainedInputDenial> {
        ApplicationEncodedInput::decode_retained_with_seed(
            bytes,
            expected,
            bounds,
            CanonicalValueSeed::<Revise>::new(bounds),
        )
    }
    fn recover_inverse_input(
        bytes: &[u8],
        expected: CanonicalDigestId,
        bounds: ValueDecodeBounds,
    ) -> Result<ApplicationEncodedInput<ReviseInput>, ApplicationRetainedInputDenial> {
        Self::recover_source_input(bytes, expected, bounds)
    }
    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<Schema, ReviseOwner, Revise, true, false> {
        ReviseOwner::reference().definition().no_external_effect()
    }
    fn disclosure_parameters(
        _: &Revise,
    ) -> Result<
        ApplicationQueryParameterSet<graph_read::source::Query>,
        ApplicationCorrectionDerivationDenial,
    > {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("revise:undo:{key}"))
    }
    fn reapplication_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("revise:reapply:{key}"))
    }
    fn visit_reapplication<V: ApplicationReapplicationVisitor<Schema>>(
        dispatch: ApplicationReapplicationDispatch<Schema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
    fn visit<V: ApplicationCorrectionVisitor<Schema>>(
        dispatch: ApplicationCorrectionDispatch<Schema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
    // Leaving members return and arriving members leave, under the one owner.
    fn authored_identity() -> Option<ApplicationAuthoredCorrection<Schema>> {
        Some(
            ApplicationAuthoredCorrection::live_owner_members::<
                Self,
                MemberFamily,
                LeavingMembers,
                ArrivingMembers,
                _,
                _,
            >(OwnedBy::reference())
            .and_live_owner_members::<Self, MemberFamily, ArrivingMembers, LeavingMembers, _, _>(
                OwnedBy::reference(),
            ),
        )
    }
    // The owner's fields are captured only where they changed; member fields
    // are captured completely on each member that leaves.
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::changed_fields_for_output(
                RevisedOwner::DESCRIPTOR,
                [
                    DeclaredPreImageLocus::from_field(SourceValue::at("SourceValue")),
                    DeclaredPreImageLocus::from_field(OwnerMark::at("OwnerMark")),
                    DeclaredPreImageLocus::from_field(MemberKey::at("MemberKey")),
                    DeclaredPreImageLocus::from_field(Value::at("MemberValue")),
                ],
                1024,
            )
            .with_relations([DeclaredPreImageRelationLocus::from_relation(
                OwnedBy::reference(),
            )]),
        )
    }
    fn derive_correction(
        original: &Revise,
        material: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<Revise, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::*;
        use ApplicationCorrectionFieldPreImage as State;
        let ApplicationCorrectionSourceMaterial::RecordedInverse(prior) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let owner = |locus| prior.field_state_for_output(&RevisedOwner::DESCRIPTOR, &locus);
        let decode = |value| U64ApplicationValueBinding::decode(value).map_err(|_| InvalidPreImage);
        let value = match owner(DeclaredPreImageLocus::from_field(SourceValue::at(
            "SourceValue",
        ))) {
            State::Uncaptured => None,
            State::Value(value) => Some(decode(value)?),
            _ => return Err(InvalidPreImage),
        };
        let mark = match owner(DeclaredPreImageLocus::from_field(OwnerMark::at(
            "OwnerMark",
        ))) {
            State::Uncaptured => MarkEdit::Unchanged,
            State::Value(mark) => MarkEdit::Set(decode(mark)?),
            State::AbsentField | State::AbsentAspect => MarkEdit::Clear,
        };
        let member = |suffix: &str, locus| {
            prior
                .field_for_family_member(&LeavingMembers::DESCRIPTOR, suffix, &locus)
                .ok_or(MissingPreImage)
        };
        let create = original
            .retire
            .iter()
            .map(|suffix| {
                Ok(Returned {
                    key: StringApplicationValueBinding::decode(member(
                        suffix,
                        DeclaredPreImageLocus::from_field(MemberKey::at("MemberKey")),
                    )?)
                    .map_err(|_| InvalidPreImage)?,
                    value: decode(member(
                        suffix,
                        DeclaredPreImageLocus::from_field(Value::at("MemberValue")),
                    )?)?,
                })
            })
            .collect::<Result<Vec<_>, ApplicationCorrectionDerivationDenial>>()?;
        Ok(Revise {
            owner: original.owner.clone(),
            value,
            mark,
            retire: original.create.iter().map(|row| row.key.clone()).collect(),
            create,
        })
    }
}
