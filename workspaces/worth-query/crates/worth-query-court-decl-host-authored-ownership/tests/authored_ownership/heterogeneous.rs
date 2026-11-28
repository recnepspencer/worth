//! Two genuinely different authored kinds share one Native co-anchor inverse.
use super::{lifecycle::*, *};
mod courts;
pub mod query;
pub struct BodyCreated<S>(PhantomData<fn() -> S>);
pub struct BodyRetired<S>(PhantomData<fn() -> S>);
impl<S: Scenario> WorthQueryApplicationOutputRole for BodyCreated<S> {
    type Schema = S;
    type Contract = CreatedOutputs;
    type Entity = Body;
    type Action = WorthQueryCreateOutput;
    type Cardinality = <S::Heterogeneous as Profile>::Births;
    const NAME: &'static str = "authored-body-created";
}
impl<S: Scenario> WorthQueryApplicationOutputRole for BodyRetired<S> {
    type Schema = S;
    type Contract = RetiredOutputs;
    type Entity = Body;
    type Action = WorthQueryRetireOutput;
    type Cardinality = <S::Heterogeneous as Profile>::Births;
    const NAME: &'static str = "authored-body-retired";
}
impl OperationCreates<CreateOp> for Body {}
impl OperationCreates<RestoreOp> for Body {}
impl OperationWrites<CreateOp> for BodyKey {}
impl OperationWrites<RestoreOp> for BodyKey {}
impl OperationReads<RetireOp> for BodyKey {}
impl OperationReads<RetireOp> for Body {}
impl OperationDeletes<RetireOp> for Body {}
pub fn active<S: Scenario>() -> bool {
    <S::Heterogeneous as Profile>::PRESENT
}
pub fn declare<S: Scenario>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    if !active::<S>() {
        return schema;
    }
    schema
        .operation_create(CreateOp::reference(), Body::reference())
        .operation_write(CreateOp::reference(), BodyKey::at("BodyKey"))
        .operation_create(RestoreOp::reference(), Body::reference())
        .operation_write(RestoreOp::reference(), BodyKey::at("BodyKey"))
        .operation_read_entity(RetireOp::reference(), Body::reference())
        .operation_read_field(RetireOp::reference(), BodyKey::at("BodyKey"))
        .operation_delete(RetireOp::reference(), Body::reference())
        .application_query(query::definition())
        .application_query_binding::<query::Binding<S>>()
}
pub struct Absent;
pub struct Present<const DECLARED: bool>;
/// A creation bears its body only when asked, and a retirement leaves bodies alone.
pub struct Optional;
pub trait Profile: Sized + 'static {
    const PRESENT: bool = false;
    /// A retirement finds and retires the body its creation bore.
    const RETIRES_BODY: bool = Self::PRESENT;
    /// How many bodies one creation bears.
    type Births: WorthQueryApplicationOutputCardinality;
    fn create<S: Scenario>(
        _: &Create,
        _: &WorthQueryApplicationEffectEntity<S, Node<S>>,
        _: &mut CandidateWriter<'_, S, CreateBinding<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Ok(())
    }
    fn retire<S: Scenario>(
        _: Option<WorthQueryInvariantMutationTarget<S, Body>>,
        _: &mut CandidateWriter<'_, S, RetireBinding<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Ok(())
    }
    fn restore<S: Scenario>(
        _: &Restore,
        _: &WorthQueryApplicationEffectEntity<S, Node<S>>,
        _: &mut CandidateWriter<'_, S, RestoreBinding<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Ok(())
    }
}
impl Profile for Absent {
    type Births = WorthQueryExactlyOneOutput;
}
impl Profile for Optional {
    const PRESENT: bool = true;
    const RETIRES_BODY: bool = false;
    type Births = WorthQueryAtMostOneOutput;
    fn create<S: Scenario>(
        input: &Create,
        scope: &WorthQueryApplicationEffectEntity<S, Node<S>>,
        writer: &mut CandidateWriter<'_, S, CreateBinding<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if input.create_co {
            Present::<true>::create(input, scope, writer)?;
        }
        Ok(())
    }
}
impl<const DECLARED: bool> Profile for Present<DECLARED> {
    const PRESENT: bool = true;
    type Births = WorthQueryExactlyOneOutput;
    fn create<S: Scenario>(
        input: &Create,
        scope: &WorthQueryApplicationEffectEntity<S, Node<S>>,
        writer: &mut CandidateWriter<'_, S, CreateBinding<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let body = writer.create_entity_in_context(
            scope,
            Body::reference(),
            WorthQueryApplicationEntityKey::new(format!("body-{}", input.id)).unwrap(),
        )?;
        writer.initialize_field(&body, BodyKey::at("BodyKey"), 10000 + input.id)?;
        writer.create_output::<BodyCreated<S>>(&body)?;
        Ok(())
    }
    fn retire<S: Scenario>(
        body: Option<WorthQueryInvariantMutationTarget<S, Body>>,
        writer: &mut CandidateWriter<'_, S, RetireBinding<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if let Some(body) = body {
            let body = writer.projected_entity(&body)?;
            writer.delete_entity(Body::reference(), &body)?;
            writer.retire_output::<BodyRetired<S>>(&body)?;
        }
        Ok(())
    }
    fn restore<S: Scenario>(
        input: &Restore,
        scope: &WorthQueryApplicationEffectEntity<S, Node<S>>,
        writer: &mut CandidateWriter<'_, S, RestoreBinding<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let body = writer.create_entity_in_context(
            scope,
            Body::reference(),
            WorthQueryApplicationEntityKey::new(format!("restored-body-{}", input.id)).unwrap(),
        )?;
        writer.initialize_field(&body, BodyKey::at("BodyKey"), input.co_values[0])?;
        writer.create_output::<BodyCreated<S>>(&body)?;
        Ok(())
    }
}

use worth_query_decl::facade::application_schema::fragment::{self, *};
pub struct Body;
impl Entity for Body {
    const NAME: &'static str = "Body";
    type Aspects = (BodyIdentity,);
    type Relations = ();
}
impl Body {
    pub fn reference<S: Scenario>() -> ApplicationEntityRef<S, Self> {
        entity(OwnershipModel::<true>, Self)
    }
}
pub struct BodyIdentity;
impl Aspect for BodyIdentity {
    const NAME: &'static str = "BodyIdentity";
    type Row = BodyKeyRow;
}
pub struct BodyKeyRow(u64);
impl fragment::Record<BodyKeyRow> for OwnershipModelBindings {
    fn record() -> impl Shape<BodyKeyRow, Self> {
        fragment::field("BodyKey", |row: &BodyKeyRow| row.0)
            .equality_queryable(true)
            .read_only()
            .unique()
            .build(|value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(BodyKeyRow(value))
            })
    }
}
pub type BodyKey = FieldSelection<OwnershipModel, Body, BodyIdentity, u64>;
