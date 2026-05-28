package com.qrj.apexsu.ui.screen

import android.annotation.SuppressLint
import android.content.Context
import android.net.Uri
import android.os.Build
import android.provider.OpenableColumns
import android.system.Os
import android.widget.Toast
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.add
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.captionBar
import androidx.compose.foundation.layout.displayCutout
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.dropUnlessResumed
import com.qrj.apexsu.R
import com.qrj.apexsu.ksuApp
import com.qrj.apexsu.ui.navigation3.LocalNavigator
import com.qrj.apexsu.ui.navigation3.Route
import com.qrj.apexsu.ui.theme.LocalEnableBlur
import com.qrj.apexsu.ui.util.LkmSelection
import dev.chrisbanes.haze.HazeState
import dev.chrisbanes.haze.HazeStyle
import dev.chrisbanes.haze.HazeTint
import dev.chrisbanes.haze.hazeEffect
import dev.chrisbanes.haze.hazeSource
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import okhttp3.Request
import top.yukonga.miuix.kmp.basic.Card
import top.yukonga.miuix.kmp.basic.CardDefaults
import top.yukonga.miuix.kmp.basic.Icon
import top.yukonga.miuix.kmp.basic.IconButton
import top.yukonga.miuix.kmp.basic.MiuixScrollBehavior
import top.yukonga.miuix.kmp.basic.Scaffold
import top.yukonga.miuix.kmp.basic.ScrollBehavior
import top.yukonga.miuix.kmp.basic.Text
import top.yukonga.miuix.kmp.basic.TopAppBar
import top.yukonga.miuix.kmp.icon.MiuixIcons
import top.yukonga.miuix.kmp.icon.extended.Back
import top.yukonga.miuix.kmp.theme.MiuixTheme.colorScheme
import top.yukonga.miuix.kmp.utils.overScrollVertical
import top.yukonga.miuix.kmp.utils.scrollEndHaptic
import java.io.File
import java.io.IOException
import java.security.MessageDigest
import java.util.Locale

private const val INSTALL_CARD = 0xFF1C1C1E
private const val INSTALL_BUTTON = 0xFF2C2C2E
private const val INSTALL_BORDER = 0xFF3A3A3C
private const val INSTALL_SECONDARY = 0xFF8E8E93
private const val INSTALL_INACTIVE_STEP = 0xFF636366
private const val INSTALL_CTA_BLUE = 0xFF0A84FF
private const val INSTALL_RED = 0xFFFF453A

private enum class LkmInstallOption {
    Local,
    Repository
}

internal data class RepoLkmInfo(
    val key: String,
    val fileName: String,
    val displayVersion: String,
    val downloadUrl: String,
)

internal fun detectRepoLkmInfo(
    release: String,
    supportedAbis: List<String> = listOf("arm64-v8a"),
): RepoLkmInfo? {
    if (!release.contains("android", ignoreCase = true)) return null
    if (supportedAbis.none { it == "arm64-v8a" }) return null
    val androidMatch = Regex("android(\\d+)").find(release)?.groupValues?.getOrNull(1)
    val kernelMatch = Regex("(\\d+\\.\\d+)").find(release)?.groupValues?.getOrNull(1)
    val key = if (androidMatch != null && kernelMatch != null) "android$androidMatch-$kernelMatch" else return null
    val fileName = when (key) {
        "android12-5.10", "android13-5.10" -> "kernelsu-5.10.209-arm64.ko"
        "android13-5.15", "android14-5.15" -> "kernelsu-5.15.148-arm64.ko"
        "android14-6.1" -> "kernelsu-6.1.96-arm64.ko"
        "android15-6.6" -> "kernelsu-6.6.35-arm64.ko"
        "android16-6.12" -> "kernelsu-6.12.6-arm64.ko"
        else -> null
    } ?: return null

    return RepoLkmInfo(
        key = key,
        fileName = fileName,
        displayVersion = fileName.removePrefix("kernelsu-"),
        downloadUrl = "https://github.com/qrjhamron/ApexSU/releases/latest/download/$fileName"
    )
}

