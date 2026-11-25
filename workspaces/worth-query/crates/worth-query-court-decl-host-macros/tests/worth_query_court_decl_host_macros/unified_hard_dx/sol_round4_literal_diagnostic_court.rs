//! Sol Round 4 diagnostic-aware compile-fail court proving that on a
//! `&'static str`-valued field, an exact `&'static str` constant compiles
//! cleanly while `&Box::new("OPEN")` fails strictly with `E0277` at the
//! exact-literal type-identity trait bound (never deref-coercing and never
//! failing with `E0308` or missing trait items).
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn static_str_query_fixture(where_rhs: &str) -> String {
    format!(
        r#"
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_query::*;
use worth_query_decl::facade::application_schema::{{fragment::*, *}};
use worth_query_decl::facade::typed::ScalarAspectType;
use worth_query_decl::facade::{{
    authentication::*, worth_query_identity_namespace, worth_query_native_scalar_bindings,
    worth_query_structured_value_binding, WorthQuery, WorthQueryRecord, WorthQuerySchema,
}};

const Q_NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("diag.uni.strref.q.v1");
const OPEN_CODE: &'static str = "OPEN";

pub struct StaticStrBinding;
impl ApplicationScalarValueBinding for StaticStrBinding {{
    type Value = &'static str;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "diag.uni.strref.str.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::String;
    fn validate(_: &&'static str) -> Result<(), ApplicationValueValidationDenial> {{ Ok(()) }}
    fn encode(v: &&'static str) -> Result<ApplicationValue, ApplicationValueEncodeDenial> {{
        StringApplicationValueBinding::encode(&(*v).to_owned())
    }}
}}
impl ApplicationReadableScalarValueBinding for StaticStrBinding {{
    fn decode(_: &ApplicationValue) -> Result<&'static str, ApplicationValueDecodeDenial> {{
        Ok("OPEN")
    }}
}}

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(schema, owner = "diag.uni.strref", name = "S", members(F))]
pub struct S;
#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(fragment, bindings = C, members(M, P, Task))]
pub struct F;
worth_query_native_scalar_bindings!(pub C);
impl Scalar<WorthQueryExternalPrincipalIdentity> for C {{
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}}
impl Scalar<WorthQueryPrincipalMappingStatus> for C {{
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}}
impl Scalar<&'static str> for C {{
    type Binding = StaticStrBinding;
}}

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(MA), relations(MT))]
pub struct M;
#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(PA))]
pub struct P;
#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(TaskFacts), reference(S, F))]
pub struct Task;
#[derive(WorthQuerySchema)]
#[worth_query(relation, to = P, from = M, reference(S, F))]
pub struct MT;

#[derive(WorthQueryRecord)]
#[worth_query(bindings = C, aspect = MA, fragment = F, entity = M)]
pub struct MRow {{
    #[worth_query(equality)]
    pub ext: WorthQueryExternalPrincipalIdentity,
    #[worth_query(equality)]
    pub st: WorthQueryPrincipalMappingStatus,
}}
#[derive(WorthQueryRecord)]
#[worth_query(bindings = C, aspect = PA, fragment = F, entity = P)]
pub struct PRow {{
    #[worth_query(equality, read_only)]
    pub pid: u64,
}}
#[derive(WorthQueryRecord)]
#[worth_query(bindings = C, aspect = TaskFacts, fragment = F, entity = Task)]
pub struct TaskRowRec {{
    #[worth_query(equality, read_only)]
    pub k: u64,
    #[worth_query(equality)]
    pub code: &'static str,
}}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskView {{
    pub k: u64,
}}
impl TaskView {{
    pub fn result_shape<Q: ApplicationQueryMarkerIdentity<S>>()
        -> TypedApplicationQueryResultShape<S, Q, Task, Self, TaskViewBinding>
    {{
        ApplicationQueryResultShapeBuilder::new(Task::reference()).build()
    }}
}}
worth_query_structured_value_binding!(pub TaskViewBinding for TaskView {{ identity: "diag.uni.strref.r.v1" }});
worth_query_structured_value_binding!(pub DenB for () {{ identity: "diag.uni.strref.den.v1" }});

