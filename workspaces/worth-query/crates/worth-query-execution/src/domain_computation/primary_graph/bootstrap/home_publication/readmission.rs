//! Admit bootstrap meaning and activation at the actual selected World root.
use super::super::{WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphPublication};
use super::{
    codec, recovery_denial as denial, request, WorthQueryPrimaryGraphInstallationDenial as Denial,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{
    ApplicationHomeBinding, AspectFieldLocator, AspectValue, ContractValidatedAspectValueView,
    InternedString, StructAspectValue,
};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationSchema, WorthQueryProgramSupportRoster,
};
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    history::CommitId,
    identity::{EntityId, VersionId},
    runtime::{RelationalRuntime, VisibilityProjectionView},
};
use worth_runtime_world::facade::ProductBranchObservation;

struct DecodedCompletion {
    binding: ApplicationHomeBinding,
    funding: Option<ExecutionMemoryReservation>,
    terminal: CommitId,
    created: VersionId,
    counts: [usize; 5],
}

pub(in crate::domain_computation::primary_graph) fn readmit<Schema: ApplicationSchema>(
    bootstrap: &WorthQueryPrimaryGraphBootstrap<Schema>,
    phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
    roster: &WorthQueryProgramSupportRoster<Schema>,
    selected: &ApplicationProgramRevision,
    root: &ProductBranchObservation,
) -> Result<WorthQueryPrimaryGraphPublication, Denial> {
    let execution = phase.execution_request();
    bootstrap
        .graph
        .integration_handle()
        .with_runtime(&execution, |native| {
            let basis = root.basis().relational_basis();
            let view = native
                .read_truth()
                .project_observation(&basis.observation())
                .map_err(|cause| {
                    denial(format!("Home completion selected basis denied: {cause:?}"))
                })?;
            let DecodedCompletion {
                binding,
                funding,
                terminal,
                created,
                counts,
            } = decode_selected_row(&view, &bootstrap.graph.layout, &execution)?;
            admit_binding(installed, roster, selected, binding)?;
            admit_receipts(native, basis, terminal, created)?;
            let activation = admit_activation(bootstrap, &view, &execution, selected)?;
            let publication = publication(installed, terminal, counts);
            let cell = bootstrap
                .program_activation
                .as_ref()
                .ok_or_else(|| denial("Home activation cell is missing"))?;
            execution.consult().map_err(request)?;
            cell.publish(activation)
                .map_err(|_| denial("Home activation was already published"))?;
            drop(funding);
            Ok(publication)
        })
}

fn decode_selected_row(
    view: &VisibilityProjectionView,
    graph_layout: &super::super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<DecodedCompletion, Denial> {
    let layout = &graph_layout.bootstrap_publication;
    let rows = view
        .bounded_entities_of_kind(execution, layout.kind, usize::MAX)
        .map_err(super::super::request_read_denial::read)?
        .map_err(|_| denial("Home completion scan exceeded its bound"))?;
    let [row] = rows.records() else {
        return Err(denial(
            "Home root must contain exactly one bootstrap completion record",
        ));
    };
    let state = row
        .authoritative_aspect_state
        .as_ref()
        .ok_or_else(|| denial("Home completion state is missing"))?;
    let aspect = state
        .get(layout.fields[0].aspect().aspect_key())
        .ok_or_else(|| denial("Home completion aspect is missing"))?;
    let ContractValidatedAspectValueView::Struct(fields) = aspect.view() else {
        return Err(denial("Home completion aspect has foreign shape"));
    };
    if numeric_field(fields, &layout.fields[0])? != 1 {
        return Err(denial("Home bootstrap completion format is unsupported"));
    }
    let AspectValue::String(InternedString::Raw(binding)) =
        required_field(fields, &layout.fields[1])?
    else {
        return Err(denial("Home completion binding has foreign shape"));
    };
    let count = |index| {
        usize::try_from(numeric_field(fields, &layout.fields[index])?)
            .map_err(|_| denial("Home completion count exceeds this host"))
    };
    let counts = [count(3)?, count(4)?, count(5)?, count(6)?, count(7)?];
    let terminal = CommitId(numeric_field(fields, &layout.fields[2])?);
    let (binding, funding) = codec::decode(binding, execution)?;
    Ok(DecodedCompletion {
        binding,
        funding,
        terminal,
        created: row.created_at_version,
        counts,
    })
}

fn required_field<'a>(
    fields: &'a StructAspectValue,
    locator: &AspectFieldLocator,
) -> Result<&'a AspectValue, Denial> {
    fields
        .get(
            locator
                .field_path()
                .fields()
                .first()
                .expect("installed completion field"),
        )
        .ok_or_else(|| denial("Home completion required field is missing"))
}
fn numeric_field(fields: &StructAspectValue, locator: &AspectFieldLocator) -> Result<u64, Denial> {
    match required_field(fields, locator)? {
        AspectValue::UInt64(value) => Ok(*value),
        _ => Err(denial("Home completion numeric field has foreign shape")),
    }
}
fn admit_binding<Schema: ApplicationSchema>(
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
    roster: &WorthQueryProgramSupportRoster<Schema>,
    selected: &ApplicationProgramRevision,
    binding: ApplicationHomeBinding,
) -> Result<(), Denial> {
    drop(
        installed
            .admit_home_binding(roster, selected, binding)
            .map_err(|cause| denial(format!("Home completion binding is foreign: {cause:?}")))?,
    );
    Ok(())
}
fn admit_receipts(
    native: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    terminal: CommitId,
    created: VersionId,
) -> Result<(), Denial> {
    let terminal_receipt = native
        .history()
        .immutable_commit_receipt(terminal)
        .ok_or_else(|| denial("Home terminal bootstrap receipt is missing"))?;
    let creation = native
        .history()
        .immutable_commit_receipt_for_version(created)
        .ok_or_else(|| denial("Home completion creation receipt is missing"))?;
    if creation.parents.as_slice() != [terminal]
        || creation.branch_id != *basis.identity().branch_id()
        || terminal_receipt.branch_id != creation.branch_id
    {
        return Err(denial(
            "Home completion does not succeed its actual terminal bootstrap on the selected branch",
        ));
    }
    Ok(())
}
fn admit_activation<Schema: ApplicationSchema>(
    bootstrap: &WorthQueryPrimaryGraphBootstrap<Schema>,
    view: &VisibilityProjectionView,
    execution: &ExecutionRequest<'_, '_>,
    selected: &ApplicationProgramRevision,
) -> Result<EntityId, Denial> {
    super::super::program_activation_recovery::read_activation(
        execution,
        view,
        bootstrap.graph.layout.program_activation(),
        usize::MAX,
        "Home root does not activate its admitted program",
        |rendering| {
            *rendering
                == super::super::super::program_occurrence::program_revision_rendering(selected)
        },
    )
    .map(|(identity, _)| identity)
}
fn publication<Schema: ApplicationSchema>(
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
    terminal: CommitId,
    counts: [usize; 5],
) -> WorthQueryPrimaryGraphPublication {
    let [principal_binding_count, identity_index_count, application_equality_index_count, policy_entity_count, policy_relation_count] =
        counts;
    WorthQueryPrimaryGraphPublication {
        binding_identity: installed.binding_identity().clone(),
        principal_binding_count,
        identity_index_count,
        application_equality_index_count,
        policy_entity_count,
        policy_relation_count,
        bootstrap_commit_id: terminal,
    }
}
