package com.qrj.apexsu.ui.screen

import android.net.Uri
import android.os.Environment
import android.os.Parcelable
import android.os.SystemClock
import android.widget.Toast
import androidx.compose.foundation.background
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.add
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.calculateEndPadding
import androidx.compose.foundation.layout.calculateStartPadding
import androidx.compose.foundation.layout.captionBar
import androidx.compose.foundation.layout.displayCutout
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.key
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.dropUnlessResumed
import dev.chrisbanes.haze.HazeState
import dev.chrisbanes.haze.HazeStyle
import dev.chrisbanes.haze.HazeTint
import dev.chrisbanes.haze.hazeEffect
import dev.chrisbanes.haze.hazeSource
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.parcelize.Parcelize
import com.qrj.apexsu.R
import com.qrj.apexsu.ui.component.KeyEventBlocker
import com.qrj.apexsu.ui.component.rememberConfirmDialog
import com.qrj.apexsu.ui.navigation3.LocalNavigator
import com.qrj.apexsu.ui.theme.LocalEnableBlur
import com.qrj.apexsu.ui.util.FlashResult
import com.qrj.apexsu.ui.util.LkmSelection
import com.qrj.apexsu.ui.util.flashModule
import com.qrj.apexsu.ui.util.installBoot
import com.qrj.apexsu.ui.util.reboot
import com.qrj.apexsu.ui.util.restoreBoot
import com.qrj.apexsu.ui.util.uninstallPermanently
import top.yukonga.miuix.kmp.basic.FloatingActionButton
import top.yukonga.miuix.kmp.basic.Icon
import top.yukonga.miuix.kmp.basic.IconButton
import top.yukonga.miuix.kmp.basic.Scaffold
import top.yukonga.miuix.kmp.basic.SmallTopAppBar
import top.yukonga.miuix.kmp.basic.Text
import top.yukonga.miuix.kmp.icon.MiuixIcons
import top.yukonga.miuix.kmp.icon.extended.Back
import top.yukonga.miuix.kmp.icon.extended.Share
import top.yukonga.miuix.kmp.theme.MiuixTheme.colorScheme
import top.yukonga.miuix.kmp.utils.scrollEndHaptic
import java.io.File
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

/**
 * @author weishu
 * @date 2023/1/1.
 */

enum class FlashingStatus {
    FLASHING,
    SUCCESS,
    FAILED
}

// Lets you flash modules sequentially when mutiple zipUris are selected
fun flashModulesSequentially(
    uris: List<Uri>,
    onStdout: (String) -> Unit,
    onStderr: (String) -> Unit
): FlashResult {
    for (uri in uris) {
        flashModule(uri, onStdout, onStderr).apply {
            if (code != 0) {
                return FlashResult(code, err, showReboot)
            }
        }
    }
    return FlashResult(0, "", true)
}

