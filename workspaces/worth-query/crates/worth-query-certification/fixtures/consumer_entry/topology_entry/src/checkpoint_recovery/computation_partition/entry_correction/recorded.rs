//! Recorded XOR inversion retains the prior value and admits its exact after-image.
use super::*;
use worth_query_decl::facade::{application_aftermath::*, application_query::*};

pub(in super::super) struct RecordedEntryCorrection<Schema>(PhantomData<fn() -> Schema>);

/// The enclosing composition dispatches correction meaning without exporting its binding.
pub(in crate::checkpoint_recovery) fn visit<
    Visitor: ApplicationCorrectionVisitor<CheckpointSchema>,
>(
    source_binding: std::any::TypeId,
    visitor: Visitor,
) -> Result<Visitor::Output, Visitor> {
    if source_binding == std::any::TypeId::of::<EntryCorrectionBinding<CheckpointSchema>>() {
        Ok(dispatch_application_correction::<
            CheckpointSchema,
            RecordedEntryCorrection<CheckpointSchema>,
            Visitor,
        >(visitor))
    } else {
        Err(visitor)
    }
}

impl<Schema: TopologySchemaBinding> ApplicationCorrectionBinding<Schema>
    for RecordedEntryCorrection<Schema>
{
    const IDENTITY: &'static str = "worth.query.certification.recorded-entry-correction.v1";
    type Source = EntryCorrectionBinding<Schema>;
    type InverseIntent = EntryCorrection;
    type Disclosure = PlanarReadBinding<Schema>;

    fn disclosure_parameters(
        _: &EntryCorrection,
    ) -> Result<ApplicationQueryParameterSet<PlanarQuery>, ApplicationCorrectionDerivationDenial>
    {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn source_definition() -> ApplicationOperationDefinitionBuilder<
        Schema,
        CorrectEntryValue,
        EntryCorrection,
        true,
        false,
    > {
        CorrectEntryValue::reference()
            .definition()
            .no_external_effect()
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::new(
                [DeclaredPreImageLocus::from_field(
                    EntryValueBits::reference(),
                )],
                U64ApplicationValueBinding::encode(&u64::MAX)
                    .unwrap()
                    .semantic_byte_width(),
            )
            .unwrap(),
        )
    }
    fn derive_correction(
        original: &EntryCorrection,
        material: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<EntryCorrection, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::*;
        let ApplicationCorrectionSourceMaterial::RecordedInverse(preimage) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let prior = U64ApplicationValueBinding::decode(
            preimage
                .field(&DeclaredPreImageLocus::from_field(
                    EntryValueBits::reference(),
                ))
                .ok_or(MissingPreImage)?,
        )
        .map_err(|_| InvalidPreImage)?;
        if original.expected.is_some_and(|expected| expected != prior) {
            return Err(InvalidOriginalInput);
        }
        let expected = prior ^ original.mask;
        Ok(EntryCorrection {
            scope_key: original.scope_key.clone(),
            entry: original.entry,
            mask: expected ^ prior,
            expected: Some(expected),
        })
    }
    fn inverse_key(key: &str) -> Result<u64, ApplicationValueEncodeDenial> {
        key.parse().map_err(|_| ApplicationValueEncodeDenial::CodecRejected {
            binding_identity: worth_query_decl::facade::portable_identity::WorthQueryPortableTypeIdentity::declared(Self::IDENTITY),
            reason: "entry correction keys are unsigned decimal integers",
        })
    }
    fn visit<Visitor: ApplicationCorrectionVisitor<Schema>>(
        dispatch: ApplicationCorrectionDispatch<Schema, Self, Visitor>,
    ) -> Visitor::Output {
        dispatch.capability::<capability::CorrectEntryCapability<Schema>>()
    }
}
