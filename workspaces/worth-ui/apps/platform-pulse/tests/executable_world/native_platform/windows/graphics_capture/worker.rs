//! Bounded DXGI acquisition and cropped readback on the owned capture worker.
use super::{
    clock::{convert_counter, validate_timestamp},
    qpc_100ns, CaptureCrop,
};
use crate::external_observation::{NativeClientPixelCapture, NativeTimedClientPixelCapture};
use std::io;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{SyncSender, TrySendError},
};
use windows_capture::dxgi_duplication_api::{DxgiDuplicationApi, DxgiDuplicationFormat, Error};
use windows_capture::monitor::Monitor;

pub(super) fn capture(
    crop: CaptureCrop,
    process_id: u32,
    sender: SyncSender<NativeTimedClientPixelCapture>,
    stop: &AtomicBool,
) -> Result<(), String> {
    let monitor = Monitor::from_raw_hmonitor(crop.monitor_handle as *mut std::ffi::c_void);
    let provenance = super::adapter_provenance::describe(crop.monitor_handle)
        .unwrap_or_else(|error| format!("adapter provenance unavailable: {error}"));
    println!("native capture adapter provenance: {provenance}");
    let mut duplication = DxgiDuplicationApi::new_options(monitor, &[DxgiDuplicationFormat::Bgra8])
        .map_err(|e| format!("desktop duplication initialization: {e}; {provenance}"))?;
    if [duplication.width(), duplication.height()] != crop.frame_extent
        || duplication.duplication_desc().Rotation.0 != 1
    {
        return Err("desktop duplication requires unchanged, unrotated monitor geometry".into());
    }
    let refresh = duplication.refresh_rate();
    println!(
        "native capture monitor refresh={}/{}Hz; desktop duplication BGRA8 task-strip only",
        refresh.0, refresh.1
    );
    let frequency = winsafe::QueryPerformanceFrequency().map_err(|e| e.to_string())?;
    let mut previous = None;
    while !stop.load(Ordering::Acquire) {
        let mut frame = match duplication.acquire_next_frame(20) {
            Ok(frame) => frame,
            Err(Error::Timeout) => continue,
            Err(error) => return Err(error.to_string()),
        };
        let acquired_qpc_100ns = qpc_100ns().map_err(|e| e.to_string())?;
        let info = frame.frame_info();
        if info.LastPresentTime == 0 {
            continue;
        } // documented pointer-only update
        if info.AccumulatedFrames > 1 || info.ProtectedContentMaskedOut.as_bool() {
            return Err(format!(
                "desktop capture lost frames or protected pixels: accumulated={}, masked={:?}",
                info.AccumulatedFrames, info.ProtectedContentMaskedOut
            ));
        }
        let captured_qpc_100ns = convert_counter(info.LastPresentTime, frequency)
            .ok_or("invalid desktop presentation QPC")?;
        if !validate_timestamp(previous, captured_qpc_100ns, acquired_qpc_100ns) {
            return Err(format!("non-monotonic or future desktop timestamp: previous={previous:?}, captured={captured_qpc_100ns}, callback={acquired_qpc_100ns}"));
        }
        let [left, top, right, bottom] = crop.edges;
        let mut buffer = frame
            .buffer_crop(left, top, right, bottom)
            .map_err(|e| e.to_string())?;
        let stride = buffer.row_pitch() as usize;
        let mut rgba = copy_rgba_rows(buffer.as_raw_buffer(), stride, right - left, bottom - top)
            .map_err(|e| e.to_string())?;
        for pixel in rgba.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }
        let copied_qpc_100ns = qpc_100ns().map_err(|e| e.to_string())?;
        let pixels = NativeClientPixelCapture::new(process_id, right - left, bottom - top, rgba)
            .ok_or("invalid cropped pixel extent")?;
        previous = Some(captured_qpc_100ns);
        publish(
            &sender,
            NativeTimedClientPixelCapture {
                captured_qpc_100ns,
                acquired_qpc_100ns,
                copied_qpc_100ns,
                pixels,
            },
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub(super) fn copy_rgba_rows(
    raw: &[u8],
    stride: usize,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, io::Error> {
    let bytes = (width as usize)
        .checked_mul(4)
        .ok_or_else(|| io::Error::other("row overflow"))?;
    if bytes == 0
        || height == 0
        || stride < bytes
        || raw.len() < stride.saturating_mul(height as usize)
    {
        return Err(io::Error::other("truncated capture row buffer"));
    }
    let mut rgba = Vec::with_capacity(bytes * height as usize);
    for row in raw.chunks_exact(stride).take(height as usize) {
        rgba.extend_from_slice(&row[..bytes]);
    }
    Ok(rgba)
}

pub(super) fn publish(
    sender: &SyncSender<NativeTimedClientPixelCapture>,
    frame: NativeTimedClientPixelCapture,
) -> Result<(), io::Error> {
    sender.try_send(frame).map_err(|error| {
        io::Error::other(match error {
            TrySendError::Full(_) => "desktop capture backlog exhausted; trace lost frames",
            TrySendError::Disconnected(_) => "desktop capture consumer disappeared",
        })
    })
}
