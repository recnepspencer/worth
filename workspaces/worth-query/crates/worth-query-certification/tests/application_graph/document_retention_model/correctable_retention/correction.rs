use super::*;
use worth_query_host::facade::declaration::{application_aftermath::*, application_query::*};

pub struct RestoreReviewedRetention;
impl ApplicationMutationInputScope<Schema, SetRetentionBinding> for SetRetentionInput {
    fn scope_for_binding(
        &self,
    ) -> <SetRetentionBinding as ApplicationMutationBinding<Schema>>::ScopeBinding {
        SetRetentionIntent {
            input: self.clone(),
        }
        .scope_binding()
    }
}
impl ApplicationCorrectionBinding<Schema> for RestoreReviewedRetention {
    const IDENTITY: &'static str = "document-retention.restore-reviewed.v1";
    type Source = CorrectableRetentionBinding;
    type InverseIntent = SetRetentionIntent;
    type Disclosure = DocumentRetentionQueryBinding;
    fn source_definition() -> ApplicationOperationDefinitionBuilder<
        Schema,
        CorrectableRetention,
        SetRetentionInput,
        true,
        false,
    > {
        CorrectableRetention::reference()
            .definition()
            .no_external_effect()
    }
    fn disclosure_parameters(
        _: &SetRetentionInput,
    ) -> Result<
        ApplicationQueryParameterSet<DocumentRetentionQuery>,
        ApplicationCorrectionDerivationDenial,
    > {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::new(
                [DeclaredPreImageLocus::from_field(
                    DocumentRetentionField::reference(),
                )],
                U64ApplicationValueBinding::encode(&u64::MAX)
                    .unwrap()
                    .semantic_byte_width(),
            )
            .unwrap(),
        )
    }
    fn derive_correction(
        original: &SetRetentionInput,
        material: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<SetRetentionIntent, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::*;
        let ApplicationCorrectionSourceMaterial::RecordedInverse(preimage) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let retention_days = U64ApplicationValueBinding::decode(
            preimage
                .field(&DeclaredPreImageLocus::from_field(
                    DocumentRetentionField::reference(),
                ))
                .ok_or(MissingPreImage)?,
        )
        .map_err(|_| InvalidPreImage)?;
        Ok(SetRetentionIntent {
            input: SetRetentionInput {
                identity: original.identity.clone(),
                retention_days,
            },
        })
    }
    fn inverse_key(key: &str) -> Result<u64, ApplicationValueEncodeDenial> {
        // The fixture presents decimal command keys, using the binding's actual key type.
        key.parse().map_err(|_| ApplicationValueEncodeDenial::CodecRejected {
            binding_identity: worth_query_host::facade::declaration::portable_identity::WorthQueryPortableTypeIdentity::declared(Self::IDENTITY),
            reason: "correction command key must be decimal",
        })
    }
    fn visit<V: ApplicationCorrectionVisitor<Schema>>(
        dispatch: ApplicationCorrectionDispatch<Schema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
