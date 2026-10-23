use crate::{estate::EstateAction, proposals::BankIdempotencyKey, queries::*, schema::*};
use worth_query_decl::facade::{
    application_aftermath::*, application_query::ApplicationQueryParameterSet,
    application_schema::*,
};

pub struct RestoreRecordedEstateFreeze;
impl ApplicationCorrectionBinding<BankSchema> for RestoreRecordedEstateFreeze {
    const IDENTITY: &'static str = "bank.estate.restore-recorded-freeze.v1";
    type Source = FreezeEstateAccountMutationBinding;
    type InverseIntent = RestoreFrozenEstateAccount;
    type Disclosure = EstateCaseOverviewQueryBinding;
    fn source_definition() -> ApplicationOperationDefinitionBuilder<
        BankSchema,
        FreezeEstateAccountOperation,
        EstateAction,
        true,
        false,
    > {
        FreezeEstateAccountOperation::reference()
            .definition()
            .no_external_effect()
    }
    fn disclosure_parameters(
        _: &EstateAction,
    ) -> Result<
        ApplicationQueryParameterSet<EstateCaseOverviewQuery>,
        ApplicationCorrectionDerivationDenial,
    > {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<BankSchema> {
        let width = [
            AccountStatus::Open,
            AccountStatus::Frozen,
            AccountStatus::Closed,
        ]
        .into_iter()
        .map(|status| {
            AccountStatusBinding::encode(&status)
                .unwrap()
                .semantic_byte_width()
        })
        .max()
        .unwrap();
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::new(
                [DeclaredPreImageLocus::from_field(Status::reference())],
                width,
            )
            .expect("account status codec has a finite declared representation"),
        )
    }
    fn derive_correction(
        original: &EstateAction,
        material: ApplicationCorrectionSourceMaterial<'_, BankSchema>,
    ) -> Result<RestoreFrozenEstateAccount, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::*;
        let EstateAction::FreezeAccount { estate, account } = *original else {
            return Err(InvalidOriginalInput);
        };
        let ApplicationCorrectionSourceMaterial::RecordedInverse(preimage) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let prior_status = AccountStatusBinding::decode(
            preimage
                .field(&DeclaredPreImageLocus::from_field(Status::reference()))
                .ok_or(MissingPreImage)?,
        )
        .map_err(|_| InvalidPreImage)?;
        if prior_status != AccountStatus::Open {
            return Err(InvalidPreImage);
        }
        Ok(RestoreFrozenEstateAccount {
            estate,
            account,
            prior_status,
        })
    }
    fn inverse_key(key: &str) -> Result<BankIdempotencyKey, ApplicationValueEncodeDenial> {
        BankIdempotencyKey::new(key).map_err(|_| {
            ApplicationValueEncodeDenial::Validation(ApplicationValueValidationDenial::rejected(
                worth_query_decl::facade::portable_identity::WorthQueryPortableTypeIdentity::declared(
                    "bank.application-mutation-client-key.v1",
                ),
                "expected a Bank idempotency key",
            ))
        })
    }
    fn visit<V: ApplicationCorrectionVisitor<BankSchema>>(
        dispatch: ApplicationCorrectionDispatch<BankSchema, Self, V>,
    ) -> V::Output {
        dispatch.capability::<RestoreFrozenEstateAccountCapability>()
    }
}
