package com.qrj.apexsu.ui.screen

import android.os.Build
import android.content.Context
import android.content.pm.PackageInfo
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.spring
import androidx.compose.animation.fadeIn
import androidx.compose.animation.slideInVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.qrj.apexsu.BuildConfig
import com.qrj.apexsu.Natives
import com.qrj.apexsu.R
import com.qrj.apexsu.getKernelVersion
import com.qrj.apexsu.ui.LocalMainPagerState
import com.qrj.apexsu.ui.navigation3.Navigator
import com.qrj.apexsu.ui.theme.DarkApexColors
import com.qrj.apexsu.ui.util.getModuleCount
import com.qrj.apexsu.ui.util.getSELinuxStatus
import com.qrj.apexsu.ui.util.getSuperuserCount
import com.qrj.apexsu.ui.util.rootAvailable
import com.qrj.apexsu.ui.util.reboot
import top.yukonga.miuix.kmp.basic.Icon
import top.yukonga.miuix.kmp.basic.Text

@Composable
fun HomePager(navigator: Navigator, bottomInnerPadding: Dp) {
    val main = LocalMainPagerState.current
    val kernel = getKernelVersion().toString()
    val active = Natives.isManager && rootAvailable()

    LazyColumn(
        modifier = Modifier.fillMaxSize().background(DarkApexColors.background),
        contentPadding = androidx.compose.foundation.layout.PaddingValues(horizontal = 16.dp, vertical = 20.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        item {
            Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                Column(Modifier.weight(1f)) {
                    Text("ApexSU", color = Color.White, fontSize = 28.sp, fontWeight = FontWeight.Bold)
                    Text("Root Manager", color = DarkApexColors.textSecondary, fontSize = 13.sp)
                }
                Box(
                    Modifier.size(36.dp).background(DarkApexColors.surfaceL2, CircleShape).clickable(indication = null, interactionSource = remember { MutableInteractionSource() }) {
                        main.animateToPage(3)
                    },
                    contentAlignment = Alignment.Center,
                ) { Icon(Icons.Rounded.Settings, contentDescription = null, tint = Color.White) }
            }
        }
        item {
            AnimatedVisibility(
                visible = true,
                enter = fadeIn(spring(dampingRatio = 0.8f, stiffness = 400f)) + slideInVertically(spring(dampingRatio = 0.8f, stiffness = 400f)) { it / 4 },
            ) {
                Row(
                    Modifier.fillMaxWidth().background(DarkApexColors.surfaceL1, RoundedCornerShape(12.dp)).padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Box(Modifier.size(34.dp).background(if (active) DarkApexColors.green else DarkApexColors.surfaceL3, CircleShape), contentAlignment = Alignment.Center) {
                        Icon(if (active) Icons.Rounded.Check else Icons.Rounded.Close, contentDescription = null, tint = Color.White)
                    }
                    Column(Modifier.padding(start = 12.dp)) {
                        Text(if (active) "Active" else stringResource(R.string.home_not_installed), color = Color.White, fontSize = 17.sp, fontWeight = FontWeight.SemiBold)
                        Text("Kernel: $kernel", color = DarkApexColors.textSecondary, fontSize = 12.sp)
                        Text("Android: ${Build.VERSION.RELEASE}", color = DarkApexColors.textTertiary, fontSize = 12.sp)
                    }
                }
            }
        }
        item {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                StatCard("${getSuperuserCount()}", "App count", Modifier.weight(1f))
                StatCard("${getModuleCount()}", "Module count", Modifier.weight(1f))
                StatCard(getSELinuxStatus(), "SELinux", Modifier.weight(1f))
            }
        }
        item {
            Text("QUICK ACTIONS", color = DarkApexColors.textSecondary, fontSize = 13.sp, modifier = Modifier.padding(top = 8.dp))
            Column(Modifier.fillMaxWidth().background(DarkApexColors.surfaceL2, RoundedCornerShape(12.dp))) {
                ActionRow("SuperUser") { main.animateToPage(1) }
                Box(Modifier.fillMaxWidth().height(0.5.dp).background(DarkApexColors.separator))
                ActionRow("Modules") { main.animateToPage(2) }
            }
        }
        item {
            Spacer(Modifier.height(8.dp))
            Text(
                text = "v${BuildConfig.VERSION_NAME} (${BuildConfig.VERSION_CODE})",
                color = DarkApexColors.textTertiary,
                fontSize = 12.sp,
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(Modifier.height(bottomInnerPadding))
        }
    }
}

@Composable
private fun StatCard(value: String, label: String, modifier: Modifier = Modifier) {
    Column(
        modifier = modifier.background(DarkApexColors.surfaceL2, RoundedCornerShape(12.dp)).padding(12.dp),
        verticalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        Text(value, color = Color.White, fontSize = 22.sp, fontWeight = FontWeight.SemiBold)
        Text(label, color = DarkApexColors.textSecondary, fontSize = 12.sp)
    }
}

@Composable
private fun ActionRow(title: String, onClick: () -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().height(54.dp).clickable(indication = null, interactionSource = remember { MutableInteractionSource() }, onClick = onClick).padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(title, color = Color.White, fontSize = 17.sp, modifier = Modifier.weight(1f))
        Text("›", color = DarkApexColors.textSecondary, fontSize = 18.sp)
    }
}

@Composable
fun RebootDropdownItem(
    id: Int,
    reason: String,
    showTopPopup: androidx.compose.runtime.MutableState<Boolean>,
    optionSize: Int,
    index: Int,
) {
    com.qrj.apexsu.ui.component.DropdownItem(
        text = stringResource(id),
        optionSize = optionSize,
        onSelectedIndexChange = {
            reboot(reason)
            showTopPopup.value = false
        },
        index = index,
    )
}

fun getManagerVersion(context: Context): Pair<String, Long> {
    val info: PackageInfo = context.packageManager.getPackageInfo(context.packageName, 0)
    val code = androidx.core.content.pm.PackageInfoCompat.getLongVersionCode(info)
    return (info.versionName ?: "unknown") to code
}
