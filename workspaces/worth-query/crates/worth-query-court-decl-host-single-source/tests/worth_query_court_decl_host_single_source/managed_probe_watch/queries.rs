//! Queries, bindings, and body action for the native/binary probe watch court.
use super::schema::*;
use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::application_query::*;
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_application_query,
    worth_query_mutation_binding, worth_query_operation, worth_query_portable_type,
    worth_query_query_binding, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDenial;

pub struct Params;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntryRow {
    pub name: String,
}
#[derive(Clone, Debug)]
pub struct ReadSource {
    pub directory: String,
}
#[derive(Clone, Debug)]
pub struct FindNative {
    pub directory: String,
    pub prefix: String,
}
#[derive(Clone, Debug)]
pub struct FindBinary {
    pub directory: String,
    pub tag: TagBytes,
}
worth_query_structured_value_binding!(pub ParamsBinding for Params { identity: "managed-probe-watch.params.v1" });
worth_query_structured_value_binding!(pub EntryRowBinding for EntryRow { identity: "managed-probe-watch.row.v1" });
worth_query_structured_value_binding!(pub ReadSourceInput for ReadSource { identity: "managed-probe-watch.source-input.v1" });
worth_query_structured_value_binding!(pub FindNativeInput for FindNative { identity: "managed-probe-watch.find-native-input.v1" });
worth_query_structured_value_binding!(pub FindBinaryInput for FindBinary { identity: "managed-probe-watch.find-binary-input.v1" });

pub struct NameSlot;
pub struct NativeProbeSlot;
pub struct BinaryProbeSlot;
worth_query_portable_type!(NameSlot => "managed-probe-watch.name-slot.v1");

worth_query_application_query!(pub SourceQuery for CatalogSchema,
    identity "managed-probe-watch.source-query.v1", parameters ParamsBinding, result EntryRowBinding,
    scope Directory => "Directory", name "catalog_source");
worth_query_application_query!(pub FindNativeQuery for CatalogSchema,
    identity "managed-probe-watch.find-native-query.v1", parameters ParamsBinding, result EntryRowBinding,
    scope Directory => "Directory", name "catalog_find_native");
worth_query_application_query!(pub FindBinaryQuery for CatalogSchema,
    identity "managed-probe-watch.find-binary-query.v1", parameters ParamsBinding, result EntryRowBinding,
    scope Directory => "Directory", name "catalog_find_binary");

pub fn entry_name<Q>() -> ApplicationQueryResultFieldRef<Q, NameSlot, CatalogSchema, EntryNameField>
{
    ApplicationQueryResultFieldRef::new("name", EntryNameField::at("EntryName"))
}
fn native_probe(
) -> ApplicationQueryParameterRef<FindNativeQuery, NativeProbeSlot, StringApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("prefix")
}
fn binary_probe(
) -> ApplicationQueryParameterRef<FindBinaryQuery, BinaryProbeSlot, TagBytesProbeBinding> {
    ApplicationQueryParameterRef::from_query_identifier("tag")
}

fn base_query<Q: ApplicationQueryMarkerIdentity<CatalogSchema, ResultBinding = EntryRowBinding>>(
    reference: ApplicationQueryReference<CatalogSchema, Q, Params, EntryRow, Directory>,
    lanes: ApplicationQueryLaneEligibility,
) -> ApplicationQueryDefinitionBuilder<CatalogSchema, Q, Params, EntryRow, Directory> {
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Entry::reference())
        .scope(Directory::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<CatalogSchema, Q, Entry, EntryRow, EntryRowBinding>::new(
                Entry::reference(),
            )
            .field(entry_name())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(lanes)
        .public()
}

pub fn source_query(
) -> ApplicationQueryDefinitionBuilder<CatalogSchema, SourceQuery, Params, EntryRow, Directory> {
    base_query(
        SourceQuery::reference(),
        ApplicationQueryLaneEligibility::one_shot(),
    )
    .root_path(
        ApplicationQueryRootPath::from(Directory::reference()).forward(Contains::reference()),
    )
    .order_by(entry_name(), ApplicationQueryOrderingDirection::Ascending)
}
pub fn find_native_query(
) -> ApplicationQueryDefinitionBuilder<CatalogSchema, FindNativeQuery, Params, EntryRow, Directory>
{
    base_query(
        FindNativeQuery::reference(),
        ApplicationQueryLaneEligibility::one_shot().with_live(),
    )
    .managed_index(NativePrefixIndex::reference(), native_probe())
    .order_by(entry_name(), ApplicationQueryOrderingDirection::Ascending)
}
pub fn find_binary_query(
) -> ApplicationQueryDefinitionBuilder<CatalogSchema, FindBinaryQuery, Params, EntryRow, Directory>
{
    base_query(
        FindBinaryQuery::reference(),
        ApplicationQueryLaneEligibility::one_shot().with_live(),
    )
    .managed_index(BinaryTagIndex::reference(), binary_probe())
    .order_by(entry_name(), ApplicationQueryOrderingDirection::Ascending)
}

