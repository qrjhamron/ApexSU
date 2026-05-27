package com.qrj.apexsu.ui.screen

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.net.Uri
import android.os.Build
import android.provider.OpenableColumns
import android.system.Os
import android.widget.Toast
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.annotation.StringRes
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.core.tween
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.add
import androidx.compose.foundation.layout.displayCutout
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Warning
import androidx.compose.material.icons.rounded.CheckCircleOutline
import androidx.compose.material.icons.rounded.ChevronRight
import androidx.compose.material.icons.rounded.Extension
import androidx.compose.material.icons.rounded.ErrorOutline
import androidx.compose.material.icons.rounded.FolderOpen
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.content.pm.PackageInfoCompat
import dev.chrisbanes.haze.HazeState
import dev.chrisbanes.haze.HazeStyle
import dev.chrisbanes.haze.HazeTint
import dev.chrisbanes.haze.hazeEffect
import dev.chrisbanes.haze.hazeSource
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewmodel.compose.viewModel
import com.qrj.apexsu.KernelVersion
import com.qrj.apexsu.Natives
import com.qrj.apexsu.R
import com.qrj.apexsu.getKernelVersion
import com.qrj.apexsu.ui.LocalMainPagerState
import com.qrj.apexsu.ui.component.DropdownItem
import com.qrj.apexsu.ui.component.RebootListPopup
import com.qrj.apexsu.ui.component.rememberConfirmDialog
import com.qrj.apexsu.ui.navigation3.Navigator
import com.qrj.apexsu.ui.navigation3.Route
import com.qrj.apexsu.ui.theme.LocalEnableBlur
import com.qrj.apexsu.ui.theme.LocalReduceMotion
import com.qrj.apexsu.ui.theme.isInDarkTheme
import com.qrj.apexsu.ui.util.checkNewVersion
import com.qrj.apexsu.ui.util.getModuleCount
import com.qrj.apexsu.ui.util.getSELinuxStatus
import com.qrj.apexsu.ui.util.getSuperuserCount
import com.qrj.apexsu.ui.util.LkmSelection
import com.qrj.apexsu.ui.util.module.LatestVersionInfo
import com.qrj.apexsu.ui.util.reboot
import com.qrj.apexsu.ui.util.rootAvailable
import top.yukonga.miuix.kmp.basic.BasicComponent
import top.yukonga.miuix.kmp.basic.Card
import top.yukonga.miuix.kmp.basic.CardDefaults
import top.yukonga.miuix.kmp.basic.Icon
import top.yukonga.miuix.kmp.basic.MiuixScrollBehavior
import top.yukonga.miuix.kmp.basic.Scaffold
import top.yukonga.miuix.kmp.basic.ScrollBehavior
import top.yukonga.miuix.kmp.basic.Text
import top.yukonga.miuix.kmp.basic.TextButton
import top.yukonga.miuix.kmp.icon.MiuixIcons
import top.yukonga.miuix.kmp.icon.extended.Link
import top.yukonga.miuix.kmp.theme.MiuixTheme
import top.yukonga.miuix.kmp.theme.MiuixTheme.colorScheme
import top.yukonga.miuix.kmp.theme.MiuixTheme.isDynamicColor
import top.yukonga.miuix.kmp.utils.PressFeedbackType
import top.yukonga.miuix.kmp.utils.overScrollVertical
import top.yukonga.miuix.kmp.utils.scrollEndHaptic

private const val IOS_BG = 0xFF1C1C1E
private const val IOS_SECONDARY = 0xFF8E8E93
private const val IOS_SEPARATOR = 0xFF38383A
private const val IOS_BLUE = 0xFF0A84FF
private const val IOS_GREEN = 0xFF30D158
private const val IOS_ORANGE = 0xFFFF9F0A

class HomeViewModel : ViewModel() {
    var selectedKoDisplayName by mutableStateOf<String?>(null)
        private set

    fun updateSelectedKoName(name: String?) {
        selectedKoDisplayName = name
    }
}

