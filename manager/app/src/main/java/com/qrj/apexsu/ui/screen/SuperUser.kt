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
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.content.edit
import androidx.lifecycle.viewmodel.compose.viewModel
import com.qrj.apexsu.R
import com.qrj.apexsu.ui.component.AppIconImage
import com.qrj.apexsu.ui.navigation3.Navigator
import com.qrj.apexsu.ui.navigation3.Route
import com.qrj.apexsu.ui.theme.DarkApexColors
import com.qrj.apexsu.ui.viewmodel.SuperUserViewModel
import top.yukonga.miuix.kmp.basic.Icon
import top.yukonga.miuix.kmp.basic.Text

@Composable
fun SuperUserPager(navigator: Navigator, bottomInnerPadding: Dp) {
    val viewModel = viewModel<SuperUserViewModel>()
    val context = LocalContext.current
    var query by rememberSaveable { mutableStateOf("") }
    var isInitialized by rememberSaveable { mutableStateOf(false) }

    LaunchedEffect(Unit) {
        if (!isInitialized || viewModel.appList.value.isEmpty()) {
            val prefs = context.getSharedPreferences("settings", Context.MODE_PRIVATE)
            viewModel.showSystemApps = prefs.getBoolean("show_system_apps", false)
            viewModel.loadAppList()
            isInitialized = true
        }
    }

    val apps = viewModel.appList.value.filter {
        query.isBlank() || it.label.contains(query, true) || it.packageName.contains(query, true)
    }

    LazyColumn(
        modifier = Modifier.fillMaxSize().background(DarkApexColors.background),
        contentPadding = androidx.compose.foundation.layout.PaddingValues(horizontal = 16.dp, vertical = 20.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        item {
            Text("SuperUser", color = Color.White, fontSize = 28.sp, fontWeight = FontWeight.Bold)
        }
        item {
            Row(
                Modifier.fillMaxWidth().height(44.dp).background(DarkApexColors.surfaceL2, RoundedCornerShape(10.dp)).padding(horizontal = 12.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text("⌕", color = DarkApexColors.textTertiary, fontSize = 16.sp)
                androidx.compose.foundation.text.BasicTextField(
                    value = query,
                    onValueChange = { query = it },
                    modifier = Modifier.weight(1f).padding(start = 8.dp),
                    singleLine = true,
                    textStyle = androidx.compose.ui.text.TextStyle(color = Color.White, fontSize = 16.sp),
                    decorationBox = { inner -> if (query.isEmpty()) Text("Search apps", color = DarkApexColors.textTertiary, fontSize = 16.sp); inner() },
                )
                if (query.isNotEmpty()) Text("Clear", color = DarkApexColors.blue, fontSize = 13.sp, modifier = Modifier.clickable(indication = null, interactionSource = remember { MutableInteractionSource() }) { query = "" })
            }
        }

        if (apps.isEmpty()) {
            item {
                Column(
                    modifier = Modifier.fillParentMaxSize().padding(bottom = bottomInnerPadding),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.Center,
                ) {
                    Text("🛡", color = DarkApexColors.textSecondary, fontSize = 64.sp)
                    Text("No apps granted root", color = DarkApexColors.textSecondary, fontSize = 17.sp, fontWeight = FontWeight.SemiBold)
                    Text("Apps that request root will appear here", color = DarkApexColors.textTertiary, fontSize = 16.sp)
                }
            }
        } else {
            item {
                Column(Modifier.fillMaxWidth().background(DarkApexColors.surfaceL2, RoundedCornerShape(12.dp))) {
                    apps.forEachIndexed { index, app ->
                        Row(
                            Modifier.fillMaxWidth().height(60.dp).clickable(indication = null, interactionSource = remember { MutableInteractionSource() }) {
                                navigator.push(Route.AppProfile(app.uid, app.packageName))
                                viewModel.markNeedRefresh()
                            }.padding(horizontal = 12.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            AppIconImage(modifier = Modifier.size(36.dp), packageInfo = app.packageInfo, label = app.label)
                            Column(Modifier.weight(1f).padding(start = 10.dp)) {
                                Text(app.label, color = Color.White, fontSize = 17.sp)
                                Text(app.packageName, color = DarkApexColors.textSecondary, fontSize = 12.sp)
                                Text("Never", color = DarkApexColors.textTertiary, fontSize = 12.sp)
                            }
                            Box(Modifier.size(8.dp).background(if (app.allowSu) DarkApexColors.green else DarkApexColors.textSecondary, CircleShape))
                            Text("  ›", color = DarkApexColors.textSecondary, fontSize = 16.sp)
                        }
                        if (index != apps.lastIndex) Box(Modifier.fillMaxWidth().padding(start = 58.dp).height(0.5.dp).background(DarkApexColors.separator))
                    }
                }
                Spacer(Modifier.height(bottomInnerPadding))
            }
        }
    }
}

@Composable
fun StatusTag(label: String, backgroundColor: Color, contentColor: Color) {
    Box(
        modifier = Modifier
            .background(backgroundColor, RoundedCornerShape(8.dp))
            .padding(horizontal = 6.dp, vertical = 3.dp),
    ) {
        Text(text = label, color = contentColor, fontSize = 10.sp)
    }
}
