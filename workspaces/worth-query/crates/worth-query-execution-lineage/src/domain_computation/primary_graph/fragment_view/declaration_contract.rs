//! These concrete application functions certify inference and schema erasure only.
use super::*;
use worth_query_testkit_decl::fragment_fixture::*;

fn read_kernel(
    view: &mut View<'_, Fixture, Reading<'_>>,
    item: &ObservedEntity<Item>,
    work: &mut ValueDecodeAdmission<'_>,
) -> Result<Measurements, FragmentDenial> {
    let row = view.read(item, Metrics, work)?;
    let _label: Option<Caption> = view.read(item, Label, work)?;
    Ok(row)
}
fn resolve_kernel(
    view: &mut View<'_, Fixture, Reading<'_>>,
    value: u64,
) -> Result<ObservedEntity<Item>, FragmentDenial> {
    let field = record_field(Fixture, Item, Metrics, "components[1]")
        .resolve()
        .unwrap();
    view.resolve(field, value)
}
fn write_kernel(
    view: &mut View<'_, Fixture, Writing<'_>>,
    item: &CandidateEntity<Item>,
    peer: &CandidateEntity<Item>,
    value: &Measurements,
    work: &mut ValueDecodeAdmission<'_>,
) -> Result<(), FragmentDenial> {
    view.write(item, Metrics, value, work)?;
    view.link(item, Next, peer)
}
#[test]
fn application_kernels_compile_with_only_fragment_and_authority_phase() {
    let _ = read_kernel;
    let _ = resolve_kernel;
    let _ = write_kernel;
}
