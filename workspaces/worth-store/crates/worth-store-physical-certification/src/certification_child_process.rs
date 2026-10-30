use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

pub(crate) fn validated_current_executable(command: &Command) -> Option<[u8; 32]> {
    let current = std::fs::canonicalize(std::env::current_exe().ok()?).ok()?;
    let commanded = std::fs::canonicalize(command.get_program()).ok()?;
    if commanded != current {
        return None;
    }
    let mut file = std::fs::File::open(current).ok()?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).ok()?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Some(digest.finalize().into())
}

pub(crate) fn fresh_challenge(
    domain: &[u8],
    subject: &[u8],
    executable_identity: [u8; 32],
) -> [u8; 32] {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let mut digest = Sha256::new();
    digest.update(domain);
    digest.update(std::process::id().to_be_bytes());
    digest.update(NEXT.fetch_add(1, Ordering::Relaxed).to_be_bytes());
    digest.update(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .to_be_bytes(),
    );
    digest.update(subject);
    digest.update(executable_identity);
    digest.finalize().into()
}

pub(crate) fn publish_new_synced(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    static NEXT_PUBLICATION: AtomicU64 = AtomicU64::new(1);
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "evidence path has no parent",
        )
    })?;
    std::fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(
        ".process-evidence-{}-{}.tmp",
        std::process::id(),
        NEXT_PUBLICATION.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        let _ = std::fs::remove_file(&temporary);
        return Err(error);
    }
    drop(file);
    if let Err(error) = std::fs::hard_link(&temporary, path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(error);
    }
    std::fs::remove_file(temporary)
}

pub(crate) fn encode_hex_32(bytes: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(64);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 15) as usize] as char);
    }
    output
}

pub(crate) fn decode_hex_32(value: &str) -> Option<[u8; 32]> {
    if value.len() != 64 {
        return None;
    }
    let mut output = [0; 32];
    for (index, pair) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        output[index] = (hex_nibble(pair[0])? << 4) | hex_nibble(pair[1])?;
    }
    Some(output)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}
