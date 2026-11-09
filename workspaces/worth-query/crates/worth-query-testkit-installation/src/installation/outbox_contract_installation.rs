//! Genuine closed outbox-contract installation profiles; no execution runtime.
use std::{num::NonZeroU64, sync::Arc};
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::facade::application_schema::{fragment, *};
use worth_query_installation::facade::{
    InstalledOperationCatalog, InstalledOperationSource, WorthQueryExternalEffectCorrelationFamily,
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

pub struct Schema;
pub struct Entity;
pub struct Facts;
pub struct FactsRow(u64);
pub struct Bindings;
impl fragment::Fragment for Schema {
    const NAME: &'static str = "Schema";
    type Bindings = Bindings;
    type Members = (Entity,);
}
impl fragment::Entity for Entity {
    const NAME: &'static str = "Entity";
    type Aspects = (Facts,);
    type Relations = ();
}
impl fragment::Aspect for Facts {
    const NAME: &'static str = "Facts";
    type Row = FactsRow;
}
impl fragment::BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        use_admission: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        use_admission(work)
    }
}
impl fragment::Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
}
impl fragment::Record<FactsRow> for Bindings {
    fn record() -> impl fragment::Shape<FactsRow, Self> {
        fragment::field("Value", |row: &FactsRow| row.0)
            .equality_queryable(true)
            .build(|value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(FactsRow(value))
            })
    }
}
worth_query_declaration::worth_query_structured_value_binding!(pub Payload for () { identity: "worth.rust.unit" });
impl ApplicationExternalEffectBinding for Payload {
    const PROTOCOL: ApplicationExternalEffectProtocol = ApplicationExternalEffectProtocol::new(
        BoundaryProtocolIdentity::new("outbox.rebinding"),
        BoundaryProtocolVersion::new(1),
    );
    const MAX_EXTERNAL_BYTES: u64 = 64;
    fn external_effect_bytes(_: &()) -> Vec<u8> {
        vec![1, 2]
    }
}
worth_query_declaration::worth_query_effect!(Notice for Schema, payload Payload);
worth_query_declaration::worth_query_operation!(pub Notify for Schema, input Payload);
worth_query_declaration::worth_query_operation_emits!(Notify => [Notice]);
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "outbox-rebinding";
    const NAME: &'static str = "Schema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declaration(Some("source-a"), 2)
    }
}
fn declaration(
    source: Option<&str>,
    accepted: u64,
) -> Result<ApplicationSchemaDeclaration<Schema>, ApplicationSchemaDeclarationDenial> {
    let family = WorthQueryExternalEffectCorrelationFamily::new("outbox-rail").unwrap();
    let operation = match source {
        Some(source) => Notify::reference()
            .definition()
            .external_effect_with_inbound(
                Notice::reference(),
                family,
                ApplicationInboundOccurrenceBinding::<Notice>::new(
                    ApplicationInboundOccurrenceProtocol::new(
                        BoundaryProtocolIdentity::new("outbox.rebinding"),
                        BoundaryProtocolVersion::new(1),
                    ),
                    source,
                    limits(accepted),
                )
                .unwrap(),
            ),
        None => Notify::reference()
            .definition()
            .external_effect(Notice::reference(), family),
    }
    .no_aftermath()
    .finish();
    fragment::declaration_builder(Schema)?
        .effect(Notice::reference())
        .operation(operation)
        .operation_emit(Notify::reference(), Notice::reference())
        .build()
}
fn limits(accepted: u64) -> ApplicationInboundOccurrenceLimits {
    let n = |value| NonZeroU64::new(value).unwrap();
    ApplicationInboundOccurrenceLimits {
        maximum_envelope_bytes: n(256),
        maximum_verifier_work: n(256),
        maximum_payload_bytes: n(64),
        maximum_outstanding_dispatch_provenance: n(3),
        maximum_accepted_occurrences: n(accepted),
        maximum_accepted_bytes: n(512),
        maximum_concurrent_publications: n(1),
        maximum_discovery_work: n(16),
        replay_window_milliseconds: n(60_000),
        maximum_cleanup_work: n(16),
    }
}
pub struct InstalledOutboxContract {
    pub catalog: Arc<InstalledOperationCatalog>,
    pub operation: Arc<dyn InstalledOperationSource>,
}
/// Closed semantic changes used to distinguish original inbound meaning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutboxContractProfile {
    Inbound,
    ChangedInboundSource,
    ChangedInboundLimits,
    NoInbound,
}

pub fn install(profile: OutboxContractProfile) -> InstalledOutboxContract {
    let (source, accepted) = match profile {
        OutboxContractProfile::Inbound => (Some("source-a"), 2),
        OutboxContractProfile::ChangedInboundSource => (Some("source-b"), 2),
        OutboxContractProfile::ChangedInboundLimits => (Some("source-a"), 3),
        OutboxContractProfile::NoInbound => (None, 2),
    };
    let declaration = declaration(source, accepted).unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "outbox-rebinding",
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    let schema = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(declaration)
    .unwrap();
    let operation = schema.installed_operation(Notify::reference()).unwrap();
    let catalog = schema.installed_view().retain_operation_catalog();
    let operation = catalog.operation(operation.operation()).unwrap();
    InstalledOutboxContract { catalog, operation }
}
