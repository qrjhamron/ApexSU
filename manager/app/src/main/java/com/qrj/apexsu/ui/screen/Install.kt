package com.qrj.apexsu.ui.screen

import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns
import android.widget.Toast
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.captionBar
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ExpandLess
import androidx.compose.material.icons.filled.ExpandMore
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.dropUnlessResumed
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import com.qrj.apexsu.BuildConfig
import com.qrj.apexsu.Natives
import com.qrj.apexsu.R
import com.qrj.apexsu.ui.component.ChooseKmiDialog
import com.qrj.apexsu.ui.navigation3.LocalNavigator
import com.qrj.apexsu.ui.navigation3.Route
import com.qrj.apexsu.ui.util.LkmSelection
import top.yukonga.miuix.kmp.basic.Icon
import top.yukonga.miuix.kmp.basic.Scaffold
import top.yukonga.miuix.kmp.basic.Text
import top.yukonga.miuix.kmp.basic.TextButton
import top.yukonga.miuix.kmp.extra.SuperArrow
import java.security.MessageDigest
import java.util.Locale

private val Bg = Color(0xFF1C1C1E)
private val CardBg = Color(0xFF2C2C2E)
private val PrimaryText = Color.White
private val SecondaryText = Color(0xFF8E8E93)
private val Accent = Color(0xFF0A84FF)

