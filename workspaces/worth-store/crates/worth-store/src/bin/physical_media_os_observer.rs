use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
use worth_foundational::CanonicalDigestId;

const IDENTITY_RELATIVE_PATH: &str = "namespace/identity";
const IDENTITY_RECORD_LENGTH: usize = 72;
const IDENTITY_MAGIC: &[u8; 8] = b"WSTNSID\0";
const IDENTITY_FIELD_TAG: u16 = 1;

fn main() {
    let mut arguments = std::env::args_os().skip(1);
    let first = arguments.next().unwrap_or_else(|| {
        eprintln!("usage: physical_media_os_observer [--namespace|--tree] <root>");
        std::process::exit(2);
    });
    let (mode, root) = if first == "--namespace" || first == "--tree" {
        let root = arguments.next().unwrap_or_else(|| {
            eprintln!("observer mode requires a root");
            std::process::exit(2);
        });
        (first, root)
    } else {
        ("--identity".into(), first)
    };
    let root = Path::new(&root);
    let result = match mode.to_string_lossy().as_ref() {
        "--namespace" => observe_namespace(root),
        "--tree" => observe_tree(root).map(|manifest| render_tree(&manifest)),
        _ => observe_identity(root).map(|identity| encode_hex(&identity.identity)),
    };
    match result {
        Ok(output) => println!("{output}"),
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    }
}

struct IdentityObservation {
    namespace_version: u16,
    encoding_version: u16,
    identity: [u8; 16],
    record_digest: CanonicalDigestId,
}

fn observe_identity(root: &Path) -> Result<IdentityObservation, &'static str> {
    let bytes = std::fs::read(root.join(IDENTITY_RELATIVE_PATH))
        .map_err(|_| "identity record is not readable")?;
    decode_identity_record(&bytes)
}

fn decode_identity_record(bytes: &[u8]) -> Result<IdentityObservation, &'static str> {
    if bytes.len() != IDENTITY_RECORD_LENGTH {
        return Err("identity record has the wrong length");
    }
    if &bytes[..8] != IDENTITY_MAGIC {
        return Err("identity record has the wrong magic");
    }
    let namespace_version = u16::from_le_bytes([bytes[8], bytes[9]]);
    let encoding_version = u16::from_le_bytes([bytes[10], bytes[11]]);
    if namespace_version != 1 || encoding_version != 1 {
        return Err("identity record version is unsupported");
    }
    if u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize
        != IDENTITY_RECORD_LENGTH
    {
        return Err("identity record declared length is invalid");
    }
    if u16::from_le_bytes([bytes[16], bytes[17]]) != 1 {
        return Err("identity record field count is invalid");
    }
    if bytes[18..20] != [0, 0] {
        return Err("identity record reserved bytes are nonzero");
    }
    if u16::from_le_bytes([bytes[20], bytes[21]]) != IDENTITY_FIELD_TAG {
        return Err("identity record field tag is invalid");
    }
    if u16::from_le_bytes([bytes[22], bytes[23]]) != 16 {
        return Err("identity record field length is invalid");
    }
    if bytes[40..] != Sha256::digest(&bytes[..40])[..] {
        return Err("identity record checksum is invalid");
    }
    let mut identity = [0_u8; 16];
    identity.copy_from_slice(&bytes[24..40]);
    if identity == [0; 16] {
        return Err("identity record contains a zero identity");
    }
    Ok(IdentityObservation {
        namespace_version,
        encoding_version,
        identity,
        record_digest: CanonicalDigestId::new(Sha256::digest(bytes).into()),
    })
}

fn observe_namespace(root: &Path) -> Result<String, &'static str> {
    let identity = observe_identity(root)?;
    let manifest = observe_tree(root)?;
    Ok(format!(
        "namespace_version={}\nencoding_version={}\nidentity={}\nidentity_record_length={}\nidentity_record_sha256={}\n{}",
        identity.namespace_version,
        identity.encoding_version,
        encode_hex(&identity.identity),
        IDENTITY_RECORD_LENGTH,
        encode_hex(identity.record_digest.bytes()),
        render_tree(&manifest),
    ))
}

struct ManifestEntry {
    relative: PathBuf,
    kind: &'static str,
    length: u64,
    digest: Option<String>,
}

