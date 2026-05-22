package com.qrj.apexsu.ui.webui

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class WebViewInterfacePolicyTest {
    @Test
    fun allowlist_contains_only_expected_actions() {
        assertTrue(isAllowlistedWebUiAction("moduleInfo"))
        assertTrue(isAllowlistedWebUiAction("listPackages"))
        assertTrue(isAllowlistedWebUiAction("getPackagesInfo"))
        assertTrue(isAllowlistedWebUiAction("runModuleAction"))

        assertFalse(isAllowlistedWebUiAction("exec"))
        assertFalse(isAllowlistedWebUiAction("spawn"))
        assertFalse(isAllowlistedWebUiAction("rm -rf /"))
    }
}
