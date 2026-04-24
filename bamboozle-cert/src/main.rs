use clap::Parser;
use rcgen::{
    BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, KeyPair,
    KeyUsagePurpose, SanType,
};
use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;
use time::{Duration, OffsetDateTime};

/// Generate self-signed TLS certificates for use with Bamboozle.
///
/// Creates a local Certificate Authority (CA) and a leaf certificate signed by
/// that CA.  Mount `cert.pem` and `key.pem` into your Bamboozle container,
/// and optionally install `ca.crt` in your OS trust store so clients accept
/// the certificate without warnings.
#[derive(Parser, Debug)]
#[command(name = "bamboozle-cert", version, about)]
struct Cli {
    /// Subject Alternative Names (hostnames or IPs) for the leaf certificate.
    /// Repeat for multiple values.  Defaults to `localhost 127.0.0.1 ::1`.
    #[arg(long = "san", value_name = "HOST_OR_IP")]
    sans: Vec<String>,

    /// Output directory for generated certificate files.
    #[arg(short, long, default_value = "./certs")]
    out: PathBuf,

    /// Certificate validity in days.
    #[arg(long, default_value_t = 365)]
    days: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let sans = if cli.sans.is_empty() {
        vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "::1".to_string(),
        ]
    } else {
        cli.sans
    };

    fs::create_dir_all(&cli.out)?;

    // ── CA Certificate ───────────────────────────────────────────────────
    let mut ca_params = CertificateParams::new(Vec::<String>::new())?;
    ca_params
        .distinguished_name
        .push(DnType::CommonName, "Bamboozle Local CA");
    ca_params
        .distinguished_name
        .push(DnType::OrganizationName, "Bamboozle");
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    ca_params.not_before = OffsetDateTime::now_utc() - Duration::days(1);
    ca_params.not_after =
        OffsetDateTime::now_utc() + Duration::days(i64::from(cli.days) + 1);

    let ca_key = KeyPair::generate()?;
    let ca_cert = ca_params.self_signed(&ca_key)?;

    // ── Leaf Certificate ─────────────────────────────────────────────────
    let san_types: Vec<SanType> = sans
        .iter()
        .map(|s| {
            if let Ok(ip) = s.parse::<IpAddr>() {
                SanType::IpAddress(ip)
            } else {
                SanType::DnsName(s.clone().try_into().expect("invalid DNS name"))
            }
        })
        .collect();

    let mut leaf_params = CertificateParams::new(Vec::<String>::new())?;
    leaf_params
        .distinguished_name
        .push(DnType::CommonName, "Bamboozle Mock Server");
    leaf_params.subject_alt_names = san_types;
    leaf_params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    leaf_params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyEncipherment,
    ];
    leaf_params.not_before = OffsetDateTime::now_utc() - Duration::days(1);
    leaf_params.not_after =
        OffsetDateTime::now_utc() + Duration::days(i64::from(cli.days));

    let leaf_key = KeyPair::generate()?;
    let leaf_cert = leaf_params.signed_by(&leaf_key, &ca_cert, &ca_key)?;

    // ── Write files ──────────────────────────────────────────────────────
    let ca_path = cli.out.join("ca.crt");
    let cert_path = cli.out.join("cert.pem");
    let key_path = cli.out.join("key.pem");

    fs::write(&ca_path, ca_cert.pem())?;
    fs::write(&cert_path, leaf_cert.pem())?;
    fs::write(&key_path, leaf_key.serialize_pem())?;

    println!("✅ Certificates generated in {}/", cli.out.display());
    println!();
    println!("  {}  — CA certificate (install in your OS trust store)", ca_path.display());
    println!("  {} — leaf certificate (mount into Bamboozle)", cert_path.display());
    println!("  {}  — private key      (mount into Bamboozle)", key_path.display());
    println!();
    println!("SANs: {}", sans.join(", "));
    println!("Valid for: {} days", cli.days);
    println!();
    println!("Quick trust (optional):");
    println!();
    #[cfg(target_os = "windows")]
    println!("  certutil -addstore -user Root {}", ca_path.display());
    #[cfg(target_os = "macos")]
    println!(
        "  sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain {}",
        ca_path.display()
    );
    #[cfg(target_os = "linux")]
    println!(
        "  sudo cp {} /usr/local/share/ca-certificates/ && sudo update-ca-certificates",
        ca_path.display()
    );
    // On any platform, also print the generic instruction
    println!();
    println!("Docker usage:");
    println!();
    println!("  docker run \\");
    println!("    -v ./certs:/certs \\");
    println!("    -e TLS_CERT_FILE=/certs/cert.pem \\");
    println!("    -e TLS_KEY_FILE=/certs/key.pem \\");
    println!("    -p 8080:8080 -p 9090:9090 \\");
    println!("    mattisthegreatest/bamboozle");

    Ok(())
}
