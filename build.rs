fn main() {
    let ca_paths = [
        "/etc/ssl/certs/ca-certificates.crt", // Debian, Ubuntu, Arch
        "/etc/pki/tls/certs/ca-bundle.crt",   // Fedora, RHEL
        "/etc/ssl/ca-bundle.pem",             // OpenSUSE
        "/etc/ssl/cert.pem",                  // Alpine, macOS
    ];

    let ca_bundle = ca_paths
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .expect("no CA bundle found");

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rustc-env=CA_BUNDLE={ca_bundle}");
}
