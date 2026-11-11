//! Forty native records exercise admitted installation identity budgets.
use super::*;
impl<const F: usize> Fragment for Records<40, F, false> {
    const NAME: &'static str = "native_records";
    type Bindings = Set;
    type Members = (
        (
            Record<0, F>,
            Record<1, F>,
            Record<2, F>,
            Record<3, F>,
            Record<4, F>,
            Record<5, F>,
            Record<6, F>,
            Record<7, F>,
        ),
        (
            Record<8, F>,
            Record<9, F>,
            Record<10, F>,
            Record<11, F>,
            Record<12, F>,
            Record<13, F>,
            Record<14, F>,
            Record<15, F>,
        ),
        (
            Record<16, F>,
            Record<17, F>,
            Record<18, F>,
            Record<19, F>,
            Record<20, F>,
            Record<21, F>,
            Record<22, F>,
            Record<23, F>,
        ),
        (
            Record<24, F>,
            Record<25, F>,
            Record<26, F>,
            Record<27, F>,
            Record<28, F>,
            Record<29, F>,
            Record<30, F>,
            Record<31, F>,
        ),
        (
            Record<32, F>,
            Record<33, F>,
            Record<34, F>,
            Record<35, F>,
            Record<36, F>,
            Record<37, F>,
            Record<38, F>,
            Record<39, F>,
        ),
        Contracts<F, false>,
    );
}
impl<const F: usize> Fragment for Records<40, F, true> {
    const NAME: &'static str = "native_records";
    type Bindings = Set;
    type Members = (
        (
            Record<39, F>,
            Record<38, F>,
            Record<37, F>,
            Record<36, F>,
            Record<35, F>,
            Record<34, F>,
            Record<33, F>,
            Record<32, F>,
        ),
        (
            Record<31, F>,
            Record<30, F>,
            Record<29, F>,
            Record<28, F>,
            Record<27, F>,
            Record<26, F>,
            Record<25, F>,
            Record<24, F>,
        ),
        (
            Record<23, F>,
            Record<22, F>,
            Record<21, F>,
            Record<20, F>,
            Record<19, F>,
            Record<18, F>,
            Record<17, F>,
            Record<16, F>,
        ),
        (
            Record<15, F>,
            Record<14, F>,
            Record<13, F>,
            Record<12, F>,
            Record<11, F>,
            Record<10, F>,
            Record<9, F>,
            Record<8, F>,
        ),
        (
            Record<7, F>,
            Record<6, F>,
            Record<5, F>,
            Record<4, F>,
            Record<3, F>,
            Record<2, F>,
            Record<1, F>,
            Record<0, F>,
        ),
        Contracts<F, true>,
    );
}
pub struct Contracts<const F: usize, const REVERSE: bool>;
impl<S: ApplicationSchema, const F: usize> fragment::Registration<S> for Contracts<F, false> {
    fn register(
        mut b: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        let op = operation();
        macro_rules! access { ($($i:literal),*) => { $(
 b = b.operation_read_record(op, Records::<40,F,false>, Record::<$i,F>, Facts::<F>)
 .operation_write_record(op, Records::<40,F,false>, Record::<$i,F>, Facts::<F>);
 )* }; }
        access!(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39
        );
        macro_rules! links { ($($i:literal),*) => { $(
 b = b.operation_create(op, fragment::entity(Records::<40,F,false>, Record::<$i,F>))
 .operation_read_relation(op, fragment::relation(Records::<40,F,false>, Record::<$i,F>, Owner::<$i,F>))
 .operation_link(op, fragment::relation(Records::<40,F,false>, Record::<$i,F>, Owner::<$i,F>));
 )* }; }
        links!(
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39
        );
        Ok(b)
    }
}
impl<S: ApplicationSchema, const F: usize> fragment::Registration<S> for Contracts<F, true> {
    fn register(
        mut b: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        let op = operation();
        macro_rules! access { ($($i:literal),*) => { $(
 b = b.operation_read_record(op, Records::<40,F,true>, Record::<$i,F>, Facts::<F>)
 .operation_write_record(op, Records::<40,F,true>, Record::<$i,F>, Facts::<F>);
 )* }; }
        access!(
            39, 38, 37, 36, 35, 34, 33, 32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18,
            17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0
        );
        macro_rules! links { ($($i:literal),*) => { $(
 b = b.operation_create(op, fragment::entity(Records::<40,F,true>, Record::<$i,F>))
 .operation_read_relation(op, fragment::relation(Records::<40,F,true>, Record::<$i,F>, Owner::<$i,F>))
 .operation_link(op, fragment::relation(Records::<40,F,true>, Record::<$i,F>, Owner::<$i,F>));
 )* }; }
        links!(
            39, 38, 37, 36, 35, 34, 33, 32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18,
            17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1
        );
        Ok(b)
    }
}