@Composable
fun FlashScreen(
    flashIt: FlashIt
) {
    val navigator = LocalNavigator.current
    var text by rememberSaveable { mutableStateOf("") }
    val logContent = rememberSaveable { StringBuilder() }
    var showFloatAction by rememberSaveable { mutableStateOf(false) }

    val context = LocalContext.current
    val enableBlur = LocalEnableBlur.current
    val scope = rememberCoroutineScope()
    val scrollState = rememberScrollState()
    var confirmed by rememberSaveable { mutableStateOf(false) }
    var flashing by rememberSaveable {
        mutableStateOf(FlashingStatus.FLASHING)
    }
    val startedAt = remember { SystemClock.elapsedRealtime() }
    val elapsed by produceState(initialValue = "00:00", key1 = flashing) {
        while (flashing == FlashingStatus.FLASHING) {
            val seconds = ((SystemClock.elapsedRealtime() - startedAt) / 1000).toInt()
            value = "%02d:%02d".format(seconds / 60, seconds % 60)
            delay(1000)
        }
    }
    val hazeState = remember { HazeState() }
    val hazeStyle = if (enableBlur) {
        HazeStyle(
            backgroundColor = colorScheme.surface,
            tint = HazeTint(colorScheme.surface.copy(0.8f))
        )
    } else {
        HazeStyle.Unspecified
    }

    val confirmDialog = rememberConfirmDialog(
        onConfirm = { confirmed = true },
        onDismiss = { navigator.pop() }
    )
    val flashTarget = when (flashIt) {
        is FlashIt.FlashBoot -> stringResource(if (flashIt.ota) R.string.flash_target_inactive_slot else R.string.flash_target_current_slot)
        is FlashIt.FlashModules -> stringResource(R.string.module)
        FlashIt.FlashRestore -> stringResource(R.string.settings_restore_stock_image)
        FlashIt.FlashUninstall -> stringResource(R.string.settings_uninstall)
    }
    val backupStatus = if (flashIt is FlashIt.FlashBoot) {
        stringResource(R.string.flash_backup_required)
    } else {
        stringResource(R.string.flash_backup_not_required)
    }
    LaunchedEffect(Unit) {
        confirmDialog.showConfirm(
            title = context.getString(R.string.flash_confirm_title),
            content = context.getString(
                R.string.flash_confirm_content,
                flashTarget,
                backupStatus,
                context.getString(R.string.flash_estimated_time_value)
            )
        )
    }

    LaunchedEffect(confirmed) {
        if (!confirmed || text.isNotEmpty()) {
            return@LaunchedEffect
        }
        val output = Channel<String>(Channel.UNLIMITED)
        val collector = launch {
            for (line in output) {
                val next = "$line\n"
                if (next.startsWith("[H[J")) { // clear command
                    text = next.substring(6)
                } else {
                    text += next
                }
                logContent.append(line).append("\n")
            }
        }
        val result = withContext(Dispatchers.IO) {
            flashIt(flashIt, onStdout = {
                output.trySend(it)
            }, onStderr = {
                output.trySend(it)
            })
        }
        output.close()
        collector.join()
        if (result.code != 0) {
            text += "Error code: ${result.code}.\n ${result.err} Please save and check the log.\n"
        }
        if (result.showReboot) {
            text += "\n\n\n"
            showFloatAction = true
        }
        flashing = if (result.code == 0) FlashingStatus.SUCCESS else FlashingStatus.FAILED
    }

    Scaffold(
        topBar = {
            TopBar(
                flashing,
                onBack = dropUnlessResumed { navigator.pop() },
                onSave = {
                    scope.launch {
                        val format = SimpleDateFormat("yyyy-MM-dd-HH-mm-ss", Locale.getDefault())
                        val date = format.format(Date())
                        val file = File(
                            Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS),
                            "ApexSU_install_log_${date}.log"
                        )
                        file.writeText(logContent.toString())
                        Toast.makeText(context, context.getString(R.string.log_saved_to, file.absolutePath), Toast.LENGTH_SHORT).show()
                    }
                },
                hazeState = hazeState,
                hazeStyle = hazeStyle,
                enableBlur = enableBlur,
            )
        },
        floatingActionButton = {
            if (showFloatAction) {
                val reboot = stringResource(id = R.string.reboot)
                FloatingActionButton(
                    modifier = Modifier
                        .padding(
                            bottom = WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding() +
                                    WindowInsets.captionBar.asPaddingValues().calculateBottomPadding() + 20.dp,
                            end = 20.dp
                        )
                        .border(0.05.dp, colorScheme.outline.copy(alpha = 0.5f), CircleShape),
                    onClick = {
                        scope.launch {
                            withContext(Dispatchers.IO) {
                                reboot()
                            }
                        }
                    },
                    shadowElevation = 0.dp,
                    content = {
                        Icon(
                            Icons.Rounded.Refresh,
                            reboot,
                            Modifier.size(40.dp),
                            tint = Color.White
                        )
                    },
                )
            }
        },
        popupHost = { },
        contentWindowInsets = WindowInsets.systemBars.add(WindowInsets.displayCutout).only(WindowInsetsSides.Horizontal)
    ) { innerPadding ->
        val layoutDirection = LocalLayoutDirection.current
        KeyEventBlocker {
            it.key == Key.VolumeDown || it.key == Key.VolumeUp
        }

        Column(
            modifier = Modifier
                .fillMaxSize(1f)
                .scrollEndHaptic()
                .let { if (enableBlur) it.hazeSource(state = hazeState) else it }
                .padding(
                    start = innerPadding.calculateStartPadding(layoutDirection),
                    end = innerPadding.calculateEndPadding(layoutDirection),
                )
                .verticalScroll(scrollState),
        ) {
            LaunchedEffect(text) {
                scrollState.animateScrollTo(scrollState.maxValue)
            }
            Spacer(Modifier.height(innerPadding.calculateTopPadding()))
            Text(
                modifier = Modifier.padding(8.dp),
                text = stringResource(R.string.flash_elapsed, elapsed),
                color = colorScheme.onSurfaceVariantSummary,
                fontSize = 12.sp,
                fontFamily = FontFamily.Monospace,
            )
            TerminalLogText(
                modifier = Modifier
                    .padding(8.dp)
                    .fillMaxWidth()
                    .background(Color(0xFF07111F))
                    .padding(12.dp),
                text = text.ifBlank { stringResource(R.string.flash_terminal_label) },
            )
            Spacer(
                Modifier.height(
                    12.dp + WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding() +
                            WindowInsets.captionBar.asPaddingValues().calculateBottomPadding()
                )
            )
        }
    }
}

