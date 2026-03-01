package com.qrj.apexsu.ui.component

import androidx.compose.runtime.Composable
import com.qrj.apexsu.Natives

@Composable
fun KsuIsValid(
    content: @Composable () -> Unit
) {
    val isManager = Natives.isManager
    val ksuVersion = if (isManager) Natives.version else null

    if (ksuVersion != null) {
        content()
    }
}
