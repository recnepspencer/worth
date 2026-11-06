use super::{BankAccounts, BankEstate, BankPayments};
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Clone, Copy)]
pub struct BankSchema;
impl Fragment for BankSchema {
    const NAME: &'static str = "BankSchema";
    type Bindings = NoBindings;
    type Members = (BankAccounts, BankPayments, BankEstate);
}
impl ApplicationSchema for BankSchema {
    const OWNER: &'static str = "WORTH.bank";
    const NAME: &'static str = "BankSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        fragment::visit_correction(Self, source, visitor)
    }
}

impl ApplicationSchemaComposition for BankSchema {
    type Contributions = <Self as Fragment>::Members;
}