@Composable
fun HomePager(
    navigator: Navigator,
    bottomInnerPadding: Dp
) {
    val homeViewModel: HomeViewModel = viewModel()
    val kernelVersion = getKernelVersion()
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    var autoDetectedKmiText by remember { mutableStateOf<String?>(null) }
    var autoDetectUnsupported by remember { mutableStateOf(false) }

    LaunchedEffect(Unit) {
        withContext(Dispatchers.IO) {
            val release = Os.uname().release
            val detected = detectBundledKo(release)
            withContext(Dispatchers.Main) {
                autoDetectedKmiText = detected.text
                autoDetectUnsupported = detected.unsupported
            }
        }
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
                val selectedName = withContext(Dispatchers.IO) { getDisplayName(context, uri) ?: (uri.lastPathSegment ?: ".ko") }
                homeViewModel.updateSelectedKoName(selectedName)
                navigator.push(
                    Route.Flash(
                        FlashIt.FlashBoot(
                            lkm = LkmSelection.LkmUri(uri),
                            ota = false,
                            partition = null
                        )
                    )
                )
            } else {
                Toast.makeText(
                    context,
                    context.getString(R.string.install_only_support_ko_file),
                    Toast.LENGTH_SHORT
                ).show()
            }
        }
    }
    val scrollBehavior = MiuixScrollBehavior()
    val enableBlur = LocalEnableBlur.current
    val hazeState = remember { HazeState() }
    val hazeStyle = if (enableBlur) {
        HazeStyle(
            backgroundColor = colorScheme.surface,
            tint = HazeTint(colorScheme.surface.copy(0.8f))
        )
    } else {
        HazeStyle.Unspecified
    }

    val prefs = context.getSharedPreferences("settings", Context.MODE_PRIVATE)
    val checkUpdate = prefs.getBoolean("check_update", true)

    Scaffold(
        topBar = { },
        popupHost = { },
        contentWindowInsets = WindowInsets.systemBars.add(WindowInsets.displayCutout).only(WindowInsetsSides.Horizontal)
    ) { innerPadding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxHeight()
                .scrollEndHaptic()
                .overScrollVertical()
                .nestedScroll(scrollBehavior.nestedScrollConnection)
                .padding(horizontal = 12.dp)
                .let { if (enableBlur) it.hazeSource(state = hazeState) else it },
            contentPadding = innerPadding,
            overscrollEffect = null,
        ) {
            item {
                val isManager = Natives.isManager
                val ksuVersion = if (isManager) Natives.version else null
                val lkmMode = ksuVersion?.let {
                    if (kernelVersion.isGKI()) Natives.isLkmMode else null
                }
                val mainState = LocalMainPagerState.current

                Column(
                    modifier = Modifier.padding(vertical = 12.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    HomeBanner(
                        onClickSettings = { mainState.animateToPage(3) }
                    )
                    if (ksuVersion != null && !Natives.isLkmMode) {
                        WarningCard(stringResource(id = R.string.home_gki_warning))
                    }
                    if (isManager && Natives.requireNewKernel()) {
                        WarningCard(
                            stringResource(id = R.string.require_kernel_version)
                                .format(ksuVersion, Natives.MINIMAL_SUPPORTED_KERNEL),
                        )
                    }
                    if (ksuVersion != null && !rootAvailable()) {
                        WarningCard(stringResource(id = R.string.grant_root_failed))
                    }
                    StatusCard(
                        kernelVersion, ksuVersion, lkmMode,
                        onClickInstall = {
                            navigator.push(Route.Install)
                        },
                        onClickLoadLkm = {
                            selectLkmLauncher.launch("*/*")
                        },
                        onClickSuperuser = {
                            mainState.animateToPage(1)
                        },
                        onclickModule = {
                            mainState.animateToPage(2)
                        },
                        selectedKoName = homeViewModel.selectedKoDisplayName,
                        autoDetectedKmiText = autoDetectedKmiText,
                        autoDetectUnsupported = autoDetectUnsupported,
                    )

                    if (checkUpdate) {
                        UpdateCard()
                    }
                    InfoCard()
                    DeviceInfoCard()
                    DonateCard()
                    LearnMoreCard()
                }
                Spacer(Modifier.height(bottomInnerPadding))
            }
        }
    }
}

