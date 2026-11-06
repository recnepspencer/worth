//! Optional generated support has its own payload lifecycle, outside the planar ring.
use super::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_host::facade::primary_graph::WorthQueryApplicationEffectEntity;

use worth_query_decl::facade::application_schema::fragment::{self, *};
pub struct SupportRecords;
impl Fragment for SupportRecords {
    const NAME: &'static str = "generated_support";
    type Bindings = SupportBindings;
    type Members = (GeneratedSupport,);
}
pub struct SupportBindings;
impl BindingSet for SupportBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<String> for SupportBindings {
    type Binding = StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl Scalar<u64> for SupportBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
}
pub struct GeneratedSupport;
impl Entity for GeneratedSupport {
    const NAME: &'static str = "GeneratedSupport";
    type Aspects = (SupportPayload,);
    type Relations = ();
}
impl GeneratedSupport {
    pub fn reference<S: TopologySchemaBinding>() -> ApplicationEntityRef<S, Self> {
        entity(SupportRecords, Self)
    }
}
pub struct SupportPayload;
impl Aspect for SupportPayload {
    const NAME: &'static str = "SupportPayload";
    type Row = SupportValueRow;
}
pub struct SupportValueRow {
    key: String,
    value: u64,
}
impl fragment::Record<SupportValueRow> for SupportBindings {
    fn record() -> impl Shape<SupportValueRow, Self> {
        field("SupportKey", |row: &SupportValueRow| row.key.clone())
            .and("SupportValue", |row: &SupportValueRow| row.value)
            .build(|key, value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(SupportValueRow { key, value })
            })
    }
}
pub type SupportKey = FieldSelection<SupportRecords, GeneratedSupport, SupportPayload, String>;
pub type SupportValue = FieldSelection<SupportRecords, GeneratedSupport, SupportPayload, u64>;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TotalSupport;
impl Relation for TotalSupport {
    const NAME: &'static str = "TotalSupport";
    type To = GeneratedSupport;
}
impl TotalSupport {
    pub(super) fn reference<Schema: TopologySchemaBinding>(
    ) -> ApplicationRelationRef<Schema, Self, Body, GeneratedSupport> {
        ApplicationRelationRef::from_schema_identifiers(
            "TotalSupport",
            "Body",
            "GeneratedSupport",
            relation_integrity::<TotalSupport>(),
        )
    }
}
worth_query_decl::facade::worth_query_operation_reads!(TotalRegionOutput =>[GeneratedSupport,TotalSupport]);

worth_query_decl::facade::worth_query_operation_creates!(TotalRegionOutput => [GeneratedSupport]);
worth_query_decl::facade::worth_query_operation_deletes!(TotalRegionOutput => [GeneratedSupport]);
worth_query_decl::facade::worth_query_operation_links!(TotalRegionOutput => [TotalSupport]);
worth_query_decl::facade::worth_query_operation_unlinks!(TotalRegionOutput => [TotalSupport]);

pub(super) fn declare<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = TotalRegionOutput::reference();
    schema
        .operation_read_entity(operation, GeneratedSupport::reference())
        .operation_read_field(operation, SupportKey::at("SupportKey"))
        .operation_read_field(operation, SupportValue::at("SupportValue"))
        .operation_read_relation(operation, TotalSupport::reference())
        .operation_write(operation, SupportKey::at("SupportKey"))
        .operation_write(operation, SupportValue::at("SupportValue"))
        .operation_create(operation, GeneratedSupport::reference())
        .operation_delete(operation, GeneratedSupport::reference())
        .operation_link(operation, TotalSupport::reference())
        .operation_unlink(operation, TotalSupport::reference())
}

pub(super) fn key(total: &str) -> String {
    format!("{total}:extra")
}
pub(super) fn required(value: u64) -> bool {
    !value.is_multiple_of(2)
}

