package com.qrj.apexsu.ui.screen

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class InstallRepoLkmTest {
    @Test
    fun detectRepoLkmInfo_mapsAndroid12Kernel510ToLatestAsset() {
        val info = detectRepoLkmInfo("5.10.198-android12-9-g123456")

        assertEquals("android12-5.10", info?.key)
        assertEquals("kernelsu-5.10.209-arm64.ko", info?.fileName)
        assertEquals("5.10.209-arm64.ko", info?.displayVersion)
        assertEquals(
            "https://github.com/qrjhamron/ApexSU/releases/latest/download/kernelsu-5.10.209-arm64.ko",
            info?.downloadUrl
        )
    }

    @Test
    fun detectRepoLkmInfo_returnsNullForUnknownKernel() {
        assertNull(detectRepoLkmInfo("4.19.0-custom"))
    }
}