@SuppressLint("LocalContextGetResourceValueCall")
@Composable
fun InstallScreen() {
    val navigator = LocalNavigator.current
    val context = LocalContext.current
    val enableBlur = LocalEnableBlur.current
    val scope = rememberCoroutineScope()
    val kernelRelease = remember { Os.uname().release }
    val isGkiDevice = remember(kernelRelease) { kernelRelease.contains("android", ignoreCase = true) }

    var selectedBootUriString by rememberSaveable { mutableStateOf<String?>(null) }
    val selectedBootUri = selectedBootUriString?.let(Uri::parse)
    val selectedBootDetails by produceState<SelectedBootImageDetails?>(
        initialValue = null,
        key1 = selectedBootUriString
    ) {
        value = null
        val uri = selectedBootUriString?.let(Uri::parse) ?: return@produceState
        value = withContext(Dispatchers.IO) {
            readBootImageDetails(context, uri)
        }
    }

    var lkmOption by rememberSaveable { mutableStateOf(LkmInstallOption.Repository) }
    var lkmSelection by rememberSaveable { mutableStateOf<LkmSelection>(LkmSelection.KmiNone) }
    val selectedLkmUriString = (lkmSelection as? LkmSelection.LkmUri)?.uri?.toString()
    val selectedLkmLabel by produceState<String?>(
        initialValue = null,
        key1 = selectedLkmUriString
    ) {
        value = null
        val uri = selectedLkmUriString?.let(Uri::parse) ?: return@produceState
        value = withContext(Dispatchers.IO) {
            getUriDisplayLabel(context, uri)
        }
    }
    val repoLkmInfo = remember(kernelRelease) {
        if (isGkiDevice) detectRepoLkmInfo(kernelRelease, Build.SUPPORTED_ABIS.toList()) else null
    }
    var isDownloadingRepoLkm by rememberSaveable { mutableStateOf(false) }
    var repoDownloadProgress by rememberSaveable { mutableFloatStateOf(0f) }
    val repositoryOptionAvailable = isGkiDevice && repoLkmInfo != null
    val localLkmSelected = lkmSelection is LkmSelection.LkmUri
    val canInstall = selectedBootUri != null &&
            !isDownloadingRepoLkm &&
            isGkiDevice &&
            when (lkmOption) {
                LkmInstallOption.Repository -> repositoryOptionAvailable
                LkmInstallOption.Local -> localLkmSelected
            }

    val selectBootLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.GetContent()
    ) { uri ->
        if (uri == null) return@rememberLauncherForActivityResult
        selectedBootUriString = uri.toString()
    }

    val selectLkmLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.GetContent()
    ) { uri ->
        if (uri == null) return@rememberLauncherForActivityResult
        scope.launch {
            val isKo = withContext(Dispatchers.IO) {
                isKoFile(context, uri)
            }
            if (isKo) {
                lkmOption = LkmInstallOption.Local
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

    fun startPatch(bootUri: Uri, lkm: LkmSelection) {
        navigator.push(
            Route.Flash(
                FlashIt.PatchBoot(
                    boot = bootUri,
                    lkm = lkm
                )
            )
        )
    }

    fun onInstallClick() {
        if (!canInstall) return
        val bootUri = selectedBootUri
        when (lkmOption) {
            LkmInstallOption.Local -> {
                val localLkm = lkmSelection as? LkmSelection.LkmUri
                if (localLkm == null) {
                    selectLkmLauncher.launch("*/*")
                    return
                }
                startPatch(bootUri, localLkm)
            }

            LkmInstallOption.Repository -> {
                val info = repoLkmInfo
                if (info == null) {
                    Toast.makeText(
                        context,
                        context.getString(R.string.install_repo_lkm_not_detected),
                        Toast.LENGTH_SHORT
                    ).show()
                    return
                }
                scope.launch {
                    isDownloadingRepoLkm = true
                    repoDownloadProgress = 0f
                    val result = runCatching {
                        downloadRepoLkm(context, info) { progress ->
                            repoDownloadProgress = progress
                        }
                    }
                    isDownloadingRepoLkm = false
                    result.onSuccess { uri ->
                        lkmSelection = LkmSelection.LkmUri(uri)
                        startPatch(bootUri, LkmSelection.LkmUri(uri))
                    }.onFailure {
                        Toast.makeText(
                            context,
                            context.getString(R.string.install_repo_lkm_download_failed),
                            Toast.LENGTH_SHORT
                        ).show()
                    }
                }
            }
        }
    }

    val scrollBehavior = MiuixScrollBehavior()
    val hazeState = remember { HazeState() }
    val hazeStyle = if (enableBlur) {
        HazeStyle(
            backgroundColor = colorScheme.surface,
            tint = HazeTint(colorScheme.surface.copy(0.8f))
        )
    } else {
        HazeStyle.Unspecified
    }

    Scaffold(
        topBar = {
            TopBar(
                onBack = dropUnlessResumed { navigator.pop() },
                scrollBehavior = scrollBehavior,
                hazeState = hazeState,
                hazeStyle = hazeStyle,
                enableBlur = enableBlur,
            )
        },
        popupHost = { },
        contentWindowInsets = WindowInsets.systemBars.add(WindowInsets.displayCutout).only(WindowInsetsSides.Horizontal)
    ) { innerPadding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxHeight()
                .scrollEndHaptic()
                .overScrollVertical()
                .nestedScroll(scrollBehavior.nestedScrollConnection)
                .let { if (enableBlur) it.hazeSource(state = hazeState) else it }
                .padding(top = 12.dp)
                .padding(horizontal = 16.dp),
            contentPadding = innerPadding,
            overscrollEffect = null,
        ) {
            item {
                StepIndicator(activeStep = if (selectedBootUri == null) 1 else 2)

                BootImageCard(
                    selectedBootDetails = selectedBootDetails,
                    isReading = selectedBootUri != null && selectedBootDetails == null,
                    onPickBoot = { selectBootLauncher.launch("*/*") }
                )

                AnimatedVisibility(
                    visible = selectedBootUri != null,
                    enter = expandVertically(),
                    exit = shrinkVertically()
                ) {
                    LkmSelectionCard(
                        option = lkmOption,
                        selectedLkmLabel = selectedLkmLabel,
                        selectedLkmUriString = selectedLkmUriString,
                        repoLkmInfo = repoLkmInfo,
                        isGkiDevice = isGkiDevice,
                        repositoryOptionAvailable = repositoryOptionAvailable,
                        isDownloading = isDownloadingRepoLkm,
                        downloadProgress = repoDownloadProgress,
                        onSelectOption = { selected ->
                            lkmOption = selected
                            if (selected == LkmInstallOption.Repository) {
                                lkmSelection = LkmSelection.KmiNone
                            }
                        },
                        onPickLocalLkm = { selectLkmLauncher.launch("*/*") }
                    )
                }

                InstallActionButton(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 12.dp),
                    text = stringResource(id = R.string.install_install_id),
                    enabled = canInstall,
                    onClick = { onInstallClick() }
                )
                Spacer(
                    Modifier.height(
                        WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding() +
                                WindowInsets.captionBar.asPaddingValues().calculateBottomPadding()
                    )
                )
            }
        }
    }
}

