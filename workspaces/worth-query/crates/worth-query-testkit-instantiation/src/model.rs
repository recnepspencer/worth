//! One reusable fragment; application identity never changes its members.
use super::*;
pub struct Records;
pub struct Item;
pub struct Fact<const I: usize>;
pub struct Set;
pub struct Edit;
pub struct Application<const I: usize>;
impl<const I: usize> ApplicationSchema for Application<I> {
    const NAME: &'static str = "instantiation";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::from_fragment(Records)?.build()
    }
}
impl Fragment for Records {
    const NAME: &'static str = "records";
    type Bindings = Set;
    type Members = (Item,);
}
impl Entity for Item {
    const NAME: &'static str = "item";
    type Aspects = (
        (
            Fact<0>,
            Fact<1>,
            Fact<2>,
            Fact<3>,
            Fact<4>,
            Fact<5>,
            Fact<6>,
            Fact<7>,
        ),
        (
            Fact<8>,
            Fact<9>,
            Fact<10>,
            Fact<11>,
            Fact<12>,
            Fact<13>,
            Fact<14>,
            Fact<15>,
        ),
        (
            Fact<16>,
            Fact<17>,
            Fact<18>,
            Fact<19>,
            Fact<20>,
            Fact<21>,
            Fact<22>,
            Fact<23>,
        ),
        (
            Fact<24>,
            Fact<25>,
            Fact<26>,
            Fact<27>,
            Fact<28>,
            Fact<29>,
            Fact<30>,
            Fact<31>,
        ),
        (
            Fact<32>,
            Fact<33>,
            Fact<34>,
            Fact<35>,
            Fact<36>,
            Fact<37>,
            Fact<38>,
            Fact<39>,
        ),
    );
    type Relations = ();
}
impl<const I: usize> Aspect for Fact<I> {
    const NAME: &'static str = NAMES[I];
    type Row = u64;
}
impl BindingSet for Set {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<u64> for Set {
    type Binding = U64ApplicationValueBinding;
}
impl OperationCreates<Edit> for Item {}
const NAMES: [&str; 40] = [
    "fact0", "fact1", "fact2", "fact3", "fact4", "fact5", "fact6", "fact7", "fact8", "fact9",
    "fact10", "fact11", "fact12", "fact13", "fact14", "fact15", "fact16", "fact17", "fact18",
    "fact19", "fact20", "fact21", "fact22", "fact23", "fact24", "fact25", "fact26", "fact27",
    "fact28", "fact29", "fact30", "fact31", "fact32", "fact33", "fact34", "fact35", "fact36",
    "fact37", "fact38", "fact39",
];
