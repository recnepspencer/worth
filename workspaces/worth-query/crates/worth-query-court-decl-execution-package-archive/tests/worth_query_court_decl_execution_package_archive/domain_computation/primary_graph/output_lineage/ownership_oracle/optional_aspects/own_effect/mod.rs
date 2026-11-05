//! Real handler writes preserve the original whole-aspect dependency.
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact as Fact;

#[derive(Clone, Copy)]
pub enum SourceWrite {
    None,
    Materialize,
    Clear,
    Unobserved,
    Incomplete,
    Select,
}
worth_query_decl::facade::worth_query_application! {
    pub MaterializingSchema { owner: "worth.query.test.optional-own-materialize", version: (1, 0), contributions: [crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::Members], }
}
worth_query_decl::facade::worth_query_application! {
    pub ClearingSchema { owner: "worth.query.test.optional-own-clear", version: (1, 0), contributions: [crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::Members], }
}
worth_query_decl::facade::worth_query_application! {
    pub RenewingSchema { owner: "worth.query.test.optional-own-renew", version: (1, 0), contributions: [crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::Members], }
}
impl TopologyModelBinding for MaterializingSchema {}
impl TopologyModelBinding for ClearingSchema {}
impl TopologyModelBinding for RenewingSchema {}
impl BirthSchema for RenewingSchema {
    const REQUIRED: bool = false;
    const SOURCE_WRITE: SourceWrite = SourceWrite::Materialize;
    const RENEW_OUTPUT: bool = true;
}
impl BirthSchema for MaterializingSchema {
    const REQUIRED: bool = false;
    const SOURCE_WRITE: SourceWrite = SourceWrite::Materialize;
}
impl BirthSchema for ClearingSchema {
    const REQUIRED: bool = false;
    const SOURCE_WRITE: SourceWrite = SourceWrite::Clear;
}

pub fn source_target<Schema: BirthSchema>(
    reader: &mut DecisionReader<'_, '_, '_, Schema, Binding<Schema, false>>,
    key: &str,
) -> Result<Option<WorthQueryInvariantMutationTarget<Schema, Body>>, HandlerExecutionDenial> {
    if matches!(Schema::SOURCE_WRITE, SourceWrite::None) {
        return Ok(None);
    }
    let source = reader
        .resolve_entity(BodyKey::reference(), key.to_owned())
        .map_err(HandlerExecutionDenial::new)?;
    // Only own-effect work observes every target it writes. The decisive
    // checkpoint/race handler still reads Selection alone.
    if matches!(Schema::SOURCE_WRITE, SourceWrite::Materialize) {
        assert!(reader.field(&source, SourceContent::reference())?.is_none());
    }
    reader.mutation_target(&source).map(Some)
}
pub fn apply<Schema: BirthSchema>(
    writer: &mut CandidateWriter<'_, Schema, Binding<Schema, false>>,
    source: Option<&WorthQueryInvariantMutationTarget<Schema, Body>>,
) -> Result<(), HandlerExecutionDenial> {
    if let Some(source) = source {
        let target = writer
            .projected_entity(source)
            .map_err(HandlerExecutionDenial::new)?;
        match Schema::SOURCE_WRITE {
            SourceWrite::Materialize | SourceWrite::Unobserved => {
                // A legitimate complete struct: Required content accompanies
                // the selected Optional field. Both write targets were read.
                writer
                    .write_field(&target, SourceContent::reference(), "complete".to_owned())
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .write_optional_field(
                        &target,
                        SourceSelection::reference(),
                        Some("selected".to_owned()),
                    )
                    .map_err(HandlerExecutionDenial::new)?;
            }
            SourceWrite::Incomplete | SourceWrite::Select => writer
                .write_optional_field(
                    &target,
                    SourceSelection::reference(),
                    Some("selected".to_owned()),
                )
                .map_err(HandlerExecutionDenial::new)?,
            SourceWrite::Clear => writer
                .write_optional_field(&target, SourceSelection::reference(), None)
                .map_err(HandlerExecutionDenial::new)?,
            SourceWrite::None => unreachable!("no source target was requested"),
        }
    }
    Ok(())
}

