use worth_query_decl::facade::worth_query_operation_reads;

use super::*;

worth_query_operation_reads!(CreatePersonalAccountOperation => [PersonalOwner, InstitutionAccount, BusinessAccount]);
worth_query_operation_reads!(CreateBusinessAccountOperation => [BusinessAccount, InstitutionAccount, PersonalOwner]);
worth_query_operation_reads!(ApplyOpeningFundingOperation => [InstitutionAccount, InstitutionCashAccount, PersonalOwner, BusinessAccount, PostingAccount, JournalPosting, JournalReversal]);
worth_query_operation_reads!(DepositOperation => [InstitutionAccount, InstitutionCashAccount, PersonalOwner, BusinessAccount, PostingAccount, JournalPosting, JournalReversal]);
worth_query_operation_reads!(WithdrawOperation => [InstitutionAccount, InstitutionCashAccount, PersonalOwner, BusinessAccount, PostingAccount, JournalPosting, JournalReversal]);
worth_query_operation_reads!(SendMoneyOperation => [PersonalOwner, InstitutionAccount, BusinessAccount, PostingAccount, JournalPosting, JournalReversal]);
worth_query_operation_reads!(InitiateBusinessPaymentOperation => [BusinessAccount, PersonalOwner, InstitutionAccount]);
worth_query_operation_reads!(ApprovePaymentOperation => [PaymentSource, PaymentDestination, PaymentBusiness, PaymentInitiator, PaymentApproval, ApprovalPrincipal, InstitutionAccount, PersonalOwner, BusinessAccount, PostingAccount, JournalPosting, JournalReversal]);
worth_query_operation_reads!(RejectPaymentOperation => [PaymentSource, PaymentDestination, PaymentBusiness, PaymentInitiator, PaymentApproval, ApprovalPrincipal, InstitutionAccount, PersonalOwner, BusinessAccount]);
worth_query_operation_reads!(GrantAccountAuthorizationOperation => [AccountAuthorizedUser, AuthorizationAccount, InstitutionAccount, PersonalOwner, BusinessAccount]);
worth_query_operation_reads!(RevokeAccountAuthorizationOperation => [AccountAuthorization, AccountAuthorizedUser, AuthorizationAccount, InstitutionAccount, PersonalOwner, BusinessAccount]);
worth_query_operation_reads!(ReverseJournalOperation => [InstitutionAccount, PersonalOwner, BusinessAccount, JournalPosting, PostingAccount, JournalReversal]);
