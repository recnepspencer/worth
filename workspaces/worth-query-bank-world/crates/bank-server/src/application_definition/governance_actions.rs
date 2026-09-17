use bank_domain::schema::{
    ApproveEstateEmergencyAccessOperation, BankSchema, CompleteEstateMandatoryReviewOperation,
    DelegateEstateCapabilityOperation, RequestEstateEmergencyAccessOperation,
    RevokeEstateCapabilityOperation, RevokeEstateEmergencyAccessOperation,
};
use worth_query_host::facade::declaration::application_program::{
    ApplicationActionLeaf, ApplicationActionList, ApplicationOperationActionRef,
};

use super::composition::BankEstateFeature;

pub(super) type BankEstateGovernanceActions = ApplicationActionList<
    ApplicationOperationActionRef<
        BankSchema,
        BankEstateFeature,
        RequestEstateEmergencyAccessOperation,
    >,
    ApplicationActionList<
        ApplicationOperationActionRef<
            BankSchema,
            BankEstateFeature,
            ApproveEstateEmergencyAccessOperation,
        >,
        ApplicationActionList<
            ApplicationOperationActionRef<
                BankSchema,
                BankEstateFeature,
                RevokeEstateEmergencyAccessOperation,
            >,
            ApplicationActionList<
                ApplicationOperationActionRef<
                    BankSchema,
                    BankEstateFeature,
                    CompleteEstateMandatoryReviewOperation,
                >,
                ApplicationActionList<
                    ApplicationOperationActionRef<
                        BankSchema,
                        BankEstateFeature,
                        DelegateEstateCapabilityOperation,
                    >,
                    ApplicationActionList<
                        ApplicationOperationActionRef<
                            BankSchema,
                            BankEstateFeature,
                            RevokeEstateCapabilityOperation,
                        >,
                        ApplicationActionLeaf,
                    >,
                >,
            >,
        >,
    >,
>;

#[cfg(test)]
mod tests;