@Composable
fun InstallScreen() {
    val navigator = LocalNavigator.current
    val context = androidx.compose.ui.platform.LocalContext.current
    val scope = rememberCoroutineScope()

    var selectedBootUriString by rememberSaveable { mutableStateOf<String?>(null) }
    var lkmSelection by rememberSaveable { mutableStateOf<LkmSelection>(LkmSelection.KmiNone) }
    var advancedExpanded by rememberSaveable { mutableStateOf(false) }
    var alwaysGrantShell by rememberSaveable { mutableStateOf(false) }
    var forceEnableAdbOnBoot by rememberSaveable { mutableStateOf(false) }

    val selectedBootUri = selectedBootUriString?.let(Uri::parse)
    val selectedBootDetails by produceState<SelectedBootImageDetails?>(
        initialValue = null,
        key1 = selectedBootUriString
    ) {
        value = null
        val uri = selectedBootUriString?.let(Uri::parse) ?: return@produceState
        value = withContext(Dispatchers.IO) { readBootImageDetails(context, uri) }
    }

    val selectedLkmUriString = (lkmSelection as? LkmSelection.LkmUri)?.uri?.toString()
    val selectedLkmLabel by produceState<String?>(
        initialValue = null,
        key1 = selectedLkmUriString
    ) {
        value = null
        val uri = selectedLkmUriString?.let(Uri::parse) ?: return@produceState
        value = withContext(Dispatchers.IO) { getUriDisplayLabel(context, uri) }
    }

    val currentKmi by produceState(initialValue = "") {
        value = withContext(Dispatchers.IO) { Natives.version.toString() }
    }

    val showChooseKmiDialog = rememberSaveable { mutableStateOf(false) }
    val chooseKmiDialog = ChooseKmiDialog(showChooseKmiDialog) { kmi ->
        val nextLkm = when {
            lkmSelection is LkmSelection.LkmUri -> lkmSelection
            !kmi.isNullOrBlank() -> LkmSelection.KmiString(kmi)
            else -> LkmSelection.KmiNone
        }
        selectedBootUri?.let { boot ->
            navigator.push(Route.Flash(FlashIt.PatchBoot(boot = boot, lkm = nextLkm)))
        }
    }

    val selectBootLauncher = rememberLauncherForActivityResult(ActivityResultContracts.GetContent()) { uri ->
        if (uri != null) selectedBootUriString = uri.toString()
    }
    val selectLkmLauncher = rememberLauncherForActivityResult(ActivityResultContracts.GetContent()) { uri ->
        if (uri == null) return@rememberLauncherForActivityResult
        scope.launch {
            val isKo = withContext(Dispatchers.IO) { isKoFile(context, uri) }
            if (isKo) {
                lkmSelection = LkmSelection.LkmUri(uri)
            } else {
                lkmSelection = LkmSelection.KmiNone
                Toast.makeText(
                    context,
                    context.getString(R.string.install_only_support_ko_file),
                    Toast.LENGTH_SHORT
                ).show()
            }
        }
    }

    Scaffold(
        popupHost = {},
        contentWindowInsets = WindowInsets(0.dp)
    ) { _ ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .background(Bg),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            item {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .background(CardBg)
                        .padding(horizontal = 16.dp, vertical = 18.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Image(
                        painter = painterResource(id = R.drawable.ic_launcher_foreground),
                        contentDescription = null,
                        modifier = Modifier.size(52.dp)
                    )
                    Spacer(Modifier.width(12.dp))
                    Column {
                        Text(
                            text = stringResource(R.string.install_banner_title),
                            color = PrimaryText,
                            fontWeight = FontWeight.SemiBold
                        )
                        Text(text = stringResource(R.string.install_banner_subtitle), color = SecondaryText)
                        Text(
                            text = stringResource(R.string.install_banner_version, BuildConfig.VERSION_NAME),
                            color = SecondaryText,
                            fontFamily = FontFamily.Monospace
                        )
                    }
                }
            }

            item {
                Column(
                    modifier = Modifier
                        .padding(horizontal = 16.dp)
                        .background(CardBg, RoundedCornerShape(10.dp))
                        .padding(16.dp)
                ) {
                    Text(stringResource(R.string.install_boot_image_title), color = PrimaryText, fontWeight = FontWeight.Medium)
                    Text(stringResource(R.string.install_boot_image_subtitle), color = SecondaryText)
                    OutlinedActionButton(
                        text = stringResource(R.string.install_select_boot_image),
                        onClick = { selectBootLauncher.launch("*/*") }
                    )
                    val bootDetailsText = selectedBootDetails?.let { details ->
                        buildString {
                            append(details.displayName)
                            append("\n")
                            append(context.getString(R.string.install_file_size))
                            append(": ")
                            append(details.sizeBytes?.let(::formatFileSize) ?: context.getString(R.string.install_file_unavailable))
                            append("\n")
                            append(context.getString(R.string.install_file_sha256))
                            append(": ")
                            append(details.sha256 ?: context.getString(R.string.install_file_unavailable))
                        }
                    }
                    if (bootDetailsText != null) {
                        Text(
                            modifier = Modifier.padding(top = 10.dp),
                            text = bootDetailsText,
                            color = SecondaryText,
                            fontFamily = FontFamily.Monospace
                        )
                    }
                    Text(
                        modifier = Modifier.padding(top = 10.dp),
                        text = buildAnnotatedString {
                            withStyle(SpanStyle(color = if (selectedBootUri != null) PrimaryText else SecondaryText)) { append("① Select") }
                            withStyle(SpanStyle(color = SecondaryText)) { append("  →  ") }
                            withStyle(SpanStyle(color = if (selectedBootUri != null) PrimaryText else SecondaryText)) { append("② Verify") }
                            withStyle(SpanStyle(color = SecondaryText)) { append("  →  ") }
                            withStyle(SpanStyle(color = if (selectedBootUri != null) PrimaryText else SecondaryText)) { append("③ Patch") }
                        }
                    )
                }
            }

            item {
                Column(
                    modifier = Modifier
                        .padding(horizontal = 16.dp)
                        .background(CardBg, RoundedCornerShape(10.dp))
                        .padding(16.dp)
                ) {
                    Text(stringResource(R.string.install_lkm_title), color = PrimaryText, fontWeight = FontWeight.Medium)
                    Text(stringResource(R.string.install_lkm_subtitle), color = SecondaryText)
                    OutlinedActionButton(
                        text = stringResource(R.string.install_lkm_select_button),
                        onClick = { selectLkmLauncher.launch("*/*") }
                    )
                    selectedLkmLabel?.let {
                        Text(
                            modifier = Modifier.padding(top = 8.dp),
                            text = it,
                            color = SecondaryText,
                            fontFamily = FontFamily.Monospace
                        )
                    }
                    Text(
                        modifier = Modifier.padding(top = 8.dp),
                        text = stringResource(R.string.install_device_kmi_info, currentKmi.ifBlank { "-" }),
                        color = SecondaryText
                    )
                    if (selectedLkmLabel == null) {
                        Text(
                            modifier = Modifier.padding(top = 6.dp),
                            text = stringResource(R.string.install_lkm_fallback_info),
                            color = SecondaryText
                        )
                    }
                }
            }

            item {
                Column(
                    modifier = Modifier
                        .padding(horizontal = 16.dp)
                        .background(CardBg, RoundedCornerShape(10.dp))
                        .padding(16.dp)
                ) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clickable { advancedExpanded = !advancedExpanded },
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(
                            text = stringResource(R.string.install_advanced_options),
                            color = PrimaryText,
                            fontWeight = FontWeight.Medium,
                            modifier = Modifier.weight(1f)
                        )
                        Icon(
                            imageVector = if (advancedExpanded) Icons.Filled.ExpandLess else Icons.Filled.ExpandMore,
                            contentDescription = null,
                            tint = SecondaryText
                        )
                    }
                    if (advancedExpanded) {
                        SuperArrow(
                            title = stringResource(R.string.install_toggle_shell_root),
                            summary = stringResource(R.string.install_toggle_shell_root_desc),
                            endActions = {
                                Text(if (alwaysGrantShell) "ON" else "OFF", color = SecondaryText)
                            },
                            onClick = { alwaysGrantShell = !alwaysGrantShell }
                        )
                        SuperArrow(
                            title = stringResource(R.string.install_toggle_adb_boot),
                            summary = stringResource(R.string.install_toggle_adb_boot_desc),
                            endActions = {
                                Text(if (forceEnableAdbOnBoot) "ON" else "OFF", color = SecondaryText)
                            },
                            onClick = { forceEnableAdbOnBoot = !forceEnableAdbOnBoot }
                        )
                    }
                }
            }

            item {
                TextButton(
                    modifier = Modifier
                        .padding(horizontal = 16.dp)
                        .fillMaxWidth()
                        .background(
                            if (selectedBootUri != null) Accent else Color(0xFF3A3A3C),
                            RoundedCornerShape(10.dp)
                        ),
                    text = stringResource(R.string.install_next),
                    enabled = selectedBootUri != null,
                    onClick = dropUnlessResumed {
                        val boot = selectedBootUri ?: return@dropUnlessResumed
                        val lkmSelected = lkmSelection !is LkmSelection.KmiNone
                        if (!lkmSelected) {
                            showChooseKmiDialog.value = true
                            chooseKmiDialog
                        } else {
                            navigator.push(Route.Flash(FlashIt.PatchBoot(boot = boot, lkm = lkmSelection)))
                        }
                    }
                )
                Spacer(
                    modifier = Modifier.height(
                        16.dp + WindowInsets.navigationBars.getBottom(androidx.compose.ui.platform.LocalDensity.current).dp +
                            WindowInsets.captionBar.getBottom(androidx.compose.ui.platform.LocalDensity.current).dp
                    )
                )
            }
        }
    }
}

