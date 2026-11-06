//! Four applications use two fragments extending one independently owned entity.
use super::*;
pub struct ExtensionItem;
pub struct ExtensionFact<const I: usize>;
pub struct ExtensionOwner;
pub struct ExtensionFirst;
pub struct ExtensionSecond;
pub struct ExtensionApplication<const I: usize>;
impl<const I: usize> ApplicationSchema for ExtensionApplication<I> {
    const NAME: &'static str = "extension_instantiation";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        struct Root;
        impl Fragment for Root {
            const NAME: &'static str = "root";
            type Bindings = NoBindings;
            type Members = (ExtensionFirst, ExtensionOwner, ExtensionSecond);
        }
        ApplicationSchemaDeclarationBuilder::from_fragment(Root)?.build()
    }
}
impl Entity for ExtensionItem {
    const NAME: &'static str = "extended_item";
    type Aspects = ();
    type Relations = ();
}
impl Fragment for ExtensionOwner {
    const NAME: &'static str = "extension_owner";
    type Bindings = Set;
    type Members = (ExtensionItem,);
}
impl OperationCreates<Edit> for ExtensionItem {}
impl<const I: usize> Aspect for ExtensionFact<I> {
    const NAME: &'static str = model::NAMES[I];
    type Row = u64;
}
impl Fragment for ExtensionFirst {
    const NAME: &'static str = "extensionfirst";
    type Bindings = Set;
    type Members = (
        Extend<
            ExtensionItem,
            (
                (
                    Option<ExtensionFact<0>>,
                    Option<ExtensionFact<1>>,
                    Option<ExtensionFact<2>>,
                    Option<ExtensionFact<3>>,
                    Option<ExtensionFact<4>>,
                    Option<ExtensionFact<5>>,
                    Option<ExtensionFact<6>>,
                    Option<ExtensionFact<7>>,
                ),
                (
                    Option<ExtensionFact<8>>,
                    Option<ExtensionFact<9>>,
                    Option<ExtensionFact<10>>,
                    Option<ExtensionFact<11>>,
                    Option<ExtensionFact<12>>,
                    Option<ExtensionFact<13>>,
                    Option<ExtensionFact<14>>,
                    Option<ExtensionFact<15>>,
                ),
                (
                    Option<ExtensionFact<16>>,
                    Option<ExtensionFact<17>>,
                    Option<ExtensionFact<18>>,
                    Option<ExtensionFact<19>>,
                ),
            ),
        >,
    );
}
impl Fragment for ExtensionSecond {
    const NAME: &'static str = "extensionsecond";
    type Bindings = Set;
    type Members = (
        Extend<
            ExtensionItem,
            (
                (
                    Option<ExtensionFact<20>>,
                    Option<ExtensionFact<21>>,
                    Option<ExtensionFact<22>>,
                    Option<ExtensionFact<23>>,
                    Option<ExtensionFact<24>>,
                    Option<ExtensionFact<25>>,
                    Option<ExtensionFact<26>>,
                    Option<ExtensionFact<27>>,
                ),
                (
                    Option<ExtensionFact<28>>,
                    Option<ExtensionFact<29>>,
                    Option<ExtensionFact<30>>,
                    Option<ExtensionFact<31>>,
                    Option<ExtensionFact<32>>,
                    Option<ExtensionFact<33>>,
                    Option<ExtensionFact<34>>,
                    Option<ExtensionFact<35>>,
                ),
                (
                    Option<ExtensionFact<36>>,
                    Option<ExtensionFact<37>>,
                    Option<ExtensionFact<38>>,
                    Option<ExtensionFact<39>>,
                ),
            ),
        >,
    );
}
mod application_0;
pub use application_0::extension_application_0;
mod application_1;
pub use application_1::extension_application_1;
mod application_2;
pub use application_2::extension_application_2;
mod application_3;
pub use application_3::extension_application_3;

mod view_access;
pub use view_access::extension_views;
