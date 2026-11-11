//! Commit to every projection field and route without running a Source provider.
use super::*;
use sha2::{Digest, Sha256};
use std::io::{self, Write};
use worth_execution::{ExecutionRequest, WorkCeilingDenial};

pub(crate) enum SourceProgramCommitmentDenial {
    Request(WorkCeilingDenial),
    Encoding,
}

impl ManagedIndexSourceProgram {
    pub(crate) fn catalog_commitment(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<[u8; 32], SourceProgramCommitmentDenial> {
        let mut writer = SourceCatalogCommitment {
            request: *request,
            hash: Sha256::new(),
            stopped: None,
        };
        writer.hash.update(b"worth.native.source.catalog.v1\0");
        writer.emit(self.0.path.parts())?;
        writer.emit(&self.fields().len())?;
        for field in self.fields() {
            writer.locator(&field.locator)?;
        }
        writer.emit(&self.projection_routes().len())?;
        for route in self.projection_routes() {
            writer.emit(&(route.name(), route.parent_route(), route.path.parts()))?;
            writer.emit(&route.fields().len())?;
            for field in route.fields() {
                writer.locator(&field.locator)?;
            }
        }
        Ok(writer.hash.finalize().into())
    }
}

struct SourceCatalogCommitment<'request, 'authority> {
    request: ExecutionRequest<'request, 'authority>,
    hash: Sha256,
    stopped: Option<WorkCeilingDenial>,
}

impl SourceCatalogCommitment<'_, '_> {
    fn locator(
        &mut self,
        value: &worth_foundational::facade::AspectFieldLocator,
    ) -> Result<(), SourceProgramCommitmentDenial> {
        use worth_foundational::facade::LocatorAuthority;
        let authority = match value.aspect().authority() {
            LocatorAuthority::Authoritative => 0u8,
            LocatorAuthority::Derived => 1,
            LocatorAuthority::Projected => 2,
            LocatorAuthority::SupportOnly => 3,
            LocatorAuthority::Planned => 4,
            LocatorAuthority::ReceiptBearing => 5,
        };
        self.emit(&(authority, value.aspect().aspect_key().as_str()))?;
        self.emit(&value.field_path().fields().len())?;
        for field in value.field_path().fields() {
            self.emit(&field.as_str())?;
        }
        Ok(())
    }

    fn emit(&mut self, value: &impl serde::Serialize) -> Result<(), SourceProgramCommitmentDenial> {
        serde_json::to_writer(&mut *self, value).map_err(|_| {
            self.stopped.take().map_or(
                SourceProgramCommitmentDenial::Encoding,
                SourceProgramCommitmentDenial::Request,
            )
        })?;
        // JSON escapes NUL, so this separates complete values unambiguously.
        self.hash.update([0]);
        Ok(())
    }
}

impl Write for SourceCatalogCommitment<'_, '_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let checked = self
            .request
            .run_native_with_checkpoint(|context| {
                if context.has_accounting() {
                    context.checkpoint(bytes.len() as u64)
                } else {
                    context.poll()
                }
            })
            .and_then(|(answer, _)| answer.map_err(WorkCeilingDenial::Stopped));
        match checked {
            Ok(()) => {
                self.hash.update(bytes);
                Ok(bytes.len())
            }
            Err(cause) => {
                self.stopped = Some(cause);
                Err(io::Error::other("Source catalog commitment stopped"))
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
