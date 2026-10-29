pub use {worth_proof::TransitionOutcome};
pub use {worth_server::WorthServerBinaryDownload, worth_server::WorthServerBinaryDownloadOutcome, worth_server::WorthServerCompatibilityInspection, worth_server::WorthServerCompatibilityRead, worth_server::WorthServerCompatibilityUpload, worth_server::WorthServerCompatibilityUploadOutcome, worth_server::WorthServerQueryHandoffDenial};

pub fn compat_read_success(
    outcome: worth_server::WorthServerCompatibilityExecutionOutcome<WorthServerCompatibilityRead>,
) -> WorthServerCompatibilityRead {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility read success, got {other:?}"),
    }
}

pub fn compat_read_denied(
    outcome: worth_server::WorthServerCompatibilityExecutionOutcome<WorthServerCompatibilityRead>,
) -> WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(value) => value,
        other => panic!("expected compatibility read denial, got {other:?}"),
    }
}

pub fn compat_inspection_success(
    outcome: worth_server::WorthServerCompatibilityExecutionOutcome<
        WorthServerCompatibilityInspection,
    >,
) -> WorthServerCompatibilityInspection {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility inspection success, got {other:?}"),
    }
}

pub fn compat_upload_success(
    outcome: WorthServerCompatibilityUploadOutcome<WorthServerCompatibilityUpload>,
) -> WorthServerCompatibilityUpload {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility upload success, got {other:?}"),
    }
}

pub fn compat_upload_denied(
    outcome: WorthServerCompatibilityUploadOutcome<WorthServerCompatibilityUpload>,
) -> WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(value) => value,
        other => panic!("expected compatibility upload denial, got {other:?}"),
    }
}

pub fn compat_download_success(
    outcome: WorthServerBinaryDownloadOutcome<WorthServerBinaryDownload>,
) -> WorthServerBinaryDownload {
    match outcome {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected binary download success, got {other:?}"),
    }
}

pub fn compat_download_denied(
    outcome: WorthServerBinaryDownloadOutcome<WorthServerBinaryDownload>,
) -> WorthServerQueryHandoffDenial {
    match outcome {
        worth_proof::TransitionOutcome::Denied(value) => value,
        other => panic!("expected binary download denial, got {other:?}"),
    }
}