impl<Q: ApplicationQueryMarkerIdentity<CatalogSchema>>
    worth_query_host::facade::primary_graph::WorthQueryApplicationProjection<CatalogSchema, Q>
    for EntryRow
{
    fn project(
        row: &worth_query_host::facade::primary_graph::WorthQueryApplicationProjectionRow<
            '_,
            CatalogSchema,
            Q,
        >,
    ) -> Result<Self, worth_query_host::facade::primary_graph::WorthQueryApplicationProjectionDenial>
    {
        Ok(Self {
            name: row.field(entry_name())?,
        })
    }
}

fn source_dir(input: ReadSource) -> String {
    input.directory
}
fn native_dir(input: FindNative) -> String {
    input.directory
}
fn binary_dir(input: FindBinary) -> String {
    input.directory
}

worth_query_query_binding!(pub SourceBinding for ReadSource, schema CatalogSchema,
    identity "managed-probe-watch.source-binding.v1", input ReadSourceInput,
    query SourceQuery, parameters ParamsBinding => |_| ApplicationQueryParameterSet::new(),
    result EntryRowBinding, principal PrincipalBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope DirectoryKeyField, field DirectoryKeyField::at("DirectoryKey"), value source_dir, limits results 32);
worth_query_query_binding!(pub FindNativeBinding for FindNative, schema CatalogSchema,
    identity "managed-probe-watch.find-native-binding.v1", input FindNativeInput,
    query FindNativeQuery, parameters ParamsBinding => |i: &FindNative| ApplicationQueryParameterSet::new().bind(native_probe(), i.prefix.clone()).unwrap(),
    result EntryRowBinding, principal PrincipalBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope DirectoryKeyField, field DirectoryKeyField::at("DirectoryKey"), value native_dir, limits results 32);
worth_query_query_binding!(pub FindBinaryBinding for FindBinary, schema CatalogSchema,
    identity "managed-probe-watch.find-binary-binding.v1", input FindBinaryInput,
    query FindBinaryQuery, parameters ParamsBinding => |i: &FindBinary| ApplicationQueryParameterSet::new().bind(binary_probe(), i.tag).unwrap(),
    result EntryRowBinding, principal PrincipalBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope DirectoryKeyField, field DirectoryKeyField::at("DirectoryKey"), value binary_dir, limits results 32);

worth_query_structured_value_binding!(pub BodyResultBinding for () { identity: "managed-probe-watch.body-result.v1" });
worth_query_structured_value_binding!(pub BodyDenialBinding for WorthQueryOperationBodyDenial { identity: "managed-probe-watch.body-denial.v1" });

pub struct CatalogContext;
impl ApplicationBindingContext<CatalogSchema> for CatalogContext {
    type PrincipalBinding = PrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type IdempotencyKey = u64;
    type DenialBinding = BodyDenialBinding;
    const IDEMPOTENCY_IDENTITY: &'static str = "managed-probe-watch.idempotency.v1";
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        CatalogSchema,
        PrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        PrincipalBinding::reference()
    }
}

#[derive(Clone, Debug, WorthValue)]
pub struct RenameEntry {
    pub entry: u64,
    pub name: String,
}
worth_query_structured_value_binding!(pub RenameEntryInput for RenameEntry {
    identity: "managed-probe-watch.rename-input.v1",
    capture_input: true
});
worth_query_operation!(pub RenameEntryOp for CatalogSchema, input RenameEntryInput;
    access { read_fields: [EntryKeyField::at("EntryKey")]; };
    reads [Entry]
);
fn rename_scope(input: &RenameEntry) -> u64 {
    input.entry
}
worth_query_mutation_binding!(pub RenameEntryBinding for RenameEntry, schema CatalogSchema, context CatalogContext,
    identity "managed-probe-watch.rename-binding.v1", input RenameEntryInput,
    operation RenameEntryOp, result BodyResultBinding,
    decision worth_query_host::facade::primary_graph::WorthQueryOperationBodyDecision<CatalogSchema, Entry>,
    handler identity "managed-probe-watch.rename-body.v1", program required,
    outputs NoApplicationMutationOutputs,
    scope EntryKeyField, field EntryKeyField::at("EntryKey"), value rename_scope,
);
pub fn rename_entry_definition(
) -> ApplicationOperationDefinition<CatalogSchema, RenameEntryOp, RenameEntry> {
    let body = RenameEntryOp::reference()
        .body()
        .assign_input(EntryNameField::at("EntryName"), "name")
        .unwrap()
        .finish()
        .unwrap();
    RenameEntryOp::reference()
        .definition()
        .no_external_effect()
        .no_aftermath()
        .finish()
        .with_body(body)
        .unwrap()
}
