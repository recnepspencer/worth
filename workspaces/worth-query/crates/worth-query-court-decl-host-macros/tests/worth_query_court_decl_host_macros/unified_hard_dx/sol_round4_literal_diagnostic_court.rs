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

struct TempDirGuard(PathBuf);

impl TempDirGuard {
    fn create(path: PathBuf) -> Self {
        std::fs::create_dir_all(&path).expect("create diagnostic tmp dir under target");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn extract_rlib_from_cargo_artifact_line(line: &str) -> Option<PathBuf> {
    if !line.contains("\"reason\":\"compiler-artifact\"")
        || !line.contains("\"name\":\"worth_query_decl\"")
    {
        return None;
    }
    let after = line.split_once("\"filenames\":[")?.1;
    let list_end = after.find(']')?;
    let mut rest = &after[..list_end];
    while let Some(start) = rest.find('"') {
        rest = &rest[start + 1..];
        let mut parsed = String::new();
        let mut chars = rest.chars();
        let mut consumed = 0;
        let mut closed = false;
        while let Some(ch) = chars.next() {
            consumed += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = chars.next() {
                    consumed += escaped.len_utf8();
                    parsed.push(escaped);
                }
            } else if ch == '"' {
                closed = true;
                break;
            } else {
                parsed.push(ch);
            }
        }
        if !closed {
            break;
        }
        rest = &rest[consumed..];
        if parsed.ends_with(".rlib") {
            return Some(PathBuf::from(parsed));
        }
    }
    None
}

/// Resolves the exact `worth-query-decl` `.rlib` artifact and its `deps`
/// directory directly from Cargo's own offline build metadata (never via
/// mtime heuristics across stale `.rlib` files in a warm cache).
fn resolve_worth_query_decl_artifact() -> (PathBuf, PathBuf, PathBuf) {
    let exe = std::env::current_exe().expect("current_exe");
    let deps_dir = exe.parent().expect("test binary lives in deps dir");
    let profile_dir = deps_dir.parent().expect("deps dir lives under profile dir");
    let is_release = profile_dir.file_name().is_some_and(|n| n == "release");
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            profile_dir
                .parent()
                .expect("profile dir lives under target dir")
                .to_path_buf()
        });
    let workspace_manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("court crate lives under workspaces/worth-query/crates/")
        .join("Cargo.toml");

    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("build")
        .arg("--lib")
        .arg("--offline")
        .arg("--manifest-path")
        .arg(&workspace_manifest)
        .arg("-p")
        .arg("worth-query-decl")
        .arg("--target-dir")
        .arg(&target_dir)
        .arg("--message-format=json");
    if is_release {
        cmd.arg("--release");
    }
    let output = cmd.output().expect("resolve worth-query-decl via cargo");
    assert!(
        output.status.success(),
        "cargo build --lib -p worth-query-decl --offline failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let decl_rlib = stdout
        .lines()
        .find_map(extract_rlib_from_cargo_artifact_line)
        .expect("cargo JSON output reported worth_query_decl .rlib artifact");
    let artifact_profile_dir = decl_rlib
        .parent()
        .expect("resolved rlib lives under profile dir")
        .to_path_buf();
    let artifact_deps_dir = artifact_profile_dir.join("deps");
    (decl_rlib, artifact_deps_dir, artifact_profile_dir)
}

fn check_fixture_with_rustc(
    label: &str,
    source: &str,
    decl_rlib: &Path,
    deps_dir: &Path,
    profile_dir: &Path,
) -> std::process::Output {
    let out_dir = TempDirGuard::create(
        profile_dir.join(format!("tmp-sol-r4-diag-{}-{label}", std::process::id())),
    );
    let rmeta = out_dir.path().join(format!("sol_r4_{label}.rmeta"));

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
    child.wait_with_output().expect("wait for rustc")
}

#[test]
fn sol_r4_blocker_1_deref_str_ref_literal_rejects_with_e0277_exact_type_bound() {
    let (decl_rlib, deps_dir, profile_dir) = resolve_worth_query_decl_artifact();

    // 1. Positive control: exact `&'static str` constant compiles cleanly (exit 0).
    let positive_src = static_str_query_fixture("OPEN_CODE");
    let positive = check_fixture_with_rustc(
        "positive_control",
        &positive_src,
        &decl_rlib,
        &deps_dir,
        &profile_dir,
    );
    assert!(
        positive.status.success(),
        "exact &'static str control fixture must compile cleanly; stderr:\n{}",
        String::from_utf8_lossy(&positive.stderr)
    );

    // 2. Negative fixture: `&Box::new("OPEN")` fails strictly with E0277 on the
    //    exact literal type-identity trait bound — never E0308 or missing items.
    let negative_src = static_str_query_fixture(r#"&Box::new("OPEN")"#);
    let negative = check_fixture_with_rustc(
        "negative_deref",
        &negative_src,
        &decl_rlib,
        &deps_dir,
        &profile_dir,
    );
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
