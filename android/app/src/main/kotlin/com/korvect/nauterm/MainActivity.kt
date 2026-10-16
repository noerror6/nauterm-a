package com.korvect.nauterm

import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

class MainActivity : FlutterActivity() {
    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, "com.korvect.nauterm/credential_keys")
            .setMethodCallHandler { call, result ->
                if (call.method != "getOrCreateKey") {
                    result.notImplemented()
                    return@setMethodCallHandler
                }
                val account = call.argument<String>("account")
                if (account.isNullOrBlank() || account.length > 128) {
                    result.error("INVALID_ACCOUNT", "A nonempty account name is required.", null)
                    return@setMethodCallHandler
                }
                try {
                    result.success(CredentialKeys(applicationContext).getOrCreate(account))
                } catch (_: Exception) {
                    // Never log or return key material or cryptographic implementation details.
                    result.error("KEYSTORE_ERROR", "Unable to access secure key storage.", null)
                }
            }
    }
}