@Composable
fun UpdateCard() {
    val context = LocalContext.current
    val latestVersionInfo = LatestVersionInfo()
    val newVersion by produceState(initialValue = latestVersionInfo) {
        value = withContext(Dispatchers.IO) {
            checkNewVersion()
        }
    }

    val currentVersionCode = getManagerVersion(context).second
    val newVersionCode = newVersion.versionCode
    val newVersionUrl = newVersion.downloadUrl
    val changelog = newVersion.changelog

    val uriHandler = LocalUriHandler.current
    val title = stringResource(id = R.string.module_changelog)
    val updateText = stringResource(id = R.string.module_update)

    AnimatedVisibility(
        visible = newVersionCode > currentVersionCode,
        enter = fadeIn() + expandVertically(),
        exit = shrinkVertically() + fadeOut()
    ) {
        val updateDialog = rememberConfirmDialog(onConfirm = { uriHandler.openUri(newVersionUrl) })
        WarningCard(
            message = stringResource(id = R.string.new_version_available).format(newVersionCode),
            colorScheme.outline
        ) {
            if (changelog.isEmpty()) {
                uriHandler.openUri(newVersionUrl)
            } else {
                updateDialog.showConfirm(
                    title = title,
                    content = changelog,
                    markdown = true,
                    confirm = updateText
                )
            }
        }
    }
}

@Composable
private fun HomeBanner(
    onClickSettings: () -> Unit,
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.defaultColors(color = Color(IOS_BG)),
        insideMargin = PaddingValues(horizontal = 16.dp, vertical = 0.dp),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(top = 20.dp, bottom = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            androidx.compose.foundation.Image(
                painter = androidx.compose.ui.res.painterResource(id = R.mipmap.ic_launcher_foreground),
                contentDescription = "ApexSU",
                modifier = Modifier.size(40.dp),
            )
            Column(
                modifier = Modifier
                    .weight(1f)
                    .padding(start = 12.dp),
            ) {
                Text(
                    text = "ApexSU",
                    color = Color.White,
                    fontSize = 22.sp,
                    fontWeight = FontWeight.W700,
                )
                Text(
                    text = "Root Manager",
                    color = Color(IOS_SECONDARY),
                    fontSize = 13.sp,
                )
            }
            Card(
                onClick = onClickSettings,
                modifier = Modifier.size(36.dp),
                colors = CardDefaults.defaultColors(color = colorScheme.surfaceContainer),
                insideMargin = PaddingValues(0.dp),
                cornerRadius = 18.dp,
                showIndication = true,
                pressFeedbackType = PressFeedbackType.Sink,
            ) {
                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Icon(Icons.Rounded.Settings, contentDescription = null, tint = Color.White)
                }
            }
        }
    }
}

