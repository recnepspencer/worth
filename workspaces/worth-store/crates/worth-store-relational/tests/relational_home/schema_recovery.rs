//! A home whose history crosses a schema transition recovers each branch under
//! the schema boundary its own commits recorded. The opening runtime declares
//! a registry with the home's kind contracts; its declared basis does not
//! choose any branch's schema. A registry with other kind contracts refuses
//! before service.

use worth_execution::ExecutionRequest;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};
use worth_store_relational::{
    RelationalStoreHome, RelationalStoreHomeOpenRefusal, RelationalStoreHomeRequest,
};

use super::durable_run::{apply, assert_holds, open, open_declaring, steps_after, steps_before};
use super::schema_transition::{basis_of, SOURCE, TARGET};
use super::workload::{self, Script};

const SEED: u64 = 0x5eed_0202;
const COMMITS: usize = 12;
/// The transition is the commit at script step `2 * COMMITS / 3`.
const THROUGH_TRANSITION: usize = 2 * COMMITS / 3 + 1;

fn branch(name: &str) -> BranchId {
    BranchId(name.to_owned())
}

#[test]
fn each_branch_recovers_under_the_schema_boundary_its_commits_recorded() {
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    // Without checkpoints the transition is readmitted from the journal; with
    // one every four commits it is first in the tail, then inside the image.
    for checkpoint_every in [0, 4] {
        let script = Script::generate(SEED, COMMITS, checkpoint_every);
        for prefix in [THROUGH_TRANSITION, COMMITS] {
            for declared in [SOURCE, TARGET] {
                let case = format!(
                    "seed {SEED:#x} checkpoint every {checkpoint_every}, \
                     {prefix} commits, declaring {declared:?}"
                );
                let directory = tempfile::tempdir().unwrap();
                let location = directory.path().join("home");
                let home = open(&location, &execution).expect("a new home opens");
                assert_eq!(
                    apply(&home, &execution, steps_before(&script, prefix)),
                    prefix
                );
                let acknowledged = workload::artifacts_of(home.runtime());
                home.close();

                let home = open_declaring(&location, declared, &execution)
                    .unwrap_or_else(|refusal| panic!("{case}: {refusal:?}"));
                assert_holds(&home, &script, &acknowledged);
                assert_eq!(basis_of(home.runtime(), &branch("main")), TARGET, "{case}");
                assert_eq!(
                    basis_of(home.runtime(), &branch("feature")),
                    SOURCE,
                    "{case}"
                );
                apply(&home, &execution, steps_after(&script, prefix));
                let everything = workload::artifacts_of(home.runtime());
                assert_eq!(&everything[..prefix], &acknowledged[..], "{case}");
                assert_holds(&home, &script, &everything);
                home.close();

                let home = open_declaring(&location, declared, &execution)
                    .unwrap_or_else(|refusal| panic!("{case}: second open: {refusal:?}"));
                assert_holds(&home, &script, &everything);
                home.close();
            }
        }
    }
}

fn foreign_registry() -> RelationalSchemaRegistry {
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(3),
            kind_name: "other.entity".to_string(),
            schema_id: SchemaId("other".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .expect("the foreign registry")
}

#[test]
fn a_registry_with_other_kind_contracts_refuses_before_service() {
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let script = Script::generate(SEED, COMMITS, 4);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = open(&location, &execution).expect("a new home opens");
    apply(&home, &execution, steps_before(&script, THROUGH_TRANSITION));
    let acknowledged = workload::artifacts_of(home.runtime());
    home.close();

    let foreign = RelationalRuntimeApi::builder()
        .runtime_setup(|runtime| {
            runtime.runtime_name(workload::RUNTIME_NAME);
        })
        .schema_setup(|setup| {
            setup.schema_registry(foreign_registry());
        });
    let refused = RelationalStoreHome::open(
        RelationalStoreHomeRequest::new(&location, foreign),
        &execution,
    );
    assert!(
        matches!(refused, Err(RelationalStoreHomeOpenRefusal::Readmission(_))),
        "{:?}",
        refused.map(|_| ())
    );
    let home = open(&location, &execution).expect("the refusal left the home intact");
    assert_holds(&home, &script, &acknowledged);
    home.close();
}