fn observe_tree(root: &Path) -> Result<Vec<ManifestEntry>, &'static str> {
    let mut entries = Vec::new();
    observe_directory(root, root, &mut entries)?;
    entries.sort_by(|left, right| left.relative.cmp(&right.relative));
    Ok(entries)
}

fn observe_directory(
    root: &Path,
    directory: &Path,
    entries: &mut Vec<ManifestEntry>,
) -> Result<(), &'static str> {
    for entry in std::fs::read_dir(directory).map_err(|_| "directory is not readable")? {
        let entry = entry.map_err(|_| "directory entry is not readable")?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|_| "entry escaped observed root")?
            .to_owned();
        let metadata = std::fs::symlink_metadata(&path).map_err(|_| "metadata is not readable")?;
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            entries.push(ManifestEntry {
                relative,
                kind: "link",
                length: metadata.len(),
                digest: None,
            });
        } else if file_type.is_dir() {
            entries.push(ManifestEntry {
                relative,
                kind: "directory",
                length: 0,
                digest: None,
            });
            observe_directory(root, &path, entries)?;
        } else if file_type.is_file() {
            let (length, digest) = hash_file(&path)?;
            let digest = observed_file_digest(&path, &relative, length, digest);
            entries.push(ManifestEntry {
                relative,
                kind: "file",
                length,
                digest: Some(digest),
            });
        } else {
            entries.push(ManifestEntry {
                relative,
                kind: "other",
                length: metadata.len(),
                digest: None,
            });
        }
    }
    Ok(())
}

fn observed_file_digest(path: &Path, relative: &Path, length: u64, digest: [u8; 32]) -> String {
    let portable = relative.to_string_lossy().replace('\\', "/");
    if length == IDENTITY_RECORD_LENGTH as u64
        && (portable == IDENTITY_RELATIVE_PATH || is_staged_identity_path(&portable))
        && std::fs::read(path)
            .ok()
            .is_some_and(|bytes| decode_identity_record(&bytes).is_ok())
    {
        return "<valid-identity-record>".into();
    }
    if portable == "namespace/mutation.lock"
        && length <= 512
        && std::fs::read(path)
            .ok()
            .is_some_and(|bytes| valid_mutation_observation(&bytes))
    {
        return "<valid-mutation-observation>".into();
    }
    encode_hex(&digest)
}

fn is_staged_identity_path(path: &str) -> bool {
    let Some(encoded) = path
        .strip_prefix("namespace/identity-")
        .and_then(|value| value.strip_suffix(".staged"))
    else {
        return false;
    };
    encoded.len() == 32
        && encoded
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && encoded.bytes().any(|byte| byte != b'0')
}

fn valid_mutation_observation(bytes: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    let mut lines = text.lines();
    lines.next() == Some("version=1")
        && lines
            .next()
            .and_then(|line| line.strip_prefix("process="))
            .and_then(|value| value.parse::<u32>().ok())
            .is_some()
        && lines
            .next()
            .and_then(|line| line.strip_prefix("runtime="))
            .is_some_and(valid_hex_identity)
        && lines
            .next()
            .and_then(|line| line.strip_prefix("attempt="))
            .is_some_and(valid_hex_identity)
        && lines.next().is_none()
}

fn valid_hex_identity(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && value.bytes().any(|byte| byte != b'0')
}

fn hash_file(path: &Path) -> Result<(u64, [u8; 32]), &'static str> {
    let mut file = std::fs::File::open(path).map_err(|_| "file is not readable")?;
    let mut digest = Sha256::new();
    let mut length = 0_u64;
    let mut buffer = [0_u8; 64 * 1_024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|_| "file read was interrupted")?;
        if read == 0 {
            break;
        }
        length = length
            .checked_add(read as u64)
            .ok_or("observed file length overflowed")?;
        digest.update(&buffer[..read]);
    }
    Ok((length, digest.finalize().into()))
}

fn render_tree(entries: &[ManifestEntry]) -> String {
    entries
        .iter()
        .map(|entry| {
            let path = entry.relative.to_string_lossy().replace('\\', "/");
            let digest = entry
                .digest
                .as_ref()
                .map_or_else(|| "-".to_owned(), Clone::clone);
            format!("entry={}|{}|{}|{}", entry.kind, entry.length, digest, path)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn encode_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(32);
    for &byte in bytes {
        text.push(DIGITS[(byte >> 4) as usize] as char);
        text.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    text
}