@Composable
private fun StepIndicator(activeStep: Int) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(bottom = 12.dp),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        StepText(text = stringResource(R.string.install_step_select), active = activeStep == 1)
        Text(
            modifier = Modifier.padding(horizontal = 8.dp),
            text = stringResource(R.string.install_step_separator),
            color = Color(INSTALL_INACTIVE_STEP),
            fontSize = 13.sp,
        )
        StepText(text = stringResource(R.string.install_step_lkm), active = activeStep == 2)
        Text(
            modifier = Modifier.padding(horizontal = 8.dp),
            text = stringResource(R.string.install_step_separator),
            color = Color(INSTALL_INACTIVE_STEP),
            fontSize = 13.sp,
        )
        StepText(text = stringResource(R.string.install_step_install), active = activeStep == 3)
    }
}

@Composable
private fun StepText(text: String, active: Boolean) {
    Text(
        text = text,
        color = if (active) Color.White else Color(INSTALL_INACTIVE_STEP),
        fontSize = 13.sp,
        fontWeight = if (active) FontWeight.SemiBold else FontWeight.Medium,
    )
}

@Composable
private fun BootImageCard(
    selectedBootDetails: SelectedBootImageDetails?,
    isReading: Boolean,
    onPickBoot: () -> Unit,
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.defaultColors(color = Color(INSTALL_CARD)),
        insideMargin = androidx.compose.foundation.layout.PaddingValues(16.dp),
        cornerRadius = 14.dp,
    ) {
        SectionHeader(title = stringResource(R.string.install_boot_image_title))
        GreyActionButton(
            modifier = Modifier
                .fillMaxWidth()
                .padding(top = 14.dp),
            text = stringResource(R.string.install_pick_boot_img),
            onClick = onPickBoot
        )
        val detailsText = selectedBootDetails?.let { details ->
            buildString {
                append(details.displayName)
                append("\n")
                append(stringResource(R.string.install_file_size))
                append(": ")
                append(details.sizeBytes?.let(::formatFileSize) ?: stringResource(R.string.install_file_unavailable))
                append("\n")
                append(stringResource(R.string.install_file_sha256))
                append(": ")
                append(details.sha256 ?: stringResource(R.string.install_file_unavailable))
            }
        }
        if (detailsText != null || isReading) {
            Text(
                modifier = Modifier.padding(top = 12.dp),
                text = detailsText ?: stringResource(R.string.install_reading_file_details),
                color = Color(INSTALL_SECONDARY),
                fontSize = 12.sp,
                fontFamily = FontFamily.Monospace,
            )
        }
    }
}