@Composable
private fun StatusCard(
    kernelVersion: KernelVersion,
    ksuVersion: Int?,
    lkmMode: Boolean?,
    onClickInstall: () -> Unit = {},
    onClickLoadLkm: () -> Unit = {},
    onClickSuperuser: () -> Unit = {},
    onclickModule: () -> Unit = {},
    selectedKoName: String?,
    autoDetectedKmiText: String?,
    autoDetectUnsupported: Boolean,
) {
    val reduceMotion = LocalReduceMotion.current
    Column(
        modifier = Modifier
    ) {
        when {
            ksuVersion != null -> {
                val superuserCount by produceState(initialValue = 0) {
                    value = withContext(Dispatchers.IO) { getSuperuserCount() }
                }
                val moduleCount by produceState(initialValue = 0) {
                    value = withContext(Dispatchers.IO) { getModuleCount() }
                }
                val safeMode = when {
                    Natives.isSafeMode -> " [${stringResource(id = R.string.safe_mode)}]"
                    else -> ""
                }

                val workingMode = when (lkmMode) {
                    null -> ""
                    true -> " <LKM>"
                    else -> " <GKI>"
                }

                val workingText = "${stringResource(id = R.string.home_working)}$workingMode$safeMode"

                AnimatedVisibility(
                    visible = true,
                    enter = if (reduceMotion) fadeIn(tween(0)) else fadeIn(tween(220)) + slideInVertically(
                        animationSpec = tween(220),
                        initialOffsetY = { it / 4 }
                    )
                ) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(IntrinsicSize.Min),
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                    Card(
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxHeight(),
                        colors = CardDefaults.defaultColors(color = Color(IOS_BG)),
                        onClick = {
                            onClickInstall()
                        },
                        showIndication = true,
                        pressFeedbackType = PressFeedbackType.Tilt
                    ) {
                        Box(
                            modifier = Modifier.fillMaxSize()
                        ) {
                            Box(
                                modifier = Modifier
                                    .fillMaxSize()
                                    .offset(38.dp, 45.dp),
                                contentAlignment = Alignment.BottomEnd
                            ) {
                                Icon(
                                    modifier = Modifier.size(170.dp),
                                    imageVector = Icons.Rounded.CheckCircleOutline,
                                    tint = if (isDynamicColor) {
                                        colorScheme.primary.copy(alpha = 0.8f)
                                    } else {
                                        Color(0xFF36D167)
                                    },
                                    contentDescription = null
                                )
                            }
                            Column(
                                modifier = Modifier
                                    .fillMaxSize()
                                    .padding(all = 16.dp)
                            ) {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    Box(
                                        modifier = Modifier
                                            .size(8.dp),
                                    ) {
                                        Card(
                                            modifier = Modifier.fillMaxSize(),
                                            colors = CardDefaults.defaultColors(color = Color(IOS_GREEN)),
                                            cornerRadius = 4.dp,
                                            insideMargin = PaddingValues(0.dp),
                                        ) {}
                                    }
                                    Text(
                                        modifier = Modifier.padding(start = 8.dp),
                                        text = stringResource(R.string.home_installed_id),
                                        fontSize = 17.sp,
                                        fontWeight = FontWeight.SemiBold,
                                        color = Color.White,
                                    )
                                }
                                Spacer(Modifier.height(2.dp))
                                Text(
                                    modifier = Modifier.fillMaxWidth(),
                                    text = stringResource(R.string.home_working_version, ksuVersion),
                                    fontSize = 14.sp,
                                    fontWeight = FontWeight.Medium,
                                    fontFamily = FontFamily.Monospace,
                                    color = Color(IOS_SECONDARY),
                                )
                                Text(
                                    text = "${stringResource(R.string.home_android_version)}: ${Build.VERSION.RELEASE}",
                                    fontSize = 13.sp,
                                    color = Color(IOS_SECONDARY),
                                )
                                Text(
                                    text = "${stringResource(R.string.home_selinux_status)}: ${getSELinuxStatus()}",
                                    fontSize = 13.sp,
                                    color = Color(IOS_SECONDARY),
                                )
                            }
                        }
                    }
                    Column(
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxHeight()
                    ) {
                        Card(
                            modifier = Modifier
                                .fillMaxWidth()
                                .weight(1f),
                            insideMargin = PaddingValues(16.dp),
                            onClick = { onClickSuperuser() },
                            showIndication = true,
                            pressFeedbackType = PressFeedbackType.Tilt
                        ) {
                            Column(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalAlignment = Alignment.Start
                            ) {
                                Text(
                                    modifier = Modifier.fillMaxWidth(),
                                    text = stringResource(R.string.superuser),
                                    fontWeight = FontWeight.Medium,
                                    fontSize = 15.sp,
                                    color = colorScheme.onSurfaceVariantSummary,
                                )
                                Text(
                                    modifier = Modifier.fillMaxWidth(),
                                    text = superuserCount.toString(),
                                    fontSize = 26.sp,
                                    fontWeight = FontWeight.SemiBold,
                                    color = colorScheme.onSurface,
                                )
                            }
                        }
                        Spacer(Modifier.height(12.dp))
                        Card(
                            modifier = Modifier
                                .fillMaxWidth()
                                .weight(1f),
                            insideMargin = PaddingValues(16.dp),
                            onClick = { onclickModule() },
                            showIndication = true,
                            pressFeedbackType = PressFeedbackType.Tilt
                        ) {
                            Column(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalAlignment = Alignment.Start
                            ) {
                                Text(
                                    modifier = Modifier.fillMaxWidth(),
                                    text = stringResource(R.string.module),
                                    fontWeight = FontWeight.Medium,
                                    fontSize = 15.sp,
                                    color = colorScheme.onSurfaceVariantSummary,
                                )
                                Text(
                                    modifier = Modifier.fillMaxWidth(),
                                    text = moduleCount.toString(),
                                    fontSize = 26.sp,
                                    fontWeight = FontWeight.SemiBold,
                                    color = colorScheme.onSurface,
                                )
                            }
                        }
                    }
                }
                }
            }

            kernelVersion.isGKI() -> {
                Card(
                    onClick = onClickInstall,
                    showIndication = true,
                    pressFeedbackType = PressFeedbackType.Sink,
                    colors = CardDefaults.defaultColors(color = Color(IOS_BG)),
                    cornerRadius = 12.dp,
                ) {
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(16.dp),
                        verticalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                            Icon(
                                Icons.Outlined.Warning,
                                stringResource(R.string.home_not_installed_id),
                                modifier = Modifier.size(20.dp),
                                tint = Color(IOS_ORANGE),
                            )
                            Column {
                                Text(
                                    text = stringResource(R.string.home_not_installed_id),
                                    fontSize = 17.sp,
                                    fontWeight = FontWeight.SemiBold,
                                    color = Color.White,
                                )
                                Text(
                                    text = stringResource(R.string.home_tap_to_install_id),
                                    color = Color(IOS_SECONDARY),
                                    fontSize = 13.sp,
                                )
                            }
                        }
                        Hairline()
                        LkmRow(
                            icon = Icons.Rounded.FolderOpen,
                            title = stringResource(R.string.home_pick_boot_image_id),
                            subtitle = stringResource(R.string.home_pick_boot_image_sub_id),
                            onClick = onClickInstall,
                        )
                        Hairline()
                        LkmRow(
                            icon = Icons.Rounded.Extension,
                            title = stringResource(R.string.home_use_local_lkm_id),
                            subtitle = when {
                                !selectedKoName.isNullOrEmpty() -> selectedKoName
                                autoDetectUnsupported -> stringResource(R.string.home_device_maybe_unsupported_gki_id)
                                !autoDetectedKmiText.isNullOrEmpty() -> autoDetectedKmiText
                                else -> stringResource(R.string.home_auto_pick_ko_id)
                            },
                            subtitleColor = if (autoDetectUnsupported) Color(IOS_ORANGE) else Color(IOS_SECONDARY),
                            onClick = onClickLoadLkm,
                        )
                    }
                }
            }

            else -> {
                Card(
                    onClick = {
                        onClickInstall()
                    },
                    showIndication = true,
                    pressFeedbackType = PressFeedbackType.Sink
                ) {
                    BasicComponent(
                        title = stringResource(R.string.home_unsupported),
                        summary = stringResource(R.string.home_unsupported_reason),
                        startAction = {
                            Icon(
                                Icons.Rounded.ErrorOutline,
                                stringResource(R.string.home_unsupported),
                                modifier = Modifier
                                    .padding(end = 16.dp),
                                tint = colorScheme.onBackground,
                            )
                        }
                    )
                }
            }
        }
    }
}