@Composable
private fun OutlinedActionButton(
    text: String,
    onClick: () -> Unit
) {
    Box(
        modifier = Modifier
            .padding(top = 10.dp)
            .fillMaxWidth()
            .border(1.dp, Color.White, RoundedCornerShape(8.dp))
            .clickable(onClick = onClick)
            .padding(vertical = 11.dp),
        contentAlignment = Alignment.Center
    ) {
        Text(text = text, color = PrimaryText, fontWeight = FontWeight.Medium)
    }
}

private data class SelectedBootImageDetails(
    val displayName: String,
    val sizeBytes: Long?,
    val sha256: String?
)

private fun readBootImageDetails(context: Context, uri: Uri): SelectedBootImageDetails {
    val metadata = queryUriMetadata(context, uri)
    return SelectedBootImageDetails(
        displayName = metadata.first ?: uri.toString(),
        sizeBytes = metadata.second,
        sha256 = computeSha256(context, uri)
    )
}

private fun getUriDisplayLabel(context: Context, uri: Uri): String {
    return queryUriMetadata(context, uri).first ?: uri.toString()
}

private fun queryUriMetadata(context: Context, uri: Uri): Pair<String?, Long?> {
    return try {
        context.contentResolver.query(
            uri,
            arrayOf(OpenableColumns.DISPLAY_NAME, OpenableColumns.SIZE),
            null,
            null,
            null
        )?.use { cursor ->
            if (!cursor.moveToFirst()) return@use null
            val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
            val sizeIndex = cursor.getColumnIndex(OpenableColumns.SIZE)
            val name = if (nameIndex != -1) cursor.getString(nameIndex) else null
            val size = if (sizeIndex != -1 && !cursor.isNull(sizeIndex)) cursor.getLong(sizeIndex) else null
            name to size
        } ?: (null to null)
    } catch (_: Throwable) {
        null to null
    }
}

private fun computeSha256(context: Context, uri: Uri): String? {
    return try {
        val digest = MessageDigest.getInstance("SHA-256")
        context.contentResolver.openInputStream(uri)?.use { input ->
            val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
            while (true) {
                val read = input.read(buffer)
                if (read <= 0) break
                digest.update(buffer, 0, read)
            }
        } ?: return null
        digest.digest().joinToString("") { "%02x".format(it.toInt() and 0xff) }
    } catch (_: Throwable) {
        null
    }
}

private fun formatFileSize(sizeBytes: Long): String {
    val units = arrayOf("B", "KB", "MB", "GB")
    var size = sizeBytes.toDouble()
    var unit = 0
    while (size >= 1024 && unit < units.lastIndex) {
        size /= 1024
        unit++
    }
    return if (unit == 0) {
        "${sizeBytes} ${units[unit]}"
    } else {
        String.format(Locale.US, "%.1f %s", size, units[unit])
    }
}

private fun isKoFile(context: Context, uri: Uri): Boolean {
    val seg = uri.lastPathSegment ?: ""
    if (seg.endsWith(".ko", ignoreCase = true)) return true
    return try {
        context.contentResolver.query(
            uri,
            arrayOf(OpenableColumns.DISPLAY_NAME),
            null,
            null,
            null
        )?.use { cursor ->
            val idx = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
            if (idx != -1 && cursor.moveToFirst()) {
                val name = cursor.getString(idx)
                name?.endsWith(".ko", ignoreCase = true) == true
            } else {
                false
            }
        } ?: false
    } catch (_: Throwable) {
        false
    }
}
