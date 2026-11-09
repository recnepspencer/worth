//! Both extension fragments use the same View read/write cores as ordinary aspects.
use super::*;
pub fn extension_views(
    read_first: &mut View<'_, ExtensionFirst, Reading<'_>>,
    read_second: &mut View<'_, ExtensionSecond, Reading<'_>>,
    write_first: &mut View<'_, ExtensionFirst, Writing<'_>>,
    write_second: &mut View<'_, ExtensionSecond, Writing<'_>>,
    observed: &ObservedEntity<ExtensionItem>,
    target: &CandidateEntity<ExtensionItem>,
    work: &mut ValueDecodeAdmission<'_>,
) {
    if let Some(value) = read_first.read(observed, ExtensionFact::<0>, work).unwrap() {
        write_first
            .write(target, ExtensionFact::<0>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first.read(observed, ExtensionFact::<1>, work).unwrap() {
        write_first
            .write(target, ExtensionFact::<1>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first.read(observed, ExtensionFact::<2>, work).unwrap() {
        write_first
            .write(target, ExtensionFact::<2>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first.read(observed, ExtensionFact::<3>, work).unwrap() {
        write_first
            .write(target, ExtensionFact::<3>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first.read(observed, ExtensionFact::<4>, work).unwrap() {
        write_first
            .write(target, ExtensionFact::<4>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first.read(observed, ExtensionFact::<5>, work).unwrap() {
        write_first
            .write(target, ExtensionFact::<5>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first.read(observed, ExtensionFact::<6>, work).unwrap() {
        write_first
            .write(target, ExtensionFact::<6>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first.read(observed, ExtensionFact::<7>, work).unwrap() {
        write_first
            .write(target, ExtensionFact::<7>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first.read(observed, ExtensionFact::<8>, work).unwrap() {
        write_first
            .write(target, ExtensionFact::<8>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first.read(observed, ExtensionFact::<9>, work).unwrap() {
        write_first
            .write(target, ExtensionFact::<9>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first
        .read(observed, ExtensionFact::<10>, work)
        .unwrap()
    {
        write_first
            .write(target, ExtensionFact::<10>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first
        .read(observed, ExtensionFact::<11>, work)
        .unwrap()
    {
        write_first
            .write(target, ExtensionFact::<11>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first
        .read(observed, ExtensionFact::<12>, work)
        .unwrap()
    {
        write_first
            .write(target, ExtensionFact::<12>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first
        .read(observed, ExtensionFact::<13>, work)
        .unwrap()
    {
        write_first
            .write(target, ExtensionFact::<13>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first
        .read(observed, ExtensionFact::<14>, work)
        .unwrap()
    {
        write_first
            .write(target, ExtensionFact::<14>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first
        .read(observed, ExtensionFact::<15>, work)
        .unwrap()
    {
        write_first
            .write(target, ExtensionFact::<15>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first
        .read(observed, ExtensionFact::<16>, work)
        .unwrap()
    {
        write_first
            .write(target, ExtensionFact::<16>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first
        .read(observed, ExtensionFact::<17>, work)
        .unwrap()
    {
        write_first
            .write(target, ExtensionFact::<17>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first
        .read(observed, ExtensionFact::<18>, work)
        .unwrap()
    {
        write_first
            .write(target, ExtensionFact::<18>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_first
        .read(observed, ExtensionFact::<19>, work)
        .unwrap()
    {
        write_first
            .write(target, ExtensionFact::<19>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<20>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<20>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<21>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<21>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<22>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<22>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<23>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<23>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<24>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<24>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<25>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<25>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<26>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<26>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<27>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<27>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<28>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<28>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<29>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<29>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<30>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<30>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<31>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<31>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<32>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<32>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<33>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<33>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<34>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<34>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<35>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<35>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<36>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<36>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<37>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<37>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<38>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<38>, &value, work)
            .unwrap();
    }
    if let Some(value) = read_second
        .read(observed, ExtensionFact::<39>, work)
        .unwrap()
    {
        write_second
            .write(target, ExtensionFact::<39>, &value, work)
            .unwrap();
    }
}