#[test]
fn ordinary_handler_own_effect_materialization_moves_aspect_absence() {
    let materializing = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<MaterializingSchema>(false);
    let changed = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::journey::materializing_demand(&materializing);
    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::court::readmit(&materializing, &changed, true);
    assert_eq!(
        changed.producer_identity(),
        <Preserve<MaterializingSchema> as WorthQueryApplicationProducerBinding<
            MaterializingSchema,
        >>::IDENTITY
    );
    // A second real installed journey distinguishes P1 creation, P2 unequal
    // publication, and P3 equal-write preservation; it is not a made-up receipt.
    let renewing = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<RenewingSchema>(false);
    let readers = renewing.application_query_basis_observer();
    let baseline = readers.observe().active();
    let renewed = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::journey::materializing_demand(&renewing);
    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::court::readmit(&renewing, &renewed, true);
    drop(renewed);
    assert_eq!(readers.observe().active(), baseline);
}

#[test]
fn ordinary_handler_clear_of_absent_field_retains_aspect_absence() {
    let clearing = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<ClearingSchema>(false);
    let cleared =
        crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::journey::demand(&clearing, true).expect("clearing an absent Optional field is lawful");
    let receipt = cleared.application_commit_receipt().unwrap();
    let completed = (*(*clearing.runtime_for_test()).primary_provider_for_test())
        .observe_completed_application_for_test(receipt.commit_reference())
        .unwrap();
    assert!(
        completed
            .commit_evidence()
            .source_facts_superseded_by_own_effect()
            .next()
            .is_none(),
        "a clear that leaves the aspect absent did not move the read"
    );
    let aspect = (*(*(*(*clearing.runtime_for_test()).primary_provider_for_test())
        .graph_for_test())
    .layout_for_test())
    .field_locator_for_test("Body", "SourceNotes", "SourceSelection")
    .unwrap()
    .aspect()
    .aspect_key();
    let facts = completed
        .commit_evidence()
        .rebased_source_facts_for_test()
        .unwrap();
    let (entity, revision) = facts
        .iter_for_test()
        .find_map(|fact| match fact {
            Fact::SourceAspectRevision {
                entity_id,
                aspect: found,
                native_revision,
            } if found == aspect => Some((*entity_id, *native_revision)),
            _ => None,
        })
        .expect("scalar clear must retain an aspect dependency, not AbsentField");
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    clearing
        .with_application_advancement(&request, |phase| {
            let execution = phase.execution_request_for_for_test(&(*(*clearing.runtime_for_test()).product_runtime_for_test())).unwrap();
            let selected = clearing.on_branch(clearing.current_world()).select(&phase).unwrap();
            let field = (*(*(*(*clearing.runtime_for_test()).primary_provider_for_test()).graph_for_test()).layout_for_test())
                .field_locator_for_test("Body", "SourceNotes", "SourceSelection").unwrap()
                .field_path().fields()[0].clone();
            (*(*(*clearing.runtime_for_test()).primary_provider_for_test()).graph_for_test()).with_runtime(&execution, |native| {
                let view = native.read_truth()
                    .project_snapshot(selected.application_basis_for_test().snapshot_handle()).unwrap();
                assert_eq!(view.read_entity_aspect_version(&execution, entity, aspect).unwrap(), Some(revision),
                    "exact native after-image revision");
                let absent = view.entity_record_with_projection_scope(
                    &execution, entity,
                    worth_relational::facade::runtime::ProjectionAspectScope::from_requirements([
                        worth_relational::facade::runtime::ProjectionAspectRequirement::fields(
                            aspect.clone(), [field.clone()])
                    ]),
                    |record| Some(matches!(record.observe_aspect_field(aspect, &field),
                        Some(worth_relational::facade::runtime::RelationalProjectedFieldObservation::AspectAbsent))),
                ).unwrap().unwrap();
                assert!(absent, "the genuine native clear leaves the whole aspect absent");
            });
        })
        .unwrap();
}

#[test]
fn required_successor_admission_seals_the_selected_edition_without_retagging_source() {
    let runtime = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::installation::install::<MaterializingSchema>(false);
    crate::domain_computation::primary_graph::output_lineage::ownership_oracle::optional_aspects::journey::required_successor_admission(&runtime);
}
