use super::*;

impl World {
    pub(super) fn inspect(&self, id: u64) -> Result<FormValue, FragmentDenial> {
        self.application
            .with_host_advancement(|phase| {
                self.invariant
                    .project_operation::<ChangeForm, _>(
                        &phase,
                        |reader| {
                            let mut view = reader.fragment(Fixture).unwrap();
                            let item = view.resolve(
                                record_field(Fixture, Item, ItemIdentity, "id").unwrap(),
                                id,
                            )?;
                            work(|work| view.read(&item, Form, work))
                        },
                        SystemAllocation,
                    )
                    .unwrap()
                    .into_output()
            })
            .unwrap()
    }
}
