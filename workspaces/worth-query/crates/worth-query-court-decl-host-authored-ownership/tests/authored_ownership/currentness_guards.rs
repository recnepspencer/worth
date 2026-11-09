//! One producer owns an orbit anchor and a fork; each guard court shrinks one of them.
//!
//! Both postcommit-currentness guard courts share this single scenario type.
//! Every scenario type instantiates the whole Query runtime again, and a second
//! one moved this court executable into the macOS arm64 exec-placement band in
//! which a fraction of launches is refused before `main`.
use super::*;
use coupled::producer::{Binding as Preserve, Operation as PreserveOp};
use sheet_boundary::{split_walk, Number, Occurrence};
pub type GuardModel = Model<70, heterogeneous::Absent, Guards>;
/// Source parameter values; each edit addresses exactly one owned structure.
pub const JOIN_ORBIT: u64 = 1;
pub const RETIRE_ANCHOR: u64 = 2;
pub const RETIRE_FORK_ROOT: u64 = 3;
pub fn declare<S: Scenario>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    if S::MODE != 70 {
        return schema;
    }
    let initial =
        ApplicationOperationRef::<S, producer::Operation, query::Input>::from_declaration();
    let preserve = PreserveOp::reference();
    let schema = schema
        .operation_create(initial, Occurrence::<S::Boundary>::reference())
        .operation_write(initial, Number::<S::Boundary>::at("Number"))
        .operation_read_entity(preserve, Occurrence::<S::Boundary>::reference())
        .operation_read_field(preserve, Number::<S::Boundary>::at("Number"))
        .operation_delete(preserve, Occurrence::<S::Boundary>::reference());
    split_walk::declare(foreign_aggregate::declare(schema))
}
pub struct Decision<S: Scenario> {
    orbit: foreign_aggregate::Decision<S>,
    fork: split_walk::Decision<S>,
}
pub struct Guards;
impl sheet_boundary::Profile for Guards {
    type Relations = (foreign_aggregate::Next,);
    const UNIQUE: bool = true;
    fn declaration<S: Scenario<Boundary = Self>>(
    ) -> Result<ApplicationSchemaDeclaration<S>, ApplicationSchemaDeclarationDenial> {
        if S::MODE == 43 {
            ApplicationSchemaDeclarationBuilder::from_fragment(schema::OwnershipRoot::<S, true>(
                PhantomData,
            ))?
            .build()
        } else {
            ApplicationSchemaDeclarationBuilder::from_fragment(schema::OwnershipRoot::<S, false>(
                PhantomData,
            ))?
            .build()
        }
    }
    type Decision<S: Scenario> = Decision<S>;
    fn initial<S: Scenario>(
        writer: &mut CandidateWriter<'_, S, producer::Binding<S>>,
        source: &WorthQueryApplicationEffectEntity<S, Node<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        foreign_aggregate::initial(writer, source)?;
        split_walk::initial(writer, source)
    }
    fn read<S: Scenario>(
        reader: &mut DecisionReader<'_, '_, '_, S, Preserve<S>>,
    ) -> Result<Option<Decision<S>>, HandlerExecutionDenial> {
        let source = reader.resolve_entity(schema::id::<S>(), 10)?;
        let value = reader.field(&source, schema::value::<S>())?;
        Ok(Some(Decision {
            orbit: foreign_aggregate::read(reader, value)?,
            fork: split_walk::read(reader, value)?,
        }))
    }
    fn trim<S: Scenario>(
        writer: &mut CandidateWriter<'_, S, Preserve<S>>,
        decision: Option<Decision<S>>,
        _: &WorthQueryApplicationEffectEntity<S, Node<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let decision = decision.expect("currentness guard decision");
        foreign_aggregate::trim(writer, decision.orbit)?;
        split_walk::shrink(writer, decision.fork)
    }
}