@Composable
private fun LkmSelectionCard(
    option: LkmInstallOption,
    selectedLkmLabel: String?,
    selectedLkmUriString: String?,
    repoLkmInfo: RepoLkmInfo?,
    isGkiDevice: Boolean,
    repositoryOptionAvailable: Boolean,
    isDownloading: Boolean,
    downloadProgress: Float,
    onSelectOption: (LkmInstallOption) -> Unit,
    onPickLocalLkm: () -> Unit,
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 12.dp),
        colors = CardDefaults.defaultColors(color = Color(INSTALL_CARD)),
        insideMargin = androidx.compose.foundation.layout.PaddingValues(16.dp),
        cornerRadius = 14.dp,
    ) {
        SectionHeader(title = stringResource(R.string.install_lkm_title))
        if (!isGkiDevice) {
            NonGkiInstallBlocker(modifier = Modifier.padding(top = 14.dp))
        } else {
            RadioOptionCard(
                modifier = Modifier.padding(top = 14.dp),
                selected = option == LkmInstallOption.Local,
                title = stringResource(R.string.home_use_local_lkm_id),
                subtext = selectedLkmLabel
                    ?: selectedLkmUriString?.let { stringResource(R.string.install_reading_selected_file) }
                    ?: stringResource(R.string.install_local_lkm_helper),
                enabled = true,
                onClick = { onSelectOption(LkmInstallOption.Local) }
            )
            AnimatedVisibility(visible = option == LkmInstallOption.Local) {
                Column {
                    GreyActionButton(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(top = 10.dp),
                        text = stringResource(R.string.install_select_lkm_file),
                        onClick = onPickLocalLkm
                    )
                    if (selectedLkmLabel != null) {
                        Text(
                            modifier = Modifier.padding(top = 8.dp),
                            text = selectedLkmLabel,
                            color = Color(INSTALL_SECONDARY),
                            fontSize = 12.sp,
                        )
                    }
                }
            }
            RadioOptionCard(
                modifier = Modifier.padding(top = 10.dp),
                selected = option == LkmInstallOption.Repository && repositoryOptionAvailable,
                title = stringResource(R.string.install_repo_lkm_title),
                subtext = repoLkmInfo?.let {
                    stringResource(R.string.install_repo_lkm_kmi_module, it.key, it.displayVersion) +
                            "\n" + stringResource(R.string.install_repo_lkm_source_short)
                } ?: stringResource(R.string.install_repo_lkm_unavailable),
                enabled = repositoryOptionAvailable,
                onClick = { onSelectOption(LkmInstallOption.Repository) }
            )
            AnimatedVisibility(visible = isDownloading) {
                LinearDownloadProgress(
                    modifier = Modifier.padding(top = 12.dp),
                    progress = downloadProgress
                )
            }
        }
    }
}

@Composable
private fun SectionHeader(
    title: String,
) {
    Text(
        text = title,
        color = Color.White,
        fontSize = 17.sp,
        fontWeight = FontWeight.SemiBold,
    )
}

@Composable
private fun RadioOptionCard(
    modifier: Modifier = Modifier,
    selected: Boolean,
    title: String,
    subtext: String?,
    enabled: Boolean,
    onClick: () -> Unit,
) {
    val borderColor = if (selected) Color.White else Color(INSTALL_BORDER)
    Card(
        modifier = modifier
            .fillMaxWidth()
            .border(1.dp, borderColor, RoundedCornerShape(14.dp)),
        onClick = { if (enabled) onClick() },
        colors = CardDefaults.defaultColors(color = Color.Transparent),
        insideMargin = androidx.compose.foundation.layout.PaddingValues(12.dp),
        cornerRadius = 14.dp,
        showIndication = enabled,
    ) {
        Column(modifier = Modifier.fillMaxWidth()) {
            Text(
                text = title,
                color = Color.White,
                fontSize = 15.sp,
                fontWeight = FontWeight.Medium,
            )
            if (!subtext.isNullOrBlank()) {
                Text(
                    modifier = Modifier.padding(top = 3.dp),
                    text = subtext,
                    color = Color(INSTALL_SECONDARY),
                    fontSize = 12.sp,
                )
            }
        }
    }
}

@Composable
private fun NonGkiInstallBlocker(
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier.fillMaxWidth()) {
        Text(
            text = stringResource(R.string.home_non_gki_title),
            color = Color(INSTALL_RED),
            fontSize = 15.sp,
            fontWeight = FontWeight.SemiBold,
        )
        Text(
            modifier = Modifier.padding(top = 4.dp),
            text = stringResource(R.string.home_non_gki_body),
            color = Color(INSTALL_SECONDARY),
            fontSize = 13.sp,
        )
        Text(
            modifier = Modifier.padding(top = 4.dp),
            text = stringResource(R.string.install_non_gki_kernel_reason),
            color = Color(INSTALL_SECONDARY),
            fontSize = 13.sp,
        )
    }
}