@Composable
private fun TerminalLogText(
    modifier: Modifier = Modifier,
    text: String
) {
    val infoColor = Color(0xFFB7C0CC)
    val successColor = Color(0xFF63E6A5)
    val errorColor = Color(0xFFFF6B6B)
    val annotated = buildAnnotatedString {
        text.lineSequence().forEach { line ->
            val color = when {
                line.contains("error", ignoreCase = true) ||
                        line.contains("fail", ignoreCase = true) ||
                        line.contains("abort", ignoreCase = true) -> errorColor
                line.contains("success", ignoreCase = true) ||
                        line.contains("done", ignoreCase = true) ||
                        line.contains("complete", ignoreCase = true) -> successColor
                else -> infoColor
            }
            pushStyle(SpanStyle(color = color))
            append(line)
            append('\n')
            pop()
        }
    }
    BasicText(
        modifier = modifier,
        text = annotated,
        style = TextStyle(
            color = infoColor,
            fontFamily = FontFamily.Monospace,
            fontSize = 12.sp,
            lineHeight = 18.sp
        )
    )
}

@Parcelize
sealed class FlashIt : Parcelable {
    @Parcelize
    data class FlashBoot(
        val boot: Uri? = null,
        val lkm: LkmSelection,
        val ota: Boolean,
        val partition: String? = null
    ) : FlashIt()

    @Parcelize
    data class FlashModules(val uris: List<Uri>) : FlashIt()

    @Parcelize
    data object FlashRestore : FlashIt()

    @Parcelize
    data object FlashUninstall : FlashIt()
}

fun flashIt(
    flashIt: FlashIt,
    onStdout: (String) -> Unit,
    onStderr: (String) -> Unit
): FlashResult {
    return when (flashIt) {
        is FlashIt.FlashBoot -> installBoot(
            flashIt.boot,
            flashIt.lkm,
            flashIt.ota,
            flashIt.partition,
            onStdout,
            onStderr
        )

        is FlashIt.FlashModules -> {
            flashModulesSequentially(flashIt.uris, onStdout, onStderr)
        }

        FlashIt.FlashRestore -> restoreBoot(onStdout, onStderr)

        FlashIt.FlashUninstall -> uninstallPermanently(onStdout, onStderr)
    }
}

@Composable
private fun TopBar(
    status: FlashingStatus,
    onBack: () -> Unit = {},
    onSave: () -> Unit = {},
    hazeState: HazeState,
    hazeStyle: HazeStyle,
    enableBlur: Boolean
) {
    SmallTopAppBar(
        modifier = if (enableBlur) {
            Modifier.hazeEffect(hazeState) {
                style = hazeStyle
                blurRadius = 30.dp
                noiseFactor = 0f
            }
        } else {
            Modifier
        },
        title = stringResource(
            when (status) {
                FlashingStatus.FLASHING -> R.string.flashing
                FlashingStatus.SUCCESS -> R.string.flash_success
                FlashingStatus.FAILED -> R.string.flash_failed
            }
        ),
        color = if (enableBlur) Color.Transparent else colorScheme.surface,
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
                    contentDescription = null,
                    tint = colorScheme.onBackground
                )
            }
        },
        actions = {
            IconButton(
                modifier = Modifier.padding(end = 16.dp),
                onClick = onSave
            ) {
                Icon(
                    imageVector = MiuixIcons.Share,
                    contentDescription = stringResource(id = R.string.save_log),
                    tint = colorScheme.onBackground
                )
            }
        },
    )
}
