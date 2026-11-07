//! Current installed authored identity and a real recorded pre-image inverse.
use super::super::*;
use worth_query_decl::facade::application_aftermath::*;
pub struct Family;
impl ApplicationAuthoredCorrectionFamily<Schema> for Family {
    const IDENTITY: &'static str = "owned-outputs.authored-member-family.v1";
    type Entity = Member;
    fn logical_key() -> ApplicationAuthoredLogicalKey<Schema, Member> {
        ApplicationAuthoredLogicalKey::from_field(MemberKey::at("MemberKey"))
    }
}
pub struct UndoRetire;
impl ApplicationCorrectionBinding<Schema> for UndoRetire {
    const IDENTITY: &'static str = "owned-outputs.authored-member-undo-retire.v1";
    type Source = super::retire::Binding;
    type InverseIntent = super::restore::Intent;
    type Disclosure = query::model::Binding;
    fn source_definition() -> ApplicationOperationDefinitionBuilder<
        Schema,
        super::retire::RetireMember,
        super::retire::Intent,
        true,
        false,
    > {
        super::retire::RetireMember::reference()
            .definition()
            .no_external_effect()
    }
    fn disclosure_parameters(
        _: &super::retire::Intent,
    ) -> Result<
        ApplicationQueryParameterSet<query::model::Query>,
        ApplicationCorrectionDerivationDenial,
    > {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn authored_identity() -> Option<ApplicationAuthoredCorrection<Schema>> {
        Some(ApplicationAuthoredCorrection::restore_retired::<
            Self,
            Family,
            super::retire::Retired,
            super::restore::Restored,
        >())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::new(
                vec![
                    DeclaredPreImageLocus::from_field(MemberKey::at("MemberKey")),
                    DeclaredPreImageLocus::from_field(MemberValue::at("MemberValue")),
                ],
                4096,
            )
            .with_relations([DeclaredPreImageRelationLocus::from_relation(
                ModelMember::reference(),
            )]),
        )
    }
    fn derive_correction(
        _: &super::retire::Intent,
        material: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<super::restore::Intent, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::{
            InvalidPreImage, MissingPreImage, UnsupportedSourceMeaning,
        };
        let ApplicationCorrectionSourceMaterial::RecordedInverse(fields) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let key = StringApplicationValueBinding::decode(
            fields
                .field(&DeclaredPreImageLocus::from_field(MemberKey::at(
                    "MemberKey",
                )))
                .ok_or(MissingPreImage)?,
        )
        .map_err(|_| InvalidPreImage)?;
        let value = U64ApplicationValueBinding::decode(
            fields
                .field(&DeclaredPreImageLocus::from_field(MemberValue::at(
                    "MemberValue",
                )))
                .ok_or(MissingPreImage)?,
        )
        .map_err(|_| InvalidPreImage)?;
        Ok(super::restore::Intent { key, value })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("restore:{key}"))
    }
    fn visit<V: ApplicationCorrectionVisitor<Schema>>(
        dispatch: ApplicationCorrectionDispatch<Schema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
