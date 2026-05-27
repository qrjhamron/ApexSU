package com.qrj.apexsu.ui.screen

import android.content.Context
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
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Add
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.qrj.apexsu.ui.navigation3.Navigator
import com.qrj.apexsu.ui.navigation3.Route
import com.qrj.apexsu.ui.theme.DarkApexColors
import com.qrj.apexsu.ui.viewmodel.ModuleViewModel
import top.yukonga.miuix.kmp.basic.Icon
import top.yukonga.miuix.kmp.basic.Switch
import top.yukonga.miuix.kmp.basic.Text

@Composable
fun ModulePager(navigator: Navigator, bottomInnerPadding: Dp) {
    val viewModel = viewModel<ModuleViewModel>()
    val context = LocalContext.current
    var initialized by rememberSaveable { mutableStateOf(false) }

    LaunchedEffect(Unit) {
        if (!initialized || viewModel.moduleList.isEmpty()) {
            val prefs = context.getSharedPreferences("settings", Context.MODE_PRIVATE)
            viewModel.checkModuleUpdate = prefs.getBoolean("module_check_update", true)
            viewModel.fetchModuleList(checkUpdate = true)
            initialized = true
        }
    }

    Box(Modifier.fillMaxSize().background(DarkApexColors.background)) {
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = androidx.compose.foundation.layout.PaddingValues(horizontal = 16.dp, vertical = 20.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            item { Text("Modules", color = Color.White, fontSize = 28.sp, fontWeight = FontWeight.Bold) }
            items(viewModel.moduleList) { module ->
                Column(
                    Modifier.fillMaxWidth().background(DarkApexColors.surfaceL2, RoundedCornerShape(12.dp)).padding(16.dp),
                    verticalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text(module.name.ifBlank { module.id }, color = Color.White, fontSize = 17.sp, fontWeight = FontWeight.SemiBold, modifier = Modifier.weight(1f))
                        Switch(checked = module.enabled, onCheckedChange = {})
                    }
                    Text("${module.version} • ${module.author}", color = DarkApexColors.textSecondary, fontSize = 12.sp)
                    Text(module.description, color = DarkApexColors.textTertiary, fontSize = 13.sp, maxLines = 2, overflow = TextOverflow.Ellipsis)
                    Row(horizontalArrangement = Arrangement.spacedBy(6.dp), modifier = Modifier.fillMaxWidth()) {
                        if (module.enabled) Chip("Enabled", DarkApexColors.green)
                        if (module.update) Chip("Update", DarkApexColors.orange)
                        if (module.remove) Chip("Error", DarkApexColors.red)
                    }
                }
            }
            item { Spacer(Modifier.height(bottomInnerPadding + 72.dp)) }
        }

        Box(
            modifier = Modifier
                .align(Alignment.BottomEnd)
                .padding(16.dp)
                .size(56.dp)
                .background(DarkApexColors.blue, CircleShape)
                .clickable(indication = null, interactionSource = androidx.compose.runtime.remember { MutableInteractionSource() }) { navigator.push(Route.Install) },
            contentAlignment = Alignment.Center,
        ) {
            Icon(Icons.Rounded.Add, contentDescription = null, tint = Color.White)
        }
    }
}

@Composable
private fun Chip(label: String, color: Color) {
    Text(
        text = label,
        color = Color.White,
        fontSize = 12.sp,
        modifier = Modifier.background(color, RoundedCornerShape(8.dp)).padding(horizontal = 8.dp, vertical = 4.dp),
    )
}
