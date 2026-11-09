//! The existing small growth samples use the same typed record facts.
use super::*;
impl<const F: usize> Fragment for Records<2, F, false> {
    const NAME: &'static str = "native_records";
    type Bindings = Set;
    type Members = (Record<0, F>, Record<1, F>, Contracts<2, F, false>);
}
impl<const F: usize> Fragment for Records<2, F, true> {
    const NAME: &'static str = "native_records";
    type Bindings = Set;
    type Members = (Record<1, F>, Record<0, F>, Contracts<2, F, true>);
}
impl<const F: usize> Fragment for Records<4, F, false> {
    const NAME: &'static str = "native_records";
    type Bindings = Set;
    type Members = (
        Record<0, F>,
        Record<1, F>,
        Record<2, F>,
        Record<3, F>,
        Contracts<4, F, false>,
    );
}
impl<const F: usize> Fragment for Records<4, F, true> {
    const NAME: &'static str = "native_records";
    type Bindings = Set;
    type Members = (
        Record<3, F>,
        Record<2, F>,
        Record<1, F>,
        Record<0, F>,
        Contracts<4, F, true>,
    );
}
pub struct Contracts<const N: usize, const F: usize, const REVERSE: bool>;
impl<S: ApplicationSchema, const F: usize> fragment::Registration<S> for Contracts<2, F, false> {
    fn register(
        mut b: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        let op = operation();
        b = b
            .operation_read_record(op, Records::<2, F, false>, Record::<0, F>, Facts::<F>)
            .operation_write_record(op, Records::<2, F, false>, Record::<0, F>, Facts::<F>);
        b = b
            .operation_read_record(op, Records::<2, F, false>, Record::<1, F>, Facts::<F>)
            .operation_write_record(op, Records::<2, F, false>, Record::<1, F>, Facts::<F>);
        b = b
            .operation_create(op, fragment::entity(Records::<2, F, false>, Record::<1, F>))
            .operation_read_relation(
                op,
                fragment::relation(Records::<2, F, false>, Record::<1, F>, Owner::<1, F>),
            )
            .operation_link(
                op,
                fragment::relation(Records::<2, F, false>, Record::<1, F>, Owner::<1, F>),
            );
        Ok(b)
    }
}
impl<S: ApplicationSchema, const F: usize> fragment::Registration<S> for Contracts<2, F, true> {
    fn register(
        mut b: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        let op = operation();
        b = b
            .operation_read_record(op, Records::<2, F, true>, Record::<1, F>, Facts::<F>)
            .operation_write_record(op, Records::<2, F, true>, Record::<1, F>, Facts::<F>);
        b = b
            .operation_create(op, fragment::entity(Records::<2, F, true>, Record::<1, F>))
            .operation_read_relation(
                op,
                fragment::relation(Records::<2, F, true>, Record::<1, F>, Owner::<1, F>),
            )
            .operation_link(
                op,
                fragment::relation(Records::<2, F, true>, Record::<1, F>, Owner::<1, F>),
            );
        b = b
            .operation_read_record(op, Records::<2, F, true>, Record::<0, F>, Facts::<F>)
            .operation_write_record(op, Records::<2, F, true>, Record::<0, F>, Facts::<F>);
        Ok(b)
    }
}
impl<S: ApplicationSchema, const F: usize> fragment::Registration<S> for Contracts<4, F, false> {
    fn register(
        mut b: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        let op = operation();
        b = b
            .operation_read_record(op, Records::<4, F, false>, Record::<0, F>, Facts::<F>)
            .operation_write_record(op, Records::<4, F, false>, Record::<0, F>, Facts::<F>);
        b = b
            .operation_read_record(op, Records::<4, F, false>, Record::<1, F>, Facts::<F>)
            .operation_write_record(op, Records::<4, F, false>, Record::<1, F>, Facts::<F>);
        b = b
            .operation_create(op, fragment::entity(Records::<4, F, false>, Record::<1, F>))
            .operation_read_relation(
                op,
                fragment::relation(Records::<4, F, false>, Record::<1, F>, Owner::<1, F>),
            )
            .operation_link(
                op,
                fragment::relation(Records::<4, F, false>, Record::<1, F>, Owner::<1, F>),
            );
        b = b
            .operation_read_record(op, Records::<4, F, false>, Record::<2, F>, Facts::<F>)
            .operation_write_record(op, Records::<4, F, false>, Record::<2, F>, Facts::<F>);
        b = b
            .operation_create(op, fragment::entity(Records::<4, F, false>, Record::<2, F>))
            .operation_read_relation(
                op,
                fragment::relation(Records::<4, F, false>, Record::<2, F>, Owner::<2, F>),
            )
            .operation_link(
                op,
                fragment::relation(Records::<4, F, false>, Record::<2, F>, Owner::<2, F>),
            );
        b = b
            .operation_read_record(op, Records::<4, F, false>, Record::<3, F>, Facts::<F>)
            .operation_write_record(op, Records::<4, F, false>, Record::<3, F>, Facts::<F>);
        b = b
            .operation_create(op, fragment::entity(Records::<4, F, false>, Record::<3, F>))
            .operation_read_relation(
                op,
                fragment::relation(Records::<4, F, false>, Record::<3, F>, Owner::<3, F>),
            )
            .operation_link(
                op,
                fragment::relation(Records::<4, F, false>, Record::<3, F>, Owner::<3, F>),
            );
        Ok(b)
    }
}
impl<S: ApplicationSchema, const F: usize> fragment::Registration<S> for Contracts<4, F, true> {
    fn register(
        mut b: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        let op = operation();
        b = b
            .operation_read_record(op, Records::<4, F, true>, Record::<3, F>, Facts::<F>)
            .operation_write_record(op, Records::<4, F, true>, Record::<3, F>, Facts::<F>);
        b = b
            .operation_create(op, fragment::entity(Records::<4, F, true>, Record::<3, F>))
            .operation_read_relation(
                op,
                fragment::relation(Records::<4, F, true>, Record::<3, F>, Owner::<3, F>),
            )
            .operation_link(
                op,
                fragment::relation(Records::<4, F, true>, Record::<3, F>, Owner::<3, F>),
            );
        b = b
            .operation_read_record(op, Records::<4, F, true>, Record::<2, F>, Facts::<F>)
            .operation_write_record(op, Records::<4, F, true>, Record::<2, F>, Facts::<F>);
        b = b
            .operation_create(op, fragment::entity(Records::<4, F, true>, Record::<2, F>))
            .operation_read_relation(
                op,
                fragment::relation(Records::<4, F, true>, Record::<2, F>, Owner::<2, F>),
            )
            .operation_link(
                op,
                fragment::relation(Records::<4, F, true>, Record::<2, F>, Owner::<2, F>),
            );
        b = b
            .operation_read_record(op, Records::<4, F, true>, Record::<1, F>, Facts::<F>)
            .operation_write_record(op, Records::<4, F, true>, Record::<1, F>, Facts::<F>);
        b = b
            .operation_create(op, fragment::entity(Records::<4, F, true>, Record::<1, F>))
            .operation_read_relation(
                op,
                fragment::relation(Records::<4, F, true>, Record::<1, F>, Owner::<1, F>),
            )
            .operation_link(
                op,
                fragment::relation(Records::<4, F, true>, Record::<1, F>, Owner::<1, F>),
            );
        b = b
            .operation_read_record(op, Records::<4, F, true>, Record::<0, F>, Facts::<F>)
            .operation_write_record(op, Records::<4, F, true>, Record::<0, F>, Facts::<F>);
        Ok(b)
    }
}
