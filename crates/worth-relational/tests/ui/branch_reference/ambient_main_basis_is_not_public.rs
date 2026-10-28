use worth_relational::facade::{
    runtime::RelationalRuntimeApi,
    schema::RelationalSchemaRegistry,
};

fn main() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .build().expect("valid native runtime configuration");

    let _ = runtime.admit_main_branch_basis();
}
