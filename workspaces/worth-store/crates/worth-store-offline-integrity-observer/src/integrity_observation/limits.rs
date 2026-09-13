#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OfflineIntegrityObservationLimits {
    maximum_entries: u64,
    maximum_bytes: u64,
    maximum_open_files: u32,
    maximum_depth: u32,
    maximum_symlinks: u64,
    maximum_elapsed_milliseconds: u64,
    maximum_report_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineIntegrityObservationLimitsDenial {
    ZeroEntries,
    ZeroBytes,
    ZeroOpenFiles,
    ZeroDepth,
    ZeroElapsedMilliseconds,
    ZeroReportBytes,
}

impl OfflineIntegrityObservationLimits {
    pub const fn new(
        maximum_entries: u64,
        maximum_bytes: u64,
        maximum_open_files: u32,
        maximum_depth: u32,
        maximum_symlinks: u64,
        maximum_elapsed_milliseconds: u64,
        maximum_report_bytes: u64,
    ) -> Result<Self, OfflineIntegrityObservationLimitsDenial> {
        if maximum_entries == 0 {
            return Err(OfflineIntegrityObservationLimitsDenial::ZeroEntries);
        }
        if maximum_bytes == 0 {
            return Err(OfflineIntegrityObservationLimitsDenial::ZeroBytes);
        }
        if maximum_open_files == 0 {
            return Err(OfflineIntegrityObservationLimitsDenial::ZeroOpenFiles);
        }
        if maximum_depth == 0 {
            return Err(OfflineIntegrityObservationLimitsDenial::ZeroDepth);
        }
        if maximum_elapsed_milliseconds == 0 {
            return Err(OfflineIntegrityObservationLimitsDenial::ZeroElapsedMilliseconds);
        }
        if maximum_report_bytes == 0 {
            return Err(OfflineIntegrityObservationLimitsDenial::ZeroReportBytes);
        }
        Ok(Self {
            maximum_entries,
            maximum_bytes,
            maximum_open_files,
            maximum_depth,
            maximum_symlinks,
            maximum_elapsed_milliseconds,
            maximum_report_bytes,
        })
    }

    pub const fn maximum_entries(self) -> u64 {
        self.maximum_entries
    }

    pub const fn maximum_bytes(self) -> u64 {
        self.maximum_bytes
    }

    pub const fn maximum_open_files(self) -> u32 {
        self.maximum_open_files
    }

    pub const fn maximum_depth(self) -> u32 {
        self.maximum_depth
    }

    pub const fn maximum_symlinks(self) -> u64 {
        self.maximum_symlinks
    }

    pub const fn maximum_elapsed_milliseconds(self) -> u64 {
        self.maximum_elapsed_milliseconds
    }

    pub const fn maximum_report_bytes(self) -> u64 {
        self.maximum_report_bytes
    }
}
