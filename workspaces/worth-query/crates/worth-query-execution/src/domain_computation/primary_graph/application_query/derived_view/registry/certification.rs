//! Explicit cold scan of Native-owned branch images for certification only.
use super::super::super::managed_index::certification::{
    ProgramPages, WorthQueryCertificationManagedIndexPages,
    WorthQueryCertificationManagedIndexPagesDenial as Denial,
};
use super::*;
impl ManagedDerivedViewRegistry {
    pub(in crate::domain_computation::primary_graph) fn inspect_source_pages(
        &self,
    ) -> Result<Vec<WorthQueryCertificationManagedIndexPages>, Denial> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut programs: Vec<ProgramPages> = Vec::new();
        let mut failure = None;
        for branch in state.source_branches.values() {
            let Some(cell) = branch.slot.admitted() else {
                continue;
            };
            let image = cell.read_image();
            worth_relational::facade::inspection::inspect_managed_source_material(
                image.payload(),
                &mut |program, payload| {
                    if failure.is_some() {
                        return;
                    }
                    let Some(service) = self.sources.service(program) else {
                        failure = Some(Denial::MissingProgram);
                        return;
                    };
                    let position = programs
                        .iter()
                        .position(|entry| entry.program == *program)
                        .unwrap_or_else(|| {
                            programs.push(ProgramPages {
                                program: program.clone(),
                                roots: 0,
                                pages: Default::default(),
                            });
                            programs.len() - 1
                        });
                    let entry = &mut programs[position];
                    entry.roots += 1;
                    if let Err(cause) = service.inspect_pages(payload, &mut entry.pages) {
                        failure = Some(Denial::Material(cause));
                    }
                },
            );
        }
        if let Some(cause) = failure {
            return Err(cause);
        }
        Ok(programs.into_iter().map(ProgramPages::finish).collect())
    }
}
