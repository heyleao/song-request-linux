use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use axum_server::tls_rustls::RustlsConfig;
use rcgen::{generate_simple_self_signed, CertifiedKey};

use crate::config::AppConfig;

pub async fn rustls_config(config: &AppConfig) -> Result<RustlsConfig> {
    let cert_path = cert_path(config);
    let key_path = key_path(config);

    if !cert_path.exists() || !key_path.exists() {
        generate_localhost_cert(&cert_path, &key_path)?;
    }

    RustlsConfig::from_pem_file(&cert_path, &key_path)
        .await
        .context("failed to load local HTTPS certificate")
}

fn generate_localhost_cert(cert_path: &PathBuf, key_path: &PathBuf) -> Result<()> {
    if let Some(parent) = cert_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let CertifiedKey { cert, signing_key } =
        generate_simple_self_signed(vec!["localhost".to_string(), "127.0.0.1".to_string()])?;

    fs::write(cert_path, cert.pem())
        .with_context(|| format!("failed to write {}", cert_path.display()))?;
    fs::write(key_path, signing_key.serialize_pem())
        .with_context(|| format!("failed to write {}", key_path.display()))?;
    restrict_file_permissions(key_path);

    Ok(())
}

fn cert_path(config: &AppConfig) -> PathBuf {
    config.paths.tls_dir.join("localhost-cert.pem")
}

fn key_path(config: &AppConfig) -> PathBuf {
    config.paths.tls_dir.join("localhost-key.pem")
}

#[cfg(unix)]
fn restrict_file_permissions(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_file_permissions(_path: &std::path::Path) {}