pub struct PB;
pub struct Ctx;
impl ApplicationBindingContext<S> for Ctx {{
    type PrincipalBinding = PB;
    type Mapping = M;
    type Principal = P;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type IdempotencyKey = u64;
    type DenialBinding = DenB;
    const IDEMPOTENCY_IDENTITY: &'static str = "diag.uni.strref.idem.v1";
    fn principal_binding()
        -> ApplicationPrincipalBindingRef<S, PB, M, P, u64, U64ApplicationValueBinding>
    {{
        unimplemented!()
    }}
}}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = S,
    context = Ctx,
    namespace = Q_NS,
    result = TaskViewBinding,
    root = Task,
    scope = K,
    fields = TaskFacts,
    limits = 8,
    where_equal(code = {where_rhs}),
)]
pub struct StrRefLiteralQuery {{
    #[worth_query(scope)]
    pub k: u64,
}}

pub fn touch_definition() {{
    let _ = OPEN_CODE;
    let _ = StrRefLiteralQueryQuery::definition();
}}
"#
    )
}

fn locate_worth_query_decl_rlib(deps_dir: &Path) -> PathBuf {
    std::fs::read_dir(deps_dir)
        .expect("read test binary deps dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with("libworth_query_decl-") && name.ends_with(".rlib")
                })
        })
        .max_by_key(|path| {
            std::fs::metadata(path)
                .and_then(|meta| meta.modified())
                .ok()
        })
        .expect("compiled libworth_query_decl-*.rlib exists in deps dir")
}

fn check_fixture_with_rustc(label: &str, source: &str) -> std::process::Output {
    let exe = std::env::current_exe().expect("current_exe");
    let deps_dir = exe.parent().expect("test binary lives in deps dir");
    let decl_rlib = locate_worth_query_decl_rlib(deps_dir);
    let out_dir = deps_dir
        .parent()
        .unwrap_or(deps_dir)
        .join("tmp-sol-r4-diag");
    std::fs::create_dir_all(&out_dir).expect("create diagnostic tmp dir under target");
    let rmeta = out_dir.join(format!("sol_r4_{label}_{}.rmeta", std::process::id()));

    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let mut child = Command::new(rustc)
        .arg("--edition=2024")
        .arg("--crate-type=lib")
        .arg("--emit=metadata")
        .arg("--crate-name")
        .arg(format!("sol_r4_{label}"))
        .arg("-L")
        .arg(format!("dependency={}", deps_dir.display()))
        .arg("--extern")
        .arg(format!("worth_query_decl={}", decl_rlib.display()))
        .arg("-o")
        .arg(&rmeta)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn rustc");
    child
        .stdin
        .take()
        .expect("rustc stdin")
        .write_all(source.as_bytes())
        .expect("write fixture to rustc stdin");
    let output = child.wait_with_output().expect("wait for rustc");
    let _ = std::fs::remove_file(&rmeta);
    output
}

#[test]
fn sol_r4_blocker_1_deref_str_ref_literal_rejects_with_e0277_exact_type_bound() {
    // 1. Positive control: exact `&'static str` constant compiles cleanly (exit 0).
    let positive_src = static_str_query_fixture("OPEN_CODE");
    let positive = check_fixture_with_rustc("positive_control", &positive_src);
    assert!(
        positive.status.success(),
        "exact &'static str control fixture must compile cleanly; stderr:\n{}",
        String::from_utf8_lossy(&positive.stderr)
    );

    // 2. Negative fixture: `&Box::new("OPEN")` fails strictly with E0277 on the
    //    exact literal type-identity trait bound — never E0308 or missing items.
    let negative_src = static_str_query_fixture(r#"&Box::new("OPEN")"#);
    let negative = check_fixture_with_rustc("negative_deref", &negative_src);
    let stderr = String::from_utf8_lossy(&negative.stderr);
    assert!(
        !negative.status.success(),
        "&Box::new(\"OPEN\") must fail compilation on &'static str field"
    );
    assert!(
        stderr.contains("error[E0277]"),
        "expected error[E0277] on &Box::new(\"OPEN\"), got:\n{stderr}"
    );
    assert!(
        stderr.contains("__WorthQueryExactLitType") && stderr.contains("Box<&str>"),
        "expected exact-literal type-identity trait bound diagnostic for Box<&str>, got:\n{stderr}"
    );
    assert!(
        !stderr.contains("error[E0308]")
            && !stderr.contains("error[E0046]")
            && !stderr.contains("error[E0432]"),
        "fixture must fail strictly at E0277 exact-type gate without unrelated errors:\n{stderr}"
    );
}