pub(super) fn observe<Schema: TopologySchemaBinding>(
    input: &RegionOutputInput,
    reader: &mut DecisionReader<'_, '_, '_, Schema, PreserveBinding<Schema>>,
) -> Result<bool, HandlerExecutionDenial> {
    let total = reader.resolve_entity(BodyKey::at("BodyKey"), output_key(&input.scope_key))?;
    let members = reader
        .relations_from(TotalSupport::reference(), &total)
        .map_err(HandlerExecutionDenial::new)?;
    for member in &members {
        let entity = member.to();
        reader.entity(entity, GeneratedSupport::reference())?;
        if let Some(key) = reader.field(entity, SupportKey::at("SupportKey"))? {
            let selected = reader.resolve_entity(SupportKey::at("SupportKey"), key)?;
            assert_eq!(selected.entity_id(), entity.entity_id());
        }
        reader.field(entity, SupportValue::at("SupportValue"))?;
    }
    assert!(
        members.len() <= 1,
        "this declared family has one optional semantic member"
    );
    Ok(!members.is_empty())
}

pub(super) fn create_initial<Schema: TopologySchemaBinding>(
    writer: &mut CandidateWriter<'_, Schema, InitialBinding<Schema>>,
    total: &WorthQueryApplicationEffectEntity<Schema, Body>,
    total_key: &str,
    value: u64,
    command: u64,
) -> Result<(), HandlerExecutionDenial> {
    let extra = writer
        .create_entity(
            GeneratedSupport::reference(),
            primary_graph::WorthQueryApplicationEntityKey::new(format!(
                "{}:{command}",
                key(total_key)
            ))
            .map_err(HandlerExecutionDenial::new)?,
        )
        .map_err(HandlerExecutionDenial::new)?;
    record_created::<_, _, binding::CreatedExtra<Schema>>(
        writer, total, &extra, total_key, value, command,
    )
}

fn record_created<Schema, Binding, Family>(
    writer: &mut CandidateWriter<'_, Schema, Binding>,
    total: &WorthQueryApplicationEffectEntity<Schema, Body>,
    extra: &WorthQueryApplicationEffectEntity<Schema, GeneratedSupport>,
    total_key: &str,
    value: u64,
    command: u64,
) -> Result<(), HandlerExecutionDenial>
where
    Schema: TopologySchemaBinding,
    Binding: ApplicationMutationBinding<Schema, Operation = TotalRegionOutput>,
    Family: WorthQueryApplicationOutputRoleFamily<
        Schema = Schema,
        Contract = Binding::Output,
        Entity = GeneratedSupport,
    >,
{
    let key = key(total_key);
    writer
        .initialize_field(extra, SupportKey::at("SupportKey"), key.clone())
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(extra, SupportValue::at("SupportValue"), value)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .link(
            TotalSupport::reference(),
            format!("{key}:{command}:member"),
            total,
            extra,
        )
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .create_member::<Family>("extra", extra)
        .map_err(HandlerExecutionDenial::new)
}

pub(super) fn preserve<Schema: TopologySchemaBinding>(
    writer: &mut CandidateWriter<'_, Schema, PreserveBinding<Schema>>,
    total_key: &str,
    value: u64,
    present: bool,
    command: u64,
) -> Result<(), HandlerExecutionDenial> {
    let total = writer
        .resolve_entity(BodyKey::at("BodyKey"), total_key.to_owned())
        .map_err(HandlerExecutionDenial::new)?;
    if !present {
        if required(value) {
            let extra = writer
                .create_entity_in_context(
                    &total,
                    GeneratedSupport::reference(),
                    primary_graph::WorthQueryApplicationEntityKey::new(format!(
                        "{}:{command}",
                        key(total_key)
                    ))
                    .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            record_created::<_, _, binding::PreservedExtra<Schema>>(
                writer, &total, &extra, total_key, value, command,
            )?;
        }
        return Ok(());
    }
    let extra = writer
        .resolve_entity(SupportKey::at("SupportKey"), key(total_key))
        .map_err(HandlerExecutionDenial::new)?;
    if required(value) {
        writer
            .write_field(&extra, SupportValue::at("SupportValue"), value)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .preserve_member::<binding::PreservedExtra<Schema>>("extra", &extra)
            .map_err(HandlerExecutionDenial::new)
    } else {
        writer
            .unlink(TotalSupport::reference(), &total, &extra)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .delete_entity(GeneratedSupport::reference(), &extra)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .retire_member::<binding::PreservedExtra<Schema>>("extra", &extra)
            .map_err(HandlerExecutionDenial::new)
    }
}
