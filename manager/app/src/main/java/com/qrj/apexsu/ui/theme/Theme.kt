package com.qrj.apexsu.ui.theme

import androidx.compose.runtime.Composable
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color
import com.qrj.apexsu.ui.webui.MonetColorsProvider.UpdateCss
import top.yukonga.miuix.kmp.theme.ColorSchemeMode
import top.yukonga.miuix.kmp.theme.MiuixTheme
import top.yukonga.miuix.kmp.theme.ThemeController

val IOSBlack = Color(0xFF000000)
val IosSurfaceL1 = Color(0xFF1C1C1E)
val IosSurfaceL2 = Color(0xFF2C2C2E)
val IosSurfaceL3 = Color(0xFF3A3A3C)
val IosSeparator = Color(0xFF38383A)
val IosTextPrimary = Color(0xFFFFFFFF)
val IosTextSecondary = Color(0xFF8E8E93)
val IosTextTertiary = Color(0xFF636366)
val IosBlue = Color(0xFF0A84FF)
val IosGreen = Color(0xFF30D158)
val IosRed = Color(0xFFFF453A)
val IosOrange = Color(0xFFFF9F0A)

data class ApexColors(
    val background: Color,
    val surfaceL1: Color,
    val surfaceL2: Color,
    val surfaceL3: Color,
    val separator: Color,
    val textPrimary: Color,
    val textSecondary: Color,
    val textTertiary: Color,
    val blue: Color,
    val green: Color,
    val red: Color,
    val orange: Color,
)

val DarkApexColors = ApexColors(
    background = IOSBlack,
    surfaceL1 = IosSurfaceL1,
    surfaceL2 = IosSurfaceL2,
    surfaceL3 = IosSurfaceL3,
    separator = IosSeparator,
    textPrimary = IosTextPrimary,
    textSecondary = IosTextSecondary,
    textTertiary = IosTextTertiary,
    blue = IosBlue,
    green = IosGreen,
    red = IosRed,
    orange = IosOrange,
)

val LocalApexColors = staticCompositionLocalOf { DarkApexColors }

@Composable
fun KernelSUTheme(
    colorMode: Int = 0,
    keyColor: Color? = null,
    content: @Composable () -> Unit,
) {
    val controller = ThemeController(ColorSchemeMode.Dark)
    MiuixTheme(
        controller = controller,
        content = {
            androidx.compose.runtime.CompositionLocalProvider(LocalApexColors provides DarkApexColors) {
                UpdateCss()
                content()
            }
        },
    )
}

@Composable
@ReadOnlyComposable
fun isInDarkTheme(): Boolean = true

val LocalColorMode = staticCompositionLocalOf { 2 }
val LocalEnableBlur = staticCompositionLocalOf { false }
val LocalEnableFloatingBottomBar = staticCompositionLocalOf { false }
val LocalEnableFloatingBottomBarBlur = staticCompositionLocalOf { false }
val LocalReduceMotion = staticCompositionLocalOf { false }
