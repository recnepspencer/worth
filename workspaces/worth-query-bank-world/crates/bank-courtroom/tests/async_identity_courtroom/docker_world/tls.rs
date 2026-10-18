//! The isolated backend serves a generated certificate; only its named CA crosses processes.
use std::{path::Path, process::Command};
pub(super) fn prepare(directory: &Path, image: &str) -> Result<String, String> {
    let output = Command::new("docker")
        .args(["run", "--rm", "--volume"])
        .arg(format!("{}:/certs", directory.display()))
        .args(["--entrypoint", "python", image, "-c", GENERATE])
        .output()
        .map_err(|error| format!("certificate generation could not start: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "certificate generation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    std::fs::read_to_string(directory.join("root.pem"))
        .map_err(|error| format!("named root could not be read: {error}"))
}
pub(super) fn declare(directory: &Path, blueprint: String) -> Result<String, String> {
    let certificate =
        std::fs::read_to_string(directory.join("server.pem")).map_err(|error| error.to_string())?;
    let key =
        std::fs::read_to_string(directory.join("server.key")).map_err(|error| error.to_string())?;
    let indent = |pem: &str| {
        pem.lines()
            .map(|line| format!("        {line}\n"))
            .collect::<String>()
    };
    Ok(format!("{blueprint}\n  - model: authentik_blueprints.metaapplyblueprint\n    attrs:\n      identifiers:\n        name: Default - Brand\n      required: true\n  - model: authentik_crypto.certificatekeypair\n    state: must_created\n    identifiers:\n      name: WORTH cold TLS\n    attrs:\n      certificate_data: |\n{}      key_data: |\n{}  - model: authentik_brands.brand\n    identifiers:\n      domain: authentik-default\n      default: true\n    attrs:\n      web_certificate: !Find [authentik_crypto.certificatekeypair, [name, WORTH cold TLS]]\n", indent(&certificate), indent(&key)))
}
const GENERATE: &str = r#"
from pathlib import Path
from datetime import datetime, timedelta, timezone
from ipaddress import ip_address
from cryptography import x509
from cryptography.x509.oid import NameOID, ExtendedKeyUsageOID
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import rsa
out=Path('/certs')
now=datetime.now(timezone.utc)
ca_key=rsa.generate_private_key(public_exponent=65537, key_size=2048)
ca_name=x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, 'WORTH isolated court CA')])
ca=(x509.CertificateBuilder().subject_name(ca_name).issuer_name(ca_name)
    .public_key(ca_key.public_key()).serial_number(x509.random_serial_number())
    .not_valid_before(now-timedelta(minutes=5)).not_valid_after(now+timedelta(days=2))
    .add_extension(x509.BasicConstraints(ca=True, path_length=0), critical=True)
    .add_extension(x509.KeyUsage(False,False,False,False,False,True,True,False,False), critical=True)
    .sign(ca_key, hashes.SHA256()))
key=rsa.generate_private_key(public_exponent=65537, key_size=2048)
leaf=(x509.CertificateBuilder().subject_name(x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, 'host.docker.internal')]))
    .issuer_name(ca_name).public_key(key.public_key()).serial_number(x509.random_serial_number())
    .not_valid_before(now-timedelta(minutes=5)).not_valid_after(now+timedelta(days=1))
    .add_extension(x509.BasicConstraints(ca=False, path_length=None), critical=True)
    .add_extension(x509.SubjectAlternativeName([x509.DNSName('host.docker.internal'),x509.DNSName('localhost'),x509.IPAddress(ip_address('127.0.0.1'))]), critical=False)
    .add_extension(x509.ExtendedKeyUsage([ExtendedKeyUsageOID.SERVER_AUTH]), critical=False)
    .add_extension(x509.KeyUsage(True,False,True,False,False,False,False,False,False), critical=True)
    .sign(ca_key, hashes.SHA256()))
(out/'root.pem').write_bytes(ca.public_bytes(serialization.Encoding.PEM))
(out/'server.pem').write_bytes(leaf.public_bytes(serialization.Encoding.PEM)+ca.public_bytes(serialization.Encoding.PEM))
(out/'server.key').write_bytes(key.private_bytes(serialization.Encoding.PEM,serialization.PrivateFormat.PKCS8,serialization.NoEncryption()))
"#;
