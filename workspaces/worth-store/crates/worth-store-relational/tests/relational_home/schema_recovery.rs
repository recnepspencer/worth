//! A home serves the schema its recovered main head records (D5): the initial
//! schema until a durable schema transition, its target after. Opening with
//! any other declared registry — the other side of the transition, or the
//! same kind contracts under another version label — refuses before service
//! and leaves the home intact. Each branch still recovers under the schema
//! boundary its own commits recorded. A registry with other kind contracts
//! refuses before service.

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

use super::durable_run::{
    apply, assert_holds, open, open_declaring, recorded_schema, served_schema, steps_after,
    steps_before,
};
use super::schema_transition::{basis_of, SOURCE, TARGET};
use super::workload::{self, Script};

const SEED: u64 = 0x5eed_0202;
const COMMITS: usize = 12;
/// The transition is the commit at script step `2 * COMMITS / 3`.
const THROUGH_TRANSITION: usize = 2 * COMMITS / 3 + 1;
/// The kind contracts of the home's schemas under a label it never used.
const OTHER_LABEL: SchemaVersionId = SchemaVersionId(7);

fn branch(name: &str) -> BranchId {
    BranchId(name.to_owned())
}

#[test]
fn a_home_serves_only_the_schema_its_main_head_records() {
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    // Without checkpoints the transition is readmitted from the journal; with
    // one every four commits it is first in the tail, then inside the image.
    for checkpoint_every in [0, 4] {
        let script = Script::generate(SEED, COMMITS, checkpoint_every);
        for prefix in [THROUGH_TRANSITION - 1, THROUGH_TRANSITION, COMMITS] {
            let case =
                format!("seed {SEED:#x} checkpoint every {checkpoint_every}, {prefix} commits");
            let directory = tempfile::tempdir().unwrap();
            let location = directory.path().join("home");
            let home = open(&location, &execution).expect("a new home opens");
            let held = steps_before(&script, prefix);
            assert_eq!(apply(&home, &execution, held), prefix);
            let acknowledged = workload::artifacts_of(home.runtime());
            home.close();

            let served = served_schema(held);
            for declared in [SOURCE, TARGET, OTHER_LABEL] {
                if declared == served {
                    continue;
                }
                let refused = open_declaring(&location, declared, &execution);
                let refusal = refused.err().unwrap_or_else(|| {
                    panic!("{case}: declaring {declared:?} over {served:?} opened")
                });
                assert_eq!(
                    recorded_schema(&refusal),
                    Some(served),
                    "{case}: {refusal:?}"
                );
            }
            let home = open_declaring(&location, served, &execution)
                .unwrap_or_else(|refusal| panic!("{case}: {refusal:?}"));
            assert_holds(&home, &script, &acknowledged);
            assert_eq!(basis_of(home.runtime(), &branch("main")), served, "{case}");
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

            let home = open_declaring(&location, TARGET, &execution)
                .unwrap_or_else(|refusal| panic!("{case}: second open: {refusal:?}"));
            assert_holds(&home, &script, &everything);
            home.close();
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
