//! End-to-end Store M4.4 courts verifying strict `KnownProjectedHeapBytes`
//! accounting on `#[derive(WorthQueryRow)]` custom scalar fields:
//! 1. A `#[derive(WorthQueryRow)]` containing an un-accounted custom scalar type
//!    (missing `KnownProjectedHeapBytes`) fails to compile (`E0277`) with ZERO
//!    silent zero-byte fallback.
//! 2. A `#[derive(WorthQueryRow)]` containing a custom heap-backed scalar type
//!    implementing `KnownProjectedHeapBytes` accurately trips `streaming_window_bytes`.

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use worth_query_decl::facade::authentication::*;
use worth_query_host::facade::admission::authenticated_principal::*;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::historical_cursor::{
    CancellationSource, ExecutionRequest, HistoricalBasisRetentionKind, HistoricalReadDenial,
    ResidencyBudget, SchemaId, SchemaVersionId, SchemaVersionIdentity, SerialMemoryBudget,
    SerialRequest,
};
use worth_query_host::facade::{primary_graph::*, WorthQueryApplicationHome, WorthQueryHomeConfig};

use super::courts::temp_home_path;
use super::retained_basis_cursor_scalar_footprint_schema::*;

#[test]
fn retained_basis_cursor_derived_row_custom_heap_scalar_trips_streaming_window_bytes() {
    let path = temp_home_path("retained-basis-custom-scalar-heap");
    let clock = WorthQueryAdmissionClock::system();
    let build = ApplicationBuildIdentity::parse("7c7c7c7c-7c7c-7c7c-7c7c-7c7c7c7c7c7c").unwrap();
    let user_id = WorthQueryExternalPrincipalIdentity::new(ScalarBudgetIssuer, "u1").unwrap();
    let user_seed = user_id.clone();

    let home = WorthQueryApplicationHome::<ScalarSchema, ScalarProgram>::open_directory(
        &path,
        WorthQueryHomeConfig::new(build, (ScalarRuntime::new(),))
            .with_clock(clock.clone())
            .with_initial_state(move |_, graph, installed| {
                let pb = installed
                    .principal_binding(ScalarPrincipal::reference())
                    .unwrap();
                graph.bind_principal(
                    &pb,
                    WorthQueryApplicationPrincipalKey::new("u1").unwrap(),
                    1_u64,
                    user_seed,
                    WorthQueryPrincipalMappingStatus::Enabled,
                )?;
                let bkey = || WorthQueryApplicationEntityKey::new("box-1".to_owned()).unwrap();
                let ikey = || WorthQueryApplicationEntityKey::new("item-1".to_owned()).unwrap();
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(SBox::reference(), bkey())
                        .field(SBoxKey::at("SBoxKey"), 1_u64),
                )?;
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(SItem::reference(), ikey())
                        .field(SItemKey::at("SItemKey"), 10_u64)
                        .field(SItemTag::at("SItemTag"), ExpandedTagScalar("tiny".into()))
                        .field(
                            SItemBlob::at("SItemBlob"),
                            worth_foundational::facade::AspectValue::InlineBytes(
                                worth_foundational::facade::InlineBytes::try_new(vec![0x42u8])
                                    .unwrap(),
                            ),
                        ),
                )?;
                graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                    SContains::reference(),
                    "c-1".to_owned(),
                    bkey(),
                    ikey(),
                ))?;
                Ok(())
            }),
    )
    .expect("open custom scalar home");

    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        clock.clone(),
    );
    let local = WorthQueryLocalProcessPrincipal::new(
        user_id,
        ScalarBudgetAudience,
        ScalarBudgetMethod,
        Duration::from_secs(600),
    )
    .unwrap();
    let me = admit_authentication_adapter(
        home.primary_graph().installed_schema(),
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::of(ScalarBudgetAudience),
            WorthQueryAuthenticationMethod::of(ScalarBudgetMethod),
            clock,
        ),
        local,
    )
    .unwrap()
    .authenticate_local(&scope)
    .unwrap();
    let session = home.session(&me);
    let schema = SchemaVersionIdentity::new(
        SchemaId("custom.scalar.schema".into()),
        SchemaVersionId(1),
        [0x7C; 32],
    )
    .unwrap();
    let budget = ResidencyBudget {
        head_working_set_bytes: 128 * 1024,
        historical_working_set_bytes: 64 * 1024,
        max_in_flight_fill_bytes: 32 * 1024,
        streaming_window_bytes: 1024,
    };
    let receipt = home
        .retain_historical_basis(
            &session,
            "custom-scalar",
            HistoricalBasisRetentionKind::PinLease,
            schema.clone(),
            budget,
        )
        .unwrap();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let exec = ExecutionRequest::serial(&serial);
    let mut cur = home
        .open_historical_query_cursor(
            &session,
            &exec,
            receipt.commit_id,
            receipt.root_semantic_commitment,
            ReadCustomScalarItems { box_key: 1 },
            &schema,
            budget,
        )
        .unwrap();
    let err = cur.next_page(&exec, 2).unwrap_err();
    assert!(
        matches!(
            err,
            HistoricalReadDenial::MemoryExhausted {
                requested_bytes,
                allowance_bytes: 1024
            } if requested_bytes >= 4096
        ),
        "derived row with custom heap-backed scalar must trip streaming_window_bytes (>= 4096 B), got {err:?}"
    );
    drop(cur);

    // Direct + end-to-end check: AspectValue::InlineBytes backed by a 1-byte
    // payload inside an 8192-byte-capacity Vec MUST report retained capacity
    // and trip streaming_window_bytes (1024 B).
    let mut over_vec = Vec::with_capacity(8192);
    over_vec.push(0x42u8);
    let over_val = worth_foundational::facade::AspectValue::InlineBytes(
        worth_foundational::facade::InlineBytes::try_new(over_vec).unwrap(),
    );
    assert!(
        over_val.known_heap_bytes() >= 8192,
        "AspectValue::InlineBytes must count retained Vec capacity, got {}",
        over_val.known_heap_bytes()
    );
    let mut blob_cur = home
        .open_historical_query_cursor(
            &session,
            &exec,
            receipt.commit_id,
            receipt.root_semantic_commitment,
            ReadOverAllocatedBytesItems { box_key: 1 },
            &schema,
            budget,
        )
        .unwrap();
    let blob_err = blob_cur.next_page(&exec, 2).unwrap_err();
    assert!(
        matches!(
            blob_err,
            HistoricalReadDenial::MemoryExhausted {
                requested_bytes,
                allowance_bytes: 1024
            } if requested_bytes >= 8192
        ),
        "1-byte AspectValue::InlineBytes with 8192B Vec capacity must trip streaming_window_bytes, got {blob_err:?}"
    );

    drop((blob_cur, session, me));
    home.close_clean().unwrap();
    let _ = std::fs::remove_dir_all(path);
}

