//! Typed recorded label inverse, derived only from the original captured pre-image.
use super::super::*;
use super::label_edit::*;
use worth_query_declaration::facade::{
    application_aftermath::*, application_query::*, application_schema::*,
};

pub(in crate::domain_computation::primary_graph) struct RecordedLabelEdit;
impl ApplicationCorrectionBinding<IdentityExecutionSchema> for RecordedLabelEdit {
    const IDENTITY: &'static str = "worth.query.test.recorded-label-edit.v1";
    type Source = LabelBinding;
    type InverseIntent = LabelEdit;
    type Disclosure = TestAccountSourceBinding;
    fn disclosure_parameters(
        original: &LabelEdit,
    ) -> Result<
        ApplicationQueryParameterSet<AccountSummaryQuery>,
        ApplicationCorrectionDerivationDenial,
    > {
        Ok(ApplicationQueryParameterSet::new()
            .bind(
                status_parameter::<AccountSummaryQuery>(),
                original.status.clone(),
            )
            .unwrap())
    }
    fn source_definition() -> ApplicationOperationDefinitionBuilder<
        IdentityExecutionSchema,
        EditLabel,
        LabelEdit,
        true,
        false,
    > {
        EditLabel::reference().definition().no_external_effect()
    }
    fn mechanism() -> DeclaredCorrectionMechanism<IdentityExecutionSchema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::new(
                [DeclaredPreImageLocus::from_field(AccountLabel::reference())],
                512,
            )
            .unwrap(),
        )
    }
    fn derive_correction(
        original: &LabelEdit,
        material: ApplicationCorrectionSourceMaterial<'_, IdentityExecutionSchema>,
    ) -> Result<LabelEdit, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::{
            InvalidPreImage, MissingPreImage, UnsupportedSourceMeaning,
        };
        let ApplicationCorrectionSourceMaterial::RecordedInverse(preimage) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let prior = preimage
            .field(&DeclaredPreImageLocus::from_field(AccountLabel::reference()))
            .ok_or(MissingPreImage)?;
        Ok(LabelEdit {
            status: original.status.clone(),
            label: StringApplicationValueBinding::decode(prior).map_err(|_| InvalidPreImage)?,
        })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("label-inverse:{key}"))
    }
    fn visit<Visitor: ApplicationCorrectionVisitor<IdentityExecutionSchema>>(
        dispatch: ApplicationCorrectionDispatch<IdentityExecutionSchema, Self, Visitor>,
    ) -> Visitor::Output {
        dispatch.conventional()
    }
}