@Composable
private fun Hairline() {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(0.5.dp),
    ) {
        Card(
            modifier = Modifier.fillMaxSize(),
            colors = CardDefaults.defaultColors(color = Color(IOS_SEPARATOR)),
            insideMargin = PaddingValues(0.dp),
        ) {}
    }
}

@Composable
private fun LkmRow(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    title: String,
    subtitle: String,
    subtitleColor: Color = Color(IOS_SECONDARY),
    onClick: () -> Unit,
) {
    Card(
        onClick = onClick,
        colors = CardDefaults.defaultColors(color = Color.Transparent),
        insideMargin = PaddingValues(0.dp),
        showIndication = true,
        pressFeedbackType = PressFeedbackType.Sink,
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(vertical = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = Color(IOS_BLUE),
                modifier = Modifier.size(20.dp),
            )
            Column(
                modifier = Modifier
                    .weight(1f)
                    .padding(start = 10.dp),
            ) {
                Text(
                    text = title,
                    fontSize = 15.sp,
                    color = Color.White,
                    fontWeight = FontWeight.Medium,
                )
                Text(
                    text = subtitle,
                    fontSize = 12.sp,
                    color = subtitleColor,
                )
            }
            Icon(
                imageVector = Icons.Rounded.ChevronRight,
                contentDescription = null,
                tint = Color(IOS_SECONDARY),
                modifier = Modifier.size(18.dp),
            )
        }
    }
}

private data class KoDetectResult(
    val text: String?,
    val unsupported: Boolean,
)