fn find_rlib(deps_dir: &std::path::Path, crate_name: &str) -> PathBuf {
    let prefix = format!("lib{crate_name}-");
    let mut rlibs: Vec<PathBuf> = std::fs::read_dir(deps_dir)
        .expect("read deps dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(&prefix) && n.ends_with(".rlib"))
        })
        .collect();
    rlibs.sort_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok());
    rlibs
        .pop()
        .unwrap_or_else(|| panic!("compiled {prefix}*.rlib"))
}

#[test]
fn retained_basis_cursor_derived_row_rejects_unaccounted_custom_scalar_at_compile_time() {
    let exe = std::env::current_exe().expect("test exe");
    let deps_dir = exe.parent().expect("deps dir");
    let decl_rlib = find_rlib(deps_dir, "worth_query_decl");
    let host_rlib = find_rlib(deps_dir, "worth_query_host");
    let found_rlib = find_rlib(deps_dir, "worth_foundational");

    let tmp = temp_home_path("retained-basis-unaccounted-scalar-probe");
    std::fs::create_dir_all(&tmp).unwrap();
    let src = tmp.join("unaccounted_derived_scalar.rs");
    let out = tmp.join("out");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(
        &src,
        r#"
use worth_foundational::facade::{AspectValue, InternedString, ScalarAspectType};
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_schema::fragment::{self, *};
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{worth_query_identity_namespace, WorthQueryRow};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnaccountedHeapScalar(pub String);
pub struct UnaccountedBinding;
impl ApplicationScalarValueBinding for UnaccountedBinding {
    type Value = UnaccountedHeapScalar;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "probe.unaccounted-scalar.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> { Ok(()) }
    fn encode(v: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Ok(AspectValue::String(InternedString::Raw(v.0.clone())))
    }
}
impl ApplicationReadableScalarValueBinding for UnaccountedBinding {
    fn decode(v: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        match v {
            AspectValue::String(InternedString::Raw(s)) => Ok(UnaccountedHeapScalar(s.clone())),
            _ => Err(ApplicationValueDecodeDenial::CodecRejected {
                binding_identity:
                    worth_query_decl::facade::portable_identity::WorthQueryPortableTypeIdentity::declared(
                        Self::IDENTITY_NAME,
                    ),
            }),
        }
    }
}
#[derive(Clone, Copy)] pub struct S;
impl Fragment for S { const NAME: &'static str = "s"; type Bindings = NoBindings; type Members = (Frag,); }
impl ApplicationSchema for S {
    const OWNER: &'static str = "probe";
    const NAME: &'static str = "s";
    fn declaration() -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> { fragment::declare(Self) }
}
#[derive(Clone, Copy)] pub struct Frag;
impl Fragment for Frag { const NAME: &'static str = "f"; type Bindings = Bind; type Members = (E,); }
pub struct Bind;
impl BindingSet for Bind {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(w: &mut Self::Work<'_>, run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T) -> T { run(w) }
}
impl Scalar<UnaccountedHeapScalar> for Bind { type Binding = UnaccountedBinding; }
#[derive(Clone, Copy)] pub struct E;
impl Entity for E { const NAME: &'static str = "E"; type Aspects = (A,); type Relations = (); }
pub struct A;
pub struct ARow(pub UnaccountedHeapScalar);
impl Aspect for A { const NAME: &'static str = "A"; type Row = ARow; }
impl Record<ARow> for Bind {
    fn record() -> impl Shape<ARow, Self> {
        field("F", |r: &ARow| r.0.clone()).build(|a, _: &mut ValueDecodeAdmission<'_>| Ok::<_, std::convert::Infallible>(ARow(a)))
    }
}
pub type F = FieldSelection<Frag, E, A, UnaccountedHeapScalar>;
const NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("probe.unaccounted.v1");

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = S, root = E, namespace = NS)]
pub struct UnaccountedDerivedRow {
    #[worth_query(field = F)]
    pub hidden: UnaccountedHeapScalar,
}
"#,
    )
    .unwrap();

    let output = Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-type=lib",
            "--emit=metadata",
            "--out-dir",
        ])
        .arg(&out)
        .args(["-L", &format!("dependency={}", deps_dir.display())])
        .args([
            "--extern",
            &format!("worth_query_decl={}", decl_rlib.display()),
        ])
        .args([
            "--extern",
            &format!("worth_query_host={}", host_rlib.display()),
        ])
        .args([
            "--extern",
            &format!("worth_foundational={}", found_rlib.display()),
        ])
        .arg(&src)
        .output()
        .expect("invoke rustc");
    let _ = std::fs::remove_dir_all(&tmp);

    assert!(
        !output.status.success(),
        "#[derive(WorthQueryRow)] with un-accounted custom scalar MUST fail to compile!"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("KnownProjectedHeapBytes") && stderr.contains("E0277"),
        "expected E0277 KnownProjectedHeapBytes failure, got:\n{stderr}"
    );
}
