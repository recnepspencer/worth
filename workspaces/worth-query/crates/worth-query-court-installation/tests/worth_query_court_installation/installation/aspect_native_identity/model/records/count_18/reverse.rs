use super::*;

impl<S: ApplicationSchema, const F: usize> fragment::Registration<S> for Contracts<F, true> {
    fn register(
        mut b: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        b = b
            .operation_read_record(
                operation(),
                Records::<18, F, true>,
                Record::<0, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<18, F, true>,
                Record::<0, F>,
                Facts::<F>,
            );
        b = b
            .operation_read_record(
                operation(),
                Records::<18, F, true>,
                Record::<1, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<18, F, true>,
                Record::<1, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<18, F, true>, Record::<1, F>);
        let r = fragment::relation(Records::<18, F, true>, Record::<1, F>, Owner::<1, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<18, F, true>,
                Record::<2, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<18, F, true>,
                Record::<2, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<18, F, true>, Record::<2, F>);
        let r = fragment::relation(Records::<18, F, true>, Record::<2, F>, Owner::<2, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<18, F, true>,
                Record::<3, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<18, F, true>,
                Record::<3, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<18, F, true>, Record::<3, F>);
        let r = fragment::relation(Records::<18, F, true>, Record::<3, F>, Owner::<3, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<18, F, true>,
                Record::<4, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<18, F, true>,
                Record::<4, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<18, F, true>, Record::<4, F>);
        let r = fragment::relation(Records::<18, F, true>, Record::<4, F>, Owner::<4, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<18, F, true>,
                Record::<5, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<18, F, true>,
                Record::<5, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<18, F, true>, Record::<5, F>);
        let r = fragment::relation(Records::<18, F, true>, Record::<5, F>, Owner::<5, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<18, F, true>,
                Record::<6, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<18, F, true>,
                Record::<6, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<18, F, true>, Record::<6, F>);
        let r = fragment::relation(Records::<18, F, true>, Record::<6, F>, Owner::<6, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<18, F, true>,
                Record::<7, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<18, F, true>,
                Record::<7, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<18, F, true>, Record::<7, F>);
        let r = fragment::relation(Records::<18, F, true>, Record::<7, F>, Owner::<7, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<18, F, true>,
                Record::<8, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<18, F, true>,
                Record::<8, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<18, F, true>, Record::<8, F>);
        let r = fragment::relation(Records::<18, F, true>, Record::<8, F>, Owner::<8, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<18, F, true>,
                Record::<9, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<18, F, true>,
                Record::<9, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<18, F, true>, Record::<9, F>);
        let r = fragment::relation(Records::<18, F, true>, Record::<9, F>, Owner::<9, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<18, F, true>,
                Record::<10, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<18, F, true>,
                Record::<10, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<18, F, true>, Record::<10, F>);
        let r = fragment::relation(Records::<18, F, true>, Record::<10, F>, Owner::<10, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<18, F, true>,
                Record::<11, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<18, F, true>,
                Record::<11, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<18, F, true>, Record::<11, F>);
        let r = fragment::relation(Records::<18, F, true>, Record::<11, F>, Owner::<11, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<18, F, true>,
                Record::<12, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<18, F, true>,
                Record::<12, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<18, F, true>, Record::<12, F>);
        let r = fragment::relation(Records::<18, F, true>, Record::<12, F>, Owner::<12, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<18, F, true>,
                Record::<13, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<18, F, true>,
                Record::<13, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<18, F, true>, Record::<13, F>);
        let r = fragment::relation(Records::<18, F, true>, Record::<13, F>, Owner::<13, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<18, F, true>,
                Record::<14, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<18, F, true>,
                Record::<14, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<18, F, true>, Record::<14, F>);
        let r = fragment::relation(Records::<18, F, true>, Record::<14, F>, Owner::<14, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<18, F, true>,
                Record::<15, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<18, F, true>,
                Record::<15, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<18, F, true>, Record::<15, F>);
        let r = fragment::relation(Records::<18, F, true>, Record::<15, F>, Owner::<15, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<18, F, true>,
                Record::<16, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<18, F, true>,
                Record::<16, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<18, F, true>, Record::<16, F>);
        let r = fragment::relation(Records::<18, F, true>, Record::<16, F>, Owner::<16, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        b = b
            .operation_read_record(
                operation(),
                Records::<18, F, true>,
                Record::<17, F>,
                Facts::<F>,
            )
            .operation_write_record(
                operation(),
                Records::<18, F, true>,
                Record::<17, F>,
                Facts::<F>,
            );
        let e = fragment::entity(Records::<18, F, true>, Record::<17, F>);
        let r = fragment::relation(Records::<18, F, true>, Record::<17, F>, Owner::<17, F>);
        b = b
            .operation_create(operation(), e)
            .operation_read_relation(operation(), r)
            .operation_link(operation(), r);
        Ok(b)
    }
}
