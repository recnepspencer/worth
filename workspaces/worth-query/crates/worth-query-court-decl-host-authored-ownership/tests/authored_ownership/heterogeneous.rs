//! Two genuinely different authored kinds share one Native co-anchor inverse.
use super::{lifecycle::*, *};
mod courts;
pub mod query;
worth_query_entity!(pub Body for S: Scenario);
worth_query_aspect!(pub BodyIdentity for S: Scenario, Body; identity = AspectIdentity(0x919B3958), revision = AspectContractRevision(1));
worth_query_field!(pub BodyKey for S: Scenario, Body, BodyIdentity: u64 => U64ApplicationValueBinding, read_only, equality);
pub struct BodyCreated<S>(PhantomData<fn() -> S>);
pub struct BodyRetired<S>(PhantomData<fn() -> S>);
impl<S: Scenario> WorthQueryApplicationOutputRole for BodyCreated<S> {
    type Schema = S;
    type Contract = CreatedOutputs;
    type Entity = Body;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "authored-body-created";
}
impl<S: Scenario> WorthQueryApplicationOutputRole for BodyRetired<S> {
    type Schema = S;
    type Contract = RetiredOutputs;
    type Entity = Body;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "authored-body-retired";
}
pub struct BodyFamily;
impl<S: Scenario> ApplicationAuthoredCorrectionFamily<S> for BodyFamily {
    const IDENTITY: &'static str = "ownership.authored.body-family.v1";
    type Entity = Body;
    fn logical_key() -> ApplicationAuthoredLogicalKey<S, Body> {
        ApplicationAuthoredLogicalKey::from_field(BodyKey::reference())
    }
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
        .entity(Body::reference())
        .aspect(Body::reference(), BodyIdentity::reference())
        .unique_field(Body::reference(), BodyKey::reference())
        .operation_create(CreateOp::reference(), Body::reference())
        .operation_write(CreateOp::reference(), BodyKey::reference())
        .operation_create(RestoreOp::reference(), Body::reference())
        .operation_write(RestoreOp::reference(), BodyKey::reference())
        .operation_read_entity(RetireOp::reference(), Body::reference())
        .operation_read_field(RetireOp::reference(), BodyKey::reference())
        .operation_delete(RetireOp::reference(), Body::reference())
        .application_query(query::definition())
        .application_query_binding::<query::Binding<S>>()
}
pub struct Absent;
pub struct Present<const DECLARED: bool>;
pub trait Profile: Sized + 'static {
    const PRESENT: bool = false;
    fn created_identity<S: Scenario>(
        identity: ApplicationAuthoredCorrection<S>,
    ) -> ApplicationAuthoredCorrection<S> {
        identity
    }
    fn retired_identity<S: Scenario>(
        identity: ApplicationAuthoredCorrection<S>,
    ) -> ApplicationAuthoredCorrection<S> {
        identity
    }
    fn create<S: Scenario>(
        _: &Create,
        _: &WorthQueryApplicationEffectEntity<S, Node>,
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
        _: &WorthQueryApplicationEffectEntity<S, Node>,
        _: &mut CandidateWriter<'_, S, RestoreBinding<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        Ok(())
    }
    fn retained_key<S: Scenario>(
        _: &dyn ApplicationCorrectionPreImage<S>,
    ) -> Result<Option<u64>, ApplicationCorrectionDerivationDenial> {
        Ok(None)
    }
}
impl Profile for Absent {}
impl<const DECLARED: bool> Profile for Present<DECLARED> {
    const PRESENT: bool = true;
    fn created_identity<S: Scenario>(
        identity: ApplicationAuthoredCorrection<S>,
    ) -> ApplicationAuthoredCorrection<S> {
        if DECLARED {
            identity.co_retire_created::<correction::UndoCreate<S>, BodyFamily, BodyCreated<S>, BodyRetired<S>>()
        } else {
            identity
        }
    }
    fn retired_identity<S: Scenario>(
        identity: ApplicationAuthoredCorrection<S>,
    ) -> ApplicationAuthoredCorrection<S> {
        if DECLARED {
            identity.co_restore_retired::<correction::UndoRetire<S>, BodyFamily, BodyRetired<S>, BodyCreated<S>>()
        } else {
            identity
        }
    }
    fn retained_key<S: Scenario>(
        fields: &dyn ApplicationCorrectionPreImage<S>,
    ) -> Result<Option<u64>, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::{InvalidPreImage, MissingPreImage};
        let key = fields
            .field_for_output(
                &<BodyRetired<S> as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
                &DeclaredPreImageLocus::from_field(BodyKey::reference::<S>()),
            )
            .ok_or(MissingPreImage)?;
        U64ApplicationValueBinding::decode(key)
            .map(Some)
            .map_err(|_| InvalidPreImage)
    }
    fn create<S: Scenario>(
        input: &Create,
        scope: &WorthQueryApplicationEffectEntity<S, Node>,
        writer: &mut CandidateWriter<'_, S, CreateBinding<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let body = writer.create_entity_in_context(
            scope,
            Body::reference(),
            WorthQueryApplicationEntityKey::new(format!("body-{}", input.id)).unwrap(),
        )?;
        writer.initialize_field(&body, BodyKey::reference(), 10000 + input.id)?;
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
        scope: &WorthQueryApplicationEffectEntity<S, Node>,
        writer: &mut CandidateWriter<'_, S, RestoreBinding<S>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let body = writer.create_entity_in_context(
            scope,
            Body::reference(),
            WorthQueryApplicationEntityKey::new(format!("restored-body-{}", input.id)).unwrap(),
        )?;
        writer.initialize_field(&body, BodyKey::reference(), input.co_values[0])?;
        writer.create_output::<BodyCreated<S>>(&body)?;
        Ok(())
    }
}
