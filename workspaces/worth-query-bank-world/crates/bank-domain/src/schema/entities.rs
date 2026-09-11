use worth_query_decl::facade::worth_query_entity;

use super::BankSchema;

worth_query_entity!(pub Institution for BankSchema);
worth_query_entity!(pub ExternalPrincipalMapping for BankSchema);
worth_query_entity!(pub Principal for BankSchema);
worth_query_entity!(pub Customer for BankSchema);
worth_query_entity!(pub Business for BankSchema);
worth_query_entity!(pub Account for BankSchema);
worth_query_entity!(pub AccountAuthorization for BankSchema);
worth_query_entity!(pub EmployeeAssignment for BankSchema);
worth_query_entity!(pub PaymentIntent for BankSchema);
worth_query_entity!(pub Approval for BankSchema);
worth_query_entity!(pub JournalEntry for BankSchema);
worth_query_entity!(pub Posting for BankSchema);