private fun detectBundledKo(release: String): KoDetectResult {
    val androidMatch = Regex("android(\\d+)").find(release)?.groupValues?.getOrNull(1)
    val kernelMatch = Regex("(\\d+\\.\\d+)").find(release)?.groupValues?.getOrNull(1)
    val key = if (androidMatch != null && kernelMatch != null) "android$androidMatch-$kernelMatch" else null
    val map = mapOf(
        "android12-5.10" to "kernelsu-5.10.209-arm64.ko",
        "android13-5.10" to "kernelsu-5.10.209-arm64.ko",
        "android13-5.15" to "kernelsu-5.15.148-arm64.ko",
        "android14-5.15" to "kernelsu-5.15.148-arm64.ko",
        "android14-6.1" to "kernelsu-6.1.96-arm64.ko",
        "android15-6.6" to "kernelsu-6.6.35-arm64.ko",
        "android16-6.12" to "kernelsu-6.12.6-arm64.ko",
    )
    val matched = key?.let(map::get)
    return if (key != null && matched != null) {
        KoDetectResult("Terdeteksi: $key → ${matched.removePrefix("kernelsu-")}", unsupported = false)
    } else {
        KoDetectResult(null, unsupported = true)
    }
}

private fun getDisplayName(context: Context, uri: Uri): String? {
    return try {
        context.contentResolver.query(
            uri,
            arrayOf(OpenableColumns.DISPLAY_NAME),
            null,
            null,
            null
        )?.use { cursor ->
            val index = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
            if (index != -1 && cursor.moveToFirst()) cursor.getString(index) else null
        }
    } catch (_: Throwable) {
        null
    }
}

@Composable
fun RebootDropdownItem(
    @StringRes id: Int, reason: String = "",
    showTopPopup: MutableState<Boolean>,
    optionSize: Int,
    index: Int,
) {
    DropdownItem(
        text = stringResource(id),
        optionSize = optionSize,
        onSelectedIndexChange = {
            reboot(reason)
            showTopPopup.value = false
        },
        index = index
    )
}

@Composable
fun WarningCard(
    message: String,
    color: Color? = null,
    onClick: (() -> Unit)? = null,
) {
    Card(
        onClick = {
            onClick?.invoke()
        },
        colors = CardDefaults.defaultColors(
            color = color ?: when {
                isDynamicColor -> colorScheme.errorContainer
                isInDarkTheme() -> Color(0XFF310808)
                else -> Color(0xFFF8E2E2)
            }
        ),
        showIndication = onClick != null,
        pressFeedbackType = PressFeedbackType.Tilt
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            Text(
                text = message,
                color = if (isDynamicColor) colorScheme.onErrorContainer else Color(0xFFF72727),
                fontSize = 14.sp
            )
        }
    }
}

@Composable
fun LearnMoreCard() {
    val uriHandler = LocalUriHandler.current
    val url = stringResource(R.string.home_learn_kernelsu_url)

    Card(
        modifier = Modifier
            .fillMaxWidth(),
    ) {
        BasicComponent(
            title = stringResource(R.string.home_learn_kernelsu),
            summary = stringResource(R.string.home_click_to_learn_kernelsu),
            endActions = {
                Icon(
                    imageVector = MiuixIcons.Link,
                    tint = colorScheme.onSurface,
                    contentDescription = null
                )
            },
            onClick = {
                uriHandler.openUri(url)
            }
        )
    }
}

