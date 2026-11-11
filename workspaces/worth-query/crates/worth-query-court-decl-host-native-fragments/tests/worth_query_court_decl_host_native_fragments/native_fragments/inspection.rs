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
                                record_field(Fixture, Item, ItemIdentity, "id")
                                    .resolve()
                                    .unwrap(),
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

impl World {
    pub(super) fn inspect_note(&self, id: u64) -> Result<Option<OptionalText>, FragmentDenial> {
        self.application
            .with_host_advancement(|phase| {
                self.invariant
                    .project_operation::<ChangeForm, _>(
                        &phase,
                        |reader| {
                            let mut view = reader.fragment(Fixture).unwrap();
                            let item = view.resolve(
                                record_field(Fixture, Item, ItemIdentity, "id")
                                    .resolve()
                                    .unwrap(),
                                id,
                            )?;
                            work(|work| view.read(&item, Note, work))
                        },
                        SystemAllocation,
                    )
                    .unwrap()
                    .into_output()
            })
            .unwrap()
    }
}

impl World {
    pub(super) fn inspect_required_note(&self, id: u64) -> Result<OptionalText, FragmentDenial> {
        self.application
            .with_host_advancement(|phase| {
                self.invariant
                    .project_operation::<ChangeForm, _>(
                        &phase,
                        |reader| {
                            let mut view = reader.fragment(Fixture).unwrap();
                            let item = view.resolve(
                                record_field(Fixture, Item, ItemIdentity, "id")
                                    .resolve()
                                    .unwrap(),
                                id,
                            )?;
                            work(|work| view.read(&item, RequiredNote, work))
                        },
                        SystemAllocation,
                    )
                    .unwrap()
                    .into_output()
            })
            .unwrap()
    }
}
