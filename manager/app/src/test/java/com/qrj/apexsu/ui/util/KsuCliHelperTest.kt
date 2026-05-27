package com.qrj.apexsu.ui.util

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder

class KsuCliHelperTest {
    @get:Rule
    val temporaryFolder = TemporaryFolder()

    @Test
    fun resolveNativeHelper_usesApplicationNativeLibraryDir() {
        val helper = resolveNativeKsudHelper("/data/app/package/lib/arm64")

        assertEquals("/data/app/package/lib/arm64/libksud.so", helper.path)
    }

    @Test
    fun prepareExecutableHelper_copiesNativeHelperToExecutableCachePath() {
        val nativeDir = temporaryFolder.newFolder("native")
        val codeCacheDir = temporaryFolder.newFolder("code_cache")
        val nativeHelper = nativeDir.resolve("libksud.so")
        nativeHelper.writeText("ksud")
        nativeHelper.setReadable(true, false)
        nativeHelper.setExecutable(false, false)

        val logs = mutableListOf<String>()
        val executableHelper = prepareExecutableKsudHelper(
            nativeHelper = nativeHelper,
            codeCacheDir = codeCacheDir,
            onLog = logs::add
        )

        assertEquals(codeCacheDir.resolve("apexsu/libksud.so").absolutePath, executableHelper.absolutePath)
        assertTrue(executableHelper.exists())
        assertTrue(executableHelper.canRead())
        assertTrue(executableHelper.canExecute())
        assertTrue(logs.any { it == "copied_helper_path=${executableHelper.absolutePath}" })
    }

    @Test
    fun prepareExecutableHelper_reportsMissingNativeHelper() {
        val codeCacheDir = temporaryFolder.newFolder("code_cache")

        val result = runCatching {
            prepareExecutableKsudHelper(
                nativeHelper = temporaryFolder.root.resolve("missing/libksud.so"),
                codeCacheDir = codeCacheDir,
                onLog = {}
            )
        }

        assertTrue(result.isFailure)
        assertEquals(
            "ApexSU native helper missing. Reinstall the official arm64 APK.",
            result.exceptionOrNull()?.message
        )
    }
}