@Composable
fun DonateCard() {
    val uriHandler = LocalUriHandler.current

    Card(
        modifier = Modifier
            .fillMaxWidth(),
    ) {
        BasicComponent(
            title = stringResource(R.string.home_support_title),
            summary = stringResource(R.string.home_support_content),
            endActions = {
                Icon(
                    imageVector = MiuixIcons.Link,
                    tint = colorScheme.onSurface,
                    contentDescription = null
                )
            },
            onClick = {
                uriHandler.openUri("https://github.com/qrjhamron/ApexSU")
            },
            insideMargin = PaddingValues(18.dp)
        )
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
private fun DeviceInfoCard() {
    val context = LocalContext.current
    @Composable
    fun InfoText(
        title: String,
        content: String,
        bottomPadding: Dp = 24.dp
    ) {
        Column(
            modifier = Modifier.combinedClickable(
                onClick = {},
                onLongClick = {
                    copyToClipboard(context, title, content)
                }
            )
        ) {
            Text(
                text = title,
                fontSize = MiuixTheme.textStyles.headline1.fontSize,
                fontWeight = FontWeight.Medium,
                color = colorScheme.onSurface
            )
            Text(
                text = content,
                fontSize = MiuixTheme.textStyles.body2.fontSize,
                color = colorScheme.onSurfaceVariantSummary,
                modifier = Modifier.padding(top = 2.dp, bottom = bottomPadding)
            )
        }
    }
    Card {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            InfoText(
                title = stringResource(R.string.home_device_model),
                content = Build.MODEL
            )
            InfoText(
                title = stringResource(R.string.home_device_brand),
                content = Build.BRAND
            )
            InfoText(
                title = stringResource(R.string.home_device_manufacturer),
                content = Build.MANUFACTURER
            )
            InfoText(
                title = stringResource(R.string.home_device_codename),
                content = Build.DEVICE
            )
            InfoText(
                title = stringResource(R.string.home_android_version),
                content = Build.VERSION.RELEASE
            )
            InfoText(
                title = stringResource(R.string.home_api_level),
                content = Build.VERSION.SDK_INT.toString()
            )
            InfoText(
                title = stringResource(R.string.home_security_patch),
                content = Build.VERSION.SECURITY_PATCH
            )
            InfoText(
                title = stringResource(R.string.home_cpu_architecture),
                content = Build.SUPPORTED_ABIS.joinToString(", "),
                bottomPadding = 0.dp
            )
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
private fun InfoCard() {
    val context = LocalContext.current
    @Composable
    fun InfoText(
        title: String,
        content: String,
        bottomPadding: Dp = 24.dp
    ) {
        Column(
            modifier = Modifier.combinedClickable(
                onClick = {},
                onLongClick = {
                    copyToClipboard(context, title, content)
                }
            )
        ) {
            Text(
                text = title,
                fontSize = MiuixTheme.textStyles.headline1.fontSize,
                fontWeight = FontWeight.Medium,
                color = colorScheme.onSurface
            )
            Text(
                text = content,
                fontSize = MiuixTheme.textStyles.body2.fontSize,
                color = colorScheme.onSurfaceVariantSummary,
                modifier = Modifier.padding(top = 2.dp, bottom = bottomPadding)
            )
        }
    }
    Card {
        val uname = Os.uname()
        val managerVersion = getManagerVersion(LocalContext.current)
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            InfoText(
                title = stringResource(R.string.home_kernel),
                content = uname.release
            )
            InfoText(
                title = stringResource(R.string.home_manager_version),
                content = "${managerVersion.first} (${managerVersion.second})"
            )
            InfoText(
                title = stringResource(R.string.home_fingerprint),
                content = Build.FINGERPRINT
            )
            InfoText(
                title = stringResource(R.string.home_selinux_status),
                content = getSELinuxStatus(),
                bottomPadding = 0.dp
            )
        }
    }
}

private fun copyToClipboard(context: Context, label: String, text: String) {
    val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    clipboard.setPrimaryClip(ClipData.newPlainText(label, text))
    Toast.makeText(
        context,
        context.getString(R.string.copied_to_clipboard, label),
        Toast.LENGTH_SHORT
    ).show()
}

private fun isKoFile(context: Context, uri: Uri): Boolean {
    val segment = uri.lastPathSegment ?: ""
    if (segment.endsWith(".ko", ignoreCase = true)) return true

    return try {
        context.contentResolver.query(
            uri,
            arrayOf(OpenableColumns.DISPLAY_NAME),
            null,
            null,
            null
        )?.use { cursor ->
            val index = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
            if (index != -1 && cursor.moveToFirst()) {
                cursor.getString(index)?.endsWith(".ko", ignoreCase = true) == true
            } else {
                false
            }
        } ?: false
    } catch (_: Throwable) {
        false
    }
}

fun getManagerVersion(context: Context): Pair<String, Long> {
    val packageInfo = context.packageManager.getPackageInfo(context.packageName, 0)!!
    val versionCode = PackageInfoCompat.getLongVersionCode(packageInfo)
    return Pair(packageInfo.versionName!!, versionCode)
}
