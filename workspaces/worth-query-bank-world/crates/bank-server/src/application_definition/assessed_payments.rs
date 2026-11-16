//! A real Bank composition whose rejection completes only with its current payment assessment.
use super::{composition::*, providers::*};
use bank_domain::{queries::PaymentDetailQueryBinding, schema::*};
use worth_query_host::facade::{
    declaration::{application_program::*, application_query::ApplicationQueryBinding},
    primary_graph::{
        WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
        WorthQueryRequiredOutputConnectionDenial,
    },
};

pub struct BankAssessedPaymentsApplication;
pub struct RejectedPaymentAssessmentFeature;
pub struct RejectedPaymentPort;
pub struct RejectedPaymentAssessmentInput;
pub struct RejectedPaymentAssessmentConnection;
pub struct RejectedPaymentAssessmentDelivery;

pub type RejectedPaymentAssessmentRoot = ApplicationOutputGraph<
    ApplicationConnectionRef<
        BankSchema,
        BankPaymentsFeature,
        RejectedPaymentPort,
        RejectedPaymentAssessmentFeature,
        RejectedPaymentAssessmentInput,
        RejectedPaymentAssessmentConnection,
    >,
    ApplicationOutputLeaf,
>;
pub type RejectedPaymentAssessmentAction = ApplicationActionReference<
    BankSchema,
    ApplicationRootComposition,
    BankPaymentsFeature,
    RejectPaymentMutationBinding,
    RejectedPaymentAssessmentDelivery,
>;

impl ApplicationFeature<BankSchema> for RejectedPaymentAssessmentFeature {
    type Inputs =
        ApplicationFeatureInputList<RejectedPaymentAssessmentInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "worth.bank.feature.rejected-payment-assessment.v1";
}
impl ApplicationOutputPort<BankSchema, BankPaymentsFeature> for RejectedPaymentPort {
    type Value = <PaymentDetailQueryBinding as ApplicationQueryBinding<BankSchema>>::ResultBinding;
    const IDENTITY: &'static str = "worth.bank.rejected-payment.source.v1";
}
impl ApplicationInputPort<BankSchema, RejectedPaymentAssessmentFeature>
    for RejectedPaymentAssessmentInput
{
    type Value = <PaymentDetailQueryBinding as ApplicationQueryBinding<BankSchema>>::ResultBinding;
    const IDENTITY: &'static str = "worth.bank.rejected-payment.assessment.v1";
    const REQUIRED: bool = true;
}
impl
    ApplicationOccurrenceConnectionBinding<
        BankSchema,
        BankPaymentsFeature,
        RejectedPaymentAssessmentFeature,
    > for RejectedPaymentAssessmentConnection
{
}
impl ApplicationConnectionIdentity for RejectedPaymentAssessmentConnection {
    const IDENTITY: &'static str = "worth.bank.rejected-payment.assessment-connection.v1";
}
impl WorthQueryApplicationRequiredOutputConnection<BankSchema>
    for RejectedPaymentAssessmentConnection
{
    type Source = RejectPaymentMutationBinding;
    type Demand = RejectedPaymentAssessmentDemand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        source: &RejectPayment,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(RejectedPaymentAssessmentDemand::new(source.payment))
    }
}
impl WorthQueryApplicationRequiredOutputSource<BankSchema, RejectedPaymentAssessmentConnection>
    for RejectPaymentMutationBinding
{
    fn demand_from_source(
        source: &RejectPayment,
    ) -> Result<RejectedPaymentAssessmentDemand, WorthQueryRequiredOutputConnectionDenial> {
        <RejectedPaymentAssessmentConnection as WorthQueryApplicationRequiredOutputConnection<
            BankSchema,
        >>::demand_from_source(source)
    }
}
impl ApplicationActionDelivery<BankSchema, RejectPaymentMutationBinding>
    for RejectedPaymentAssessmentDelivery
{
    type Mode =
        ApplicationFixedDeliveryMode<RejectedPaymentAssessmentRoot, ApplicationNoProjection>;
    const IDENTITY: &'static str = "worth.bank.rejected-payment.assessed-delivery.v1";
}
impl ApplicationFixedActionDelivery<BankSchema, RejectPaymentMutationBinding>
    for RejectedPaymentAssessmentDelivery
{
    type Root = RejectedPaymentAssessmentRoot;
    type Projection = ApplicationNoProjection;
}
impl ApplicationProgramDefinition<BankSchema> for BankAssessedPaymentsApplication {
    type Contributions = (
        BankAccountsProvider,
        BankPaymentsProvider,
        BankEstateProvider,
    );
    type Outputs = ApplicationProgramOutputs<RejectedPaymentAssessmentRoot>;
    type Rules = BankRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.bank.assessed-payments.v1");
    fn adoption_authorization<
        Visitor: ApplicationProgramAdoptionAuthorizationVisitor<BankSchema>,
    >(
        visitor: Visitor,
    ) -> Visitor::Output {
        visitor.capability::<BankProgramDeploymentBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut features =
            bank_features_with_rejection::<RejectedPaymentAssessmentDelivery>(true, true, true);
        features.push(
            ApplicationFeatureSpec::root::<BankSchema, RejectedPaymentAssessmentFeature>()
                .conditional_operation::<PublishApprovedPaymentAssessment>()
                .finish(),
        );
        features
    }
}
