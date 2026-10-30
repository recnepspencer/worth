use syn::visit::Visit;
use syn::{ImplItem, Item, Visibility};

use super::WorkspaceSourceInventory;

const SEALED_AUTHORITIES: [(&str, &str, &str); 2] = [
    (
        "crates/worth-ui-runtime/src/runtime/execution/canvas_spatial_lane/plan_contract/canvas_node.rs",
        "WorthUiCanvasRenderResourceRef",
        "new",
    ),
    (
        "crates/worth-ui-runtime/src/runtime/execution/realtime_overlay_lane/renderer_surface/renderer_surface_admission.rs",
        "WorthUiRendererSurfaceAdmission",
        "new",
    ),
];

const FRAME_EXECUTORS: [&str; 2] = [
    "crates/worth-ui-runtime/src/runtime/execution/canvas_spatial_lane/frame_execution/frame_executor.rs",
    "crates/worth-ui-runtime/src/runtime/execution/realtime_overlay_lane/frame_execution/frame_executor.rs",
];

const REGIONAL_PLAN_BUILDERS: [(&str, &[&str]); 3] = [
    (
        "crates/worth-ui-runtime/src/runtime/execution/canvas_spatial_lane/plan_contract/plan_builder.rs",
        &[
            "spatial_slots.for_each",
            "Vec::with_capacity",
            ".collect::<Vec",
        ],
    ),
    (
        "crates/worth-ui-runtime/src/runtime/execution/realtime_overlay_lane/plan_contract/plan_builder.rs",
        &["slots.for_each", "for _ in 0..row_count", ".collect::<Vec"],
    ),
    (
        "crates/worth-ui-runtime/src/runtime/execution/virtualized_data_lane/plan_builder.rs",
        &["slots.for_each", "Vec::with_capacity", ".collect::<Vec"],
    ),
];

const FORBIDDEN_FRAME_DEPENDENCIES: [&str; 9] = [
    "WorthUiArtifact",
    "WorthUiExecutionPlanInput",
    "WorthUiGraph",
    "ComponentDescriptor",
    "WorthUiOrdinaryLanePlan",
    "WorthUiComponentLoweringHook",
    "HashMap",
    "BTreeMap",
    "Vec",
];

pub fn audit_lane_extension_authority(inventory: &WorkspaceSourceInventory) -> Vec<String> {
    let mut violations = Vec::new();
    for (path, authority, constructor) in SEALED_AUTHORITIES {
        let source = inventory
            .source(path)
            .unwrap_or_else(|| panic!("lane authority source `{path}` exists"));
        violations.extend(audit_sealed_authority_source(
            source.text(),
            path,
            authority,
            constructor,
        ));
    }
    for path in FRAME_EXECUTORS {
        let source = inventory
            .source(path)
            .unwrap_or_else(|| panic!("lane executor source `{path}` exists"));
        violations.extend(audit_frame_executor_source(source.text(), path));
    }
    for (path, forbidden_fragments) in REGIONAL_PLAN_BUILDERS {
        let source = inventory
            .source(path)
            .unwrap_or_else(|| panic!("regional plan builder `{path}` exists"));
        for forbidden in forbidden_fragments {
            if compact(source.text()).contains(&compact(forbidden)) {
                violations.push(format!(
                    "{path} rebuilds complete lane storage through `{forbidden}`"
                ));
            }
        }
    }
    let gate_path = "crates/worth-ui-runtime/src/runtime/activation/gate.rs";
    let gate = inventory
        .source(gate_path)
        .expect("activation gate source exists")
        .text();
    let succession = audit_query_succession_calls(gate, gate_path);
    if succession.regional_calls != 1 {
        violations.push(format!(
            "{gate_path} must invoke exactly one regional Query succession; found {}",
            succession.regional_calls
        ));
    }
    if succession.full_calls != 0 {
        violations.push(format!(
            "{gate_path} invokes full Query succession during regional replacement"
        ));
    }
    if succession.installed_reference_calls != 0 {
        violations.push(format!(
            "{gate_path} materializes the complete Query reference catalog during replacement"
        ));
    }
    violations.sort();
    violations
}

fn audit_query_succession_calls(source: &str, path: &str) -> QuerySuccessionCalls {
    let syntax = syn::parse_file(source).unwrap_or_else(|error| panic!("{path} parses: {error}"));
    let mut calls = QuerySuccessionCalls::default();
    calls.visit_file(&syntax);
    calls
}

fn audit_sealed_authority_source(
    source: &str,
    path: &str,
    authority: &str,
    constructor: &str,
) -> Vec<String> {
    let syntax = syn::parse_file(source).unwrap_or_else(|error| panic!("{path} parses: {error}"));
    let mut violations = Vec::new();
    let item = syntax.items.iter().find_map(|item| match item {
        Item::Struct(item) if item.ident == authority => Some(item),
        _ => None,
    });
    match item {
        Some(item) if item.fields.iter().any(|field| !is_private(&field.vis)) => violations.push(
            format!("{path} exposes fields of sealed lane authority `{authority}`"),
        ),
        Some(_) => {}
        None => violations.push(format!("{path} omits sealed lane authority `{authority}`")),
    }
    let constructor_is_public = syntax.items.iter().any(|item| match item {
        Item::Impl(item) => item.items.iter().any(|member| {
            matches!(member, ImplItem::Fn(method) if method.sig.ident == constructor && matches!(method.vis, Visibility::Public(_)))
        }),
        _ => false,
    });
    if constructor_is_public {
        violations.push(format!(
            "{path} publicly exposes `{authority}::{constructor}` without host/plan admission"
        ));
    }
    violations
}

fn audit_frame_executor_source(source: &str, path: &str) -> Vec<String> {
    let syntax = syn::parse_file(source).unwrap_or_else(|error| panic!("{path} parses: {error}"));
    let mut visitor = FrameDependencyVisitor::default();
    visitor.visit_file(&syntax);
    let mut violations = visitor
        .forbidden_paths
        .into_iter()
        .map(|dependency| format!("{path} reaches forbidden frame dependency `{dependency}`"))
        .collect::<Vec<_>>();
    if visitor.collect_call_count > 0 {
        violations.push(format!(
            "{path} materializes collection work on the targeted frame path"
        ));
    }
    violations
}

fn is_private(visibility: &Visibility) -> bool {
    matches!(visibility, Visibility::Inherited)
}

fn compact(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[derive(Default)]
struct FrameDependencyVisitor {
    forbidden_paths: Vec<String>,
    collect_call_count: usize,
}

#[derive(Default)]
struct QuerySuccessionCalls {
    regional_calls: usize,
    full_calls: usize,
    installed_reference_calls: usize,
}

impl<'ast> Visit<'ast> for FrameDependencyVisitor {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if let Some(identifier) = path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            && FORBIDDEN_FRAME_DEPENDENCIES.contains(&identifier.as_str())
        {
            self.forbidden_paths.push(identifier);
        }
        syn::visit::visit_path(self, path);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if call.method == "collect" {
            self.collect_call_count += 1;
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}

impl<'ast> Visit<'ast> for QuerySuccessionCalls {
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        match call.method.to_string().as_str() {
            "prepare_regional_succession" => self.regional_calls += 1,
            "prepare_succession" => self.full_calls += 1,
            "installed_query_references" => self.installed_reference_calls += 1,
            _ => {}
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}
