use super::*;
impl<const F: usize> Fragment for Records<8, F, false> {
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
        ),
        (Record<7, F>, Contracts<F, false>),
    );
}
impl<const F: usize> Fragment for Records<8, F, true> {
    const NAME: &'static str = "native_records";
    type Bindings = Set;
    type Members = (
        (
            Record<7, F>,
            Record<6, F>,
            Record<5, F>,
            Record<4, F>,
            Record<3, F>,
            Record<2, F>,
            Record<1, F>,
        ),
        (Record<0, F>, Contracts<F, true>),
    );
}
pub struct Contracts<const F: usize, const REVERSE: bool>;
impl<S: ApplicationSchema, const F: usize> fragment::Registration<S> for Contracts<F, false> {
    fn register(
        mut b: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        b = b
            .operation_read_record(
                operation(),
                Records::<8, F, false>,
                Record::<0, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<8, F, false>,
                Record::<0, F>,
                Facts::<F>,
            );
        b = b
            .operation_read_record(
                operation(),
                Records::<8, F, false>,
                Record::<1, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<8, F, false>,
                Record::<1, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<8, F, false>, Record::<1, F>);
        let r = fragment::relation(Records::<8, F, false>, Record::<1, F>, Owner::<1, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<8, F, false>,
                Record::<2, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<8, F, false>,
                Record::<2, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<8, F, false>, Record::<2, F>);
        let r = fragment::relation(Records::<8, F, false>, Record::<2, F>, Owner::<2, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<8, F, false>,
                Record::<3, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<8, F, false>,
                Record::<3, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<8, F, false>, Record::<3, F>);
        let r = fragment::relation(Records::<8, F, false>, Record::<3, F>, Owner::<3, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<8, F, false>,
                Record::<4, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<8, F, false>,
                Record::<4, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<8, F, false>, Record::<4, F>);
        let r = fragment::relation(Records::<8, F, false>, Record::<4, F>, Owner::<4, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<8, F, false>,
                Record::<5, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<8, F, false>,
                Record::<5, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<8, F, false>, Record::<5, F>);
        let r = fragment::relation(Records::<8, F, false>, Record::<5, F>, Owner::<5, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<8, F, false>,
                Record::<6, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<8, F, false>,
                Record::<6, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<8, F, false>, Record::<6, F>);
        let r = fragment::relation(Records::<8, F, false>, Record::<6, F>, Owner::<6, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<8, F, false>,
                Record::<7, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<8, F, false>,
                Record::<7, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<8, F, false>, Record::<7, F>);
        let r = fragment::relation(Records::<8, F, false>, Record::<7, F>, Owner::<7, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        Ok(b)
    }
}
impl<S: ApplicationSchema, const F: usize> fragment::Registration<S> for Contracts<F, true> {
    fn register(
        mut b: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        b = b
            .operation_read_record(
                operation(),
                Records::<8, F, true>,
                Record::<0, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<8, F, true>,
                Record::<0, F>,
                Facts::<F>,
            );
        b = b
            .operation_read_record(
                operation(),
                Records::<8, F, true>,
                Record::<1, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<8, F, true>,
                Record::<1, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<8, F, true>, Record::<1, F>);
        let r = fragment::relation(Records::<8, F, true>, Record::<1, F>, Owner::<1, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<8, F, true>,
                Record::<2, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<8, F, true>,
                Record::<2, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<8, F, true>, Record::<2, F>);
        let r = fragment::relation(Records::<8, F, true>, Record::<2, F>, Owner::<2, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<8, F, true>,
                Record::<3, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<8, F, true>,
                Record::<3, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<8, F, true>, Record::<3, F>);
        let r = fragment::relation(Records::<8, F, true>, Record::<3, F>, Owner::<3, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<8, F, true>,
                Record::<4, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<8, F, true>,
                Record::<4, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<8, F, true>, Record::<4, F>);
        let r = fragment::relation(Records::<8, F, true>, Record::<4, F>, Owner::<4, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<8, F, true>,
                Record::<5, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<8, F, true>,
                Record::<5, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<8, F, true>, Record::<5, F>);
        let r = fragment::relation(Records::<8, F, true>, Record::<5, F>, Owner::<5, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<8, F, true>,
                Record::<6, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<8, F, true>,
                Record::<6, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<8, F, true>, Record::<6, F>);
        let r = fragment::relation(Records::<8, F, true>, Record::<6, F>, Owner::<6, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<8, F, true>,
                Record::<7, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<8, F, true>,
                Record::<7, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<8, F, true>, Record::<7, F>);
        let r = fragment::relation(Records::<8, F, true>, Record::<7, F>, Owner::<7, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        Ok(b)
    }
}
