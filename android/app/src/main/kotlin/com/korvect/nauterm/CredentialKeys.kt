package com.korvect.nauterm

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import java.security.KeyStore
import java.security.MessageDigest
import java.security.SecureRandom
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/** Persist random per-account keys encrypted by a non-exportable AndroidKeyStore AES key. */
internal class CredentialKeys(context: Context) {
    private val preferences = context.getSharedPreferences("credential_keys", Context.MODE_PRIVATE)

    // The channel runs on the Android platform thread; synchronize as a safeguard for other callers.
    @Synchronized
    fun getOrCreate(account: String): ByteArray {
        val preferenceKey = Base64.encodeToString(
            MessageDigest.getInstance("SHA-256").digest(account.toByteArray(Charsets.UTF_8)),
            Base64.NO_WRAP or Base64.URL_SAFE,
        )
        val existing = preferences.getString(preferenceKey, null)
        val key = wrappingKey()
        if (existing != null) {
            val sealed = Base64.decode(existing, Base64.NO_WRAP)
            require(sealed.size >= IV_LENGTH + TAG_LENGTH + KEY_LENGTH) { "Invalid sealed key" }
            val cipher = Cipher.getInstance(TRANSFORMATION)
            cipher.init(Cipher.DECRYPT_MODE, key, GCMParameterSpec(128, sealed, 0, IV_LENGTH))
            cipher.updateAAD(account.toByteArray(Charsets.UTF_8))
            val plaintext = cipher.doFinal(sealed, IV_LENGTH, sealed.size - IV_LENGTH)
            require(plaintext.size == KEY_LENGTH) { "Invalid key length" }
            return plaintext
        }

        val plaintext = ByteArray(KEY_LENGTH).also { SecureRandom().nextBytes(it) }
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.ENCRYPT_MODE, key)
        cipher.updateAAD(account.toByteArray(Charsets.UTF_8))
        val sealed = cipher.iv + cipher.doFinal(plaintext)
        if (!preferences.edit().putString(
                preferenceKey,
                Base64.encodeToString(sealed, Base64.NO_WRAP),
            ).commit()
        ) {
            plaintext.fill(0)
            error("Unable to persist credential key")
        }
        return plaintext
    }

    private fun wrappingKey(): SecretKey {
        val keyStore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        (keyStore.getKey(KEY_ALIAS, null) as? SecretKey)?.let { return it }
        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore")
        generator.init(
            KeyGenParameterSpec.Builder(
                KEY_ALIAS,
                KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT,
            ).setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setKeySize(256)
                .build(),
        )
        return generator.generateKey()
    }

    private companion object {
        const val KEY_ALIAS = "com.korvect.nauterm.credential_keys.v1"
        const val TRANSFORMATION = "AES/GCM/NoPadding"
        const val KEY_LENGTH = 32
        const val IV_LENGTH = 12
        const val TAG_LENGTH = 16
    }
}
