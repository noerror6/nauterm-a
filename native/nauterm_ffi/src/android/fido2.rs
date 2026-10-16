use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fido2Device {
    pub id: String,
    pub name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub has_pin: bool,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fido2GenerateRequest {
    pub device_id: String,
    pub label: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fido2GeneratedKey {
    pub private_key: String,
    pub public_key: String,
    pub key_type: String,
    pub application: String,
}
#[derive(Clone, Copy, Debug)]
pub enum Fido2SshAlgorithm {
    Ecdsa,
    Ed25519,
}

pub fn list_devices() -> Vec<Fido2Device> {
    Vec::new()
}
pub fn generate_key(_: Fido2GenerateRequest) -> Result<Fido2GeneratedKey, String> {
    Err("FIDO2 HID is unavailable on Android".into())
}
pub fn verify_pin(_: String, _: String) -> Result<(), String> {
    Err("FIDO2 HID is unavailable on Android".into())
}
pub fn sign_ssh(
    _: &str,
    _: &[u8],
    _: Fido2SshAlgorithm,
    _: &[u8],
    _: Option<&str>,
    _: bool,
) -> Result<Vec<u8>, String> {
    Err("FIDO2 HID SSH signing is unavailable on Android".into())
}
