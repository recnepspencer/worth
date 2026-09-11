use worth_query_decl::facade::worth_query_entity;

use crate::schema::BankSchema;

worth_query_entity!(pub Branch for BankSchema);
worth_query_entity!(pub CapabilityGrant for BankSchema);
worth_query_entity!(pub DeathNotice for BankSchema);
worth_query_entity!(pub EmergencyAccess for BankSchema);
worth_query_entity!(pub EstateCase for BankSchema);
worth_query_entity!(pub LegalAuthority for BankSchema);
worth_query_entity!(pub MandatoryReview for BankSchema);
