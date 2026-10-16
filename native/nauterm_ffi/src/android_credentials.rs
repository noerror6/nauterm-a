//! Android credential bridge contract (the Android app must implement the callbacks).
//!
//! Register with `nauterm_android_set_credential_callbacks` before opening the database.
//! `load(account, out32)` returns 0 and writes exactly 32 bytes on success, 1 only
//! when the account has never been stored, or -1 on any Keystore/decryption error.
//! A restored ciphertext with no matching Keystore key is an error, never "absent".
//! `store(account, key32)` must atomically persist the 32-byte key encrypted with a
//! non-exportable AES-256-GCM AndroidKeyStore key and return 0 only after durable
//! persistence; return -1 otherwise. The Java/Kotlin side must keep the wrapping
//! key in AndroidKeyStore, and only the nonce + ciphertext/tag in app-private,
//! no-backup storage (bind the ciphertext to the account via GCM AAD).
//! Never store raw/base64 keys in preferences, files or logs.
//! Implement `load` and `store` as synchronous callbacks callable from arbitrary
//! Rust threads, attaching to the JVM as necessary. Do not regenerate on an
//! invalidated/lost wrapping key: return -1. Reject backup/restore of the encrypted
//! database without a matching Keystore key. Accounts are stable UTF-8 C strings:
//! `local-database-key` and `device-key`. Clear callbacks only after all database
//! users have stopped. Register again on every process launch.
//! The Flutter side must call `nauterm_database_open_path` with its app-private
//! support-directory path after registration. `open_default` is disabled on Android.

use std::collections::HashMap;
use std::ffi::{c_char, CString};
use std::sync::{Mutex, OnceLock};
use zeroize::Zeroizing;

use crate::crypto::{CryptoError, AES_KEY_LENGTH};

type Load = extern "C" fn(*const c_char, *mut u8) -> i32;
type Store = extern "C" fn(*const c_char, *const u8) -> i32;

static CALLBACKS: OnceLock<Mutex<Option<(Load, Store)>>> = OnceLock::new();
static REGISTERED_KEYS: OnceLock<Mutex<HashMap<String, Zeroizing<[u8; AES_KEY_LENGTH]>>>> =
    OnceLock::new();

/// Register an already durably stored key obtained from Android Keystore via
/// the platform channel. No key is written to disk by Rust.
#[no_mangle]
pub unsafe extern "C" fn nauterm_android_register_credential_key(
    account: *const c_char,
    bytes: *const u8,
    length: usize,
) -> bool {
    if account.is_null() || bytes.is_null() || length != AES_KEY_LENGTH {
        return false;
    }
    let Ok(account) = std::ffi::CStr::from_ptr(account).to_str() else {
        return false;
    };
    if !matches!(account, "local-database-key" | "device-key") {
        return false;
    }
    let mut key = Zeroizing::new([0u8; AES_KEY_LENGTH]);
    key.copy_from_slice(std::slice::from_raw_parts(bytes, length));
    let Ok(mut keys) = REGISTERED_KEYS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
    else {
        return false;
    };
    keys.insert(account.to_owned(), key);
    true
}

fn registered_key(account: &str) -> Result<Option<Zeroizing<[u8; AES_KEY_LENGTH]>>, CryptoError> {
    let keys = REGISTERED_KEYS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .map_err(|_| CryptoError::new("Android credential bridge unavailable"))?;
    Ok(keys.get(account).map(|key| Zeroizing::new(**key)))
}

#[no_mangle]
pub extern "C" fn nauterm_android_set_credential_callbacks(
    load: Option<Load>,
    store: Option<Store>,
) -> bool {
    let Ok(mut callbacks) = CALLBACKS.get_or_init(|| Mutex::new(None)).lock() else {
        return false;
    };
    match (load, store) {
        (Some(load), Some(store)) => {
            *callbacks = Some((load, store));
            true
        }
        (None, None) => {
            *callbacks = None;
            true
        }
        _ => false,
    }
}

pub fn load_or_create_key(
    account: &str,
    description: &str,
) -> Result<Zeroizing<[u8; AES_KEY_LENGTH]>, CryptoError> {
    if let Some(key) = registered_key(account)? {
        return Ok(key);
    }
    let callbacks = CALLBACKS.get_or_init(|| Mutex::new(None));
    // Serialize load-create-store so two native threads cannot independently generate
    // different keys for a fresh database. The platform must also serialize its store.
    let callbacks = callbacks
        .lock()
        .map_err(|_| CryptoError::new("Android credential bridge unavailable"))?;
    let (load, store) = callbacks
        .ok_or_else(|| CryptoError::new("Android Keystore credential bridge not registered"))?;
    let account =
        CString::new(account).map_err(|_| CryptoError::new("Invalid credential account"))?;
    let mut key = Zeroizing::new([0u8; AES_KEY_LENGTH]);
    match load(account.as_ptr(), key.as_mut_ptr()) {
        0 => Ok(key),
        1 => {
            // Never silently replace a missing key for an existing SQLCipher DB.
            // Database::open checks for an existing file before calling us.
            getrandom::fill(key.as_mut_slice())
                .map_err(|_| CryptoError::new("Random source unavailable"))?;
            if store(account.as_ptr(), key.as_ptr()) != 0 {
                return Err(CryptoError::new(format!(
                    "Could not persist Android {description} securely"
                )));
            }
            Ok(key)
        }
        _ => Err(CryptoError::new(format!(
            "Android {description} unavailable or invalidated"
        ))),
    }
}

/// Refuse to recreate credentials when the database already exists.
pub fn require_existing_key(account: &str) -> Result<(), CryptoError> {
    if registered_key(account)?.is_some() {
        return Ok(());
    }
    let callbacks = CALLBACKS.get_or_init(|| Mutex::new(None));
    let callbacks = callbacks
        .lock()
        .map_err(|_| CryptoError::new("Android credential bridge unavailable"))?;
    let (load, _) = callbacks
        .ok_or_else(|| CryptoError::new("Android Keystore credential bridge not registered"))?;
    let account =
        CString::new(account).map_err(|_| CryptoError::new("Invalid credential account"))?;
    let mut key = Zeroizing::new([0u8; AES_KEY_LENGTH]);
    if load(account.as_ptr(), key.as_mut_ptr()) == 0 {
        Ok(())
    } else {
        Err(CryptoError::new(
            "Existing Android database credential missing or invalidated",
        ))
    }
}
