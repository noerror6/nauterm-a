# Android host integration

This is the Flutter Android embedding for `lib/main_android.dart`. The
desktop entry point `lib/main.dart` remains unchanged. Install Flutter, Android
SDK/NDK, Rust, and the `aarch64-linux-android` and `x86_64-linux-android`
Rust targets. Use a JDK 21 installation with the Gradle wrapper.

From the repository root:

```sh
flutter pub get
./scripts/build_android_native.sh debug
flutter build apk --debug --target lib/main_android.dart --target-platform android-x64 --no-pub
adb -s emulator-5554 install -r build/app/outputs/flutter-apk/app-debug.apk
adb -s emulator-5554 shell am start --display 2 -n com.korvect.nauterm/.MainActivity
```

For a physical arm64 device, build Rust with `release`, then build the APK
with `--release --target-platform android-arm64`. The native build script
places ABI-specific `libnauterm_ffi.so` in `android/app/src/main/jniLibs/`;
these generated binaries are gitignored. `android/local.properties` contains
local SDK paths and is gitignored as well. When Flutter's startup version check
blocks on an inaccessible Git remote, use `flutter --no-version-check ...`.

This initial port keeps the existing desktop workspace and requires a wide
display with mouse and keyboard. It uses the Alacritty terminal backend on
Android. Local shell, serial devices, FIDO2 HID, and Mosh (until its separate
library is packaged) are not supported yet. SSH and encrypted host storage
initialize on the emulator; remote SSH/SFTP connections still require testing
against a server you own.

## Credential-key bridge

`MainActivity` exposes MethodChannel `com.korvect.nauterm/credential_keys`:

* Method: `getOrCreateKey`
* Argument: `{"account": "local-database-key"}` or `{"account": "device-key"}`
* Success: 32 raw bytes (`Uint8List` in Dart); errors: `INVALID_ACCOUNT` or
  `KEYSTORE_ERROR` (no key returned on failure).

The returned bytes are stable per account for this installation. Each key is
generated using `SecureRandom` and encrypted with a non-exportable AES-256-GCM
key in AndroidKeyStore. The IV and ciphertext live in private SharedPreferences;
Android backup and device transfer are disabled so ciphertext is not restored
without the wrapping key. A lost/invalidated Keystore key produces an error
rather than silently regenerating the database key.

`lib/main_android.dart` registers both keys through
`nauterm_android_register_credential_key` before opening the Rust SQLCipher
database. Rust copies the keys into process memory and wipes temporary Dart
copies. Android's Keystore wrapping key is never exported. Do not log the returned bytes or
commit app data or APK build artifacts.
