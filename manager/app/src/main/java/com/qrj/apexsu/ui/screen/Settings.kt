package com.qrj.apexsu.ui.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.DeleteForever
import androidx.compose.material.icons.rounded.FolderDelete
import androidx.compose.ui.graphics.vector.ImageVector
import com.qrj.apexsu.BuildConfig
import com.qrj.apexsu.R
import com.qrj.apexsu.getKernelVersion
import com.qrj.apexsu.ui.navigation3.Navigator
import com.qrj.apexsu.ui.theme.DarkApexColors
import top.yukonga.miuix.kmp.basic.Switch
import top.yukonga.miuix.kmp.basic.Text

enum class UninstallType(
    val title: Int,
    val message: Int,
    val icon: ImageVector,
) {
    NONE(0, 0, Icons.Rounded.Delete),
    TEMPORARY(R.string.settings_uninstall_temporary, R.string.settings_uninstall_temporary_message, Icons.Rounded.Delete),
    PERMANENT(R.string.settings_uninstall_permanent, R.string.settings_uninstall_permanent_message, Icons.Rounded.DeleteForever),
    RESTORE_STOCK_IMAGE(R.string.settings_restore_stock_image, R.string.settings_restore_stock_image_message, Icons.Rounded.FolderDelete),
}

@Composable
fun SettingPager(navigator: Navigator, bottomInnerPadding: Dp) {
    val uri = LocalUriHandler.current

    LazyColumn(
        modifier = Modifier.fillMaxSize().background(DarkApexColors.background),
        contentPadding = androidx.compose.foundation.layout.PaddingValues(horizontal = 16.dp, vertical = 20.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        item { Text("Settings", color = Color.White, fontSize = 28.sp, fontWeight = FontWeight.Bold) }

        item { SectionHeader("GENERAL") }
        item {
            SectionCard {
                SettingRow("Language", "System", chevron = true) {}
                Separator()
                SettingRow("Theme", "Dark", chevron = false) {}
                Separator()
                SettingRow("Check for Updates", "", chevron = true) {}
            }
        }

        item { SectionHeader("ROOT") }
        item {
            SectionCard {
                ToggleRow("Umount Modules")
                Separator()
                ToggleRow("Always ADB Root")
            }
        }

        item { SectionHeader("ABOUT") }
        item {
            SectionCard {
                SettingRow("Version", "${BuildConfig.VERSION_NAME}", chevron = false) {}
                Separator()
                SettingRow("Kernel Module", getKernelVersion().toString(), chevron = false) {}
                Separator()
                SettingRow("GitHub", "qrjhamron/ApexSU", chevron = true) {
                    uri.openUri("https://github.com/qrjhamron/ApexSU")
                }
                Separator()
                SettingRow("Report Issue", "", chevron = true) {
                    uri.openUri("https://github.com/qrjhamron/ApexSU/issues")
                }
            }
        }
        item { Spacer(Modifier.height(bottomInnerPadding)) }
    }
}

@Composable
private fun SectionHeader(text: String) {
    Text(text = text, color = DarkApexColors.textSecondary, fontSize = 13.sp, modifier = Modifier.padding(top = 8.dp))
}

@Composable
private fun SectionCard(content: @Composable () -> Unit) {
    Column(Modifier.fillMaxWidth().background(DarkApexColors.surfaceL2, RoundedCornerShape(12.dp))) { content() }
}

@Composable
private fun SettingRow(title: String, value: String, chevron: Boolean, onClick: () -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().height(54.dp).clickable(indication = null, interactionSource = androidx.compose.runtime.remember { MutableInteractionSource() }, onClick = onClick).padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(title, color = Color.White, fontSize = 17.sp, modifier = Modifier.weight(1f))
        if (value.isNotBlank()) Text(value, color = DarkApexColors.textSecondary, fontSize = 13.sp)
        if (chevron) Text("  ›", color = DarkApexColors.textSecondary, fontSize = 16.sp)
    }
}

@Composable
private fun ToggleRow(title: String) {
    Row(Modifier.fillMaxWidth().height(54.dp).padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically) {
        Text(title, color = Color.White, fontSize = 17.sp, modifier = Modifier.weight(1f))
        Switch(checked = false, onCheckedChange = {})
    }
}

@Composable
private fun Separator() {
    Spacer(Modifier.fillMaxWidth().padding(start = 16.dp).height(0.5.dp).background(DarkApexColors.separator))
}