@Composable
private fun GreyActionButton(
    modifier: Modifier = Modifier,
    text: String,
    onClick: () -> Unit,
) {
    Card(
        modifier = modifier
            .height(48.dp)
            .border(1.dp, Color(INSTALL_BORDER), RoundedCornerShape(14.dp)),
        colors = CardDefaults.defaultColors(color = Color(INSTALL_BUTTON)),
        cornerRadius = 14.dp,
        insideMargin = androidx.compose.foundation.layout.PaddingValues(0.dp),
        onClick = onClick,
        showIndication = true,
    ) {
        Box(
            modifier = Modifier.fillMaxWidth().height(48.dp),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = text,
                color = Color.White,
                fontSize = 15.sp,
                fontWeight = FontWeight.Medium,
            )
        }
    }
}

@Composable
private fun InstallActionButton(
    modifier: Modifier = Modifier,
    text: String,
    enabled: Boolean,
    onClick: () -> Unit,
) {
    Card(
        modifier = modifier.height(50.dp),
        colors = CardDefaults.defaultColors(
            color = if (enabled) Color(INSTALL_CTA_BLUE) else Color(INSTALL_BUTTON)
        ),
        cornerRadius = 14.dp,
        insideMargin = androidx.compose.foundation.layout.PaddingValues(0.dp),
        onClick = { if (enabled) onClick() },
        showIndication = enabled,
    ) {
        Box(
            modifier = Modifier.fillMaxWidth().height(50.dp),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = text,
                color = Color.White,
                fontSize = 16.sp,
                fontWeight = FontWeight.SemiBold,
            )
        }
    }
}

@Composable
private fun LinearDownloadProgress(
    modifier: Modifier = Modifier,
    progress: Float,
) {
    Box(
        modifier = modifier
            .fillMaxWidth()
            .height(4.dp)
            .background(Color(INSTALL_BORDER), RoundedCornerShape(2.dp))
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth(progress.coerceIn(0f, 1f))
                .height(4.dp)
                .background(Color.White, RoundedCornerShape(2.dp))
        )
    }
}

private suspend fun downloadRepoLkm(
    context: Context,
    info: RepoLkmInfo,
    onProgress: (Float) -> Unit,
): Uri = withContext(Dispatchers.IO) {
    val targetDir = File(context.filesDir, "apexsu")
    val target = File(targetDir, info.fileName)
    targetDir.mkdirs()

    ksuApp.okhttpClient.newCall(Request.Builder().url(info.downloadUrl).build()).execute().use { response ->
        if (!response.isSuccessful) throw IOException("HTTP ${response.code}")
        val body = response.body
        val total = body.contentLength()
        body.byteStream().use { input ->
            target.outputStream().use { output ->
                val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
                var copied = 0L
                while (true) {
                    val read = input.read(buffer)
                    if (read < 0) break
                    output.write(buffer, 0, read)
                    copied += read
                    if (total > 0L) {
                        withContext(Dispatchers.Main) {
                            onProgress((copied.toFloat() / total.toFloat()).coerceIn(0f, 1f))
                        }
                    }
                }
            }
        }
    }
    withContext(Dispatchers.Main) { onProgress(1f) }
    Uri.fromFile(target)
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

@Composable
private fun TopBar(
    onBack: () -> Unit = {},
    scrollBehavior: ScrollBehavior,
    hazeState: HazeState,
    hazeStyle: HazeStyle,
    enableBlur: Boolean
) {
    TopAppBar(
        modifier = if (enableBlur) {
            Modifier.hazeEffect(hazeState) {
                style = hazeStyle
                blurRadius = 30.dp
                noiseFactor = 0f
            }
        } else {
            Modifier
        },
        color = if (enableBlur) Color.Transparent else colorScheme.surface,
        title = stringResource(R.string.install),
        navigationIcon = {
            IconButton(
                modifier = Modifier.padding(start = 16.dp),
                onClick = onBack
            ) {
                val layoutDirection = LocalLayoutDirection.current
                Icon(
                    modifier = Modifier.graphicsLayer {
                        if (layoutDirection == LayoutDirection.Rtl) scaleX = -1f
                    },
                    imageVector = MiuixIcons.Back,
                    tint = colorScheme.onSurface,
                    contentDescription = null,
                )
            }
        },
        scrollBehavior = scrollBehavior
    )
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
