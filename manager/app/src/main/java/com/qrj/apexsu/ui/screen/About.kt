package com.qrj.apexsu.ui.screen

import android.util.Log
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
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
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Book
import androidx.compose.material.icons.rounded.BugReport
import androidx.compose.material.icons.rounded.Code
import androidx.compose.material.icons.rounded.Favorite
import androidx.compose.material.icons.rounded.NewReleases
import androidx.compose.material.icons.outlined.Security
import androidx.compose.material.icons.rounded.StarOutline
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.dropUnlessResumed
import com.kyant.capsule.ContinuousRoundedRectangle
import dev.chrisbanes.haze.HazeState
import dev.chrisbanes.haze.HazeStyle
import dev.chrisbanes.haze.HazeTint
import dev.chrisbanes.haze.hazeEffect
import dev.chrisbanes.haze.hazeSource
import com.qrj.apexsu.BuildConfig
import com.qrj.apexsu.R
import com.qrj.apexsu.ui.navigation3.LocalNavigator
import com.qrj.apexsu.ui.theme.LocalEnableBlur
import top.yukonga.miuix.kmp.basic.Card
import top.yukonga.miuix.kmp.basic.Icon
import top.yukonga.miuix.kmp.basic.IconButton
import top.yukonga.miuix.kmp.basic.MiuixScrollBehavior
import top.yukonga.miuix.kmp.basic.Scaffold
import top.yukonga.miuix.kmp.basic.Text
import top.yukonga.miuix.kmp.basic.TopAppBar
import top.yukonga.miuix.kmp.extra.SuperArrow
import top.yukonga.miuix.kmp.icon.MiuixIcons
import top.yukonga.miuix.kmp.icon.extended.Back
import top.yukonga.miuix.kmp.icon.extended.Link
import top.yukonga.miuix.kmp.theme.MiuixTheme.colorScheme
import top.yukonga.miuix.kmp.utils.overScrollVertical

@Composable
fun AboutScreen() {
    val navigator = LocalNavigator.current
    val uriHandler = LocalUriHandler.current
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

    val htmlString = stringResource(
        id = R.string.about_source_code,
        "<b><a href=\"https://github.com/qrjhamron/ApexSU\">GitHub</a></b>",
        "<b><a href=\"https://github.com/qrjhamron/ApexSU\">ApexSU</a></b>"
    )
    val result = extractLinks(htmlString)

    Scaffold(
        topBar = {
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
                title = stringResource(R.string.about),
                navigationIcon = {
                    IconButton(
                        modifier = Modifier.padding(start = 16.dp),
                        onClick = dropUnlessResumed { navigator.pop() }
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
                scrollBehavior = scrollBehavior
            )
        },
        popupHost = { },
        contentWindowInsets = WindowInsets.systemBars.add(WindowInsets.displayCutout).only(WindowInsetsSides.Horizontal)
    ) { innerPadding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxHeight()
                .overScrollVertical()
                .nestedScroll(scrollBehavior.nestedScrollConnection)
                .let { if (enableBlur) it.hazeSource(state = hazeState) else it }
                .padding(horizontal = 12.dp),
            contentPadding = innerPadding,
            overscrollEffect = null,
        ) {
            item {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 12.dp)
                        .padding(vertical = 48.dp),
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    Box(
                        contentAlignment = Alignment.Center,
                        modifier = Modifier
                            .size(80.dp)
                            .clip(ContinuousRoundedRectangle(16.dp))
                            .background(Color.White)
                    ) {
                        Icon(
                            imageVector = Icons.Outlined.Security,
                            contentDescription = stringResource(id = R.string.app_name),
                            tint = Color(0xFF0A84FF),
                            modifier = Modifier.size(48.dp),
                        )
                    }
                    Text(
                        modifier = Modifier.padding(top = 12.dp),
                        text = stringResource(id = R.string.app_name),
                        fontWeight = FontWeight.Medium,
                        fontSize = 26.sp
                    )
                    Text(
                        text = BuildConfig.VERSION_NAME,
                        fontSize = 14.sp
                    )
                }
            }
            item {
                Card(
                    modifier = Modifier.padding(bottom = 12.dp)
                ) {
                    result.forEach {
                        SuperArrow(
                            title = it.fullText,
                            onClick = {
                                uriHandler.openUri(it.url)
                            }
                        )
                    }
                }
            }
            item {
                Card(
                    modifier = Modifier.padding(bottom = 12.dp)
                ) {
                    SuperArrow(
                        title = stringResource(R.string.about_our_repository),
                        startAction = {
                            Icon(
                                imageVector = Icons.Rounded.Favorite,
                                contentDescription = null,
                                tint = colorScheme.primary,
                                modifier = Modifier
                                    .padding(end = 12.dp)
                                    .size(24.dp)
                            )
                        },
                        onClick = {
                            uriHandler.openUri("https://github.com/qrjhamron/ApexSU")
                        }
                    )
                    SuperArrow(
                        title = stringResource(R.string.about_star_on_github),
                        startAction = {
                            Icon(
                                imageVector = Icons.Rounded.StarOutline,
                                contentDescription = null,
                                tint = colorScheme.onSurfaceVariantSummary,
                                modifier = Modifier
                                    .padding(end = 12.dp)
                                    .size(24.dp)
                            )
                        },
                        onClick = {
                            uriHandler.openUri("https://github.com/qrjhamron/ApexSU")
                        }
                    )
                    SuperArrow(
                        title = stringResource(R.string.about_view_source),
                        startAction = {
                            Icon(
                                imageVector = Icons.Rounded.Code,
                                contentDescription = null,
                                tint = colorScheme.onSurfaceVariantSummary,
                                modifier = Modifier
                                    .padding(end = 12.dp)
                                    .size(24.dp)
                            )
                        },
                        onClick = {
                            uriHandler.openUri("https://github.com/qrjhamron/ApexSU")
                        }
                    )
                    SuperArrow(
                        title = stringResource(R.string.about_report_issue),
                        startAction = {
                            Icon(
                                imageVector = Icons.Rounded.BugReport,
                                contentDescription = null,
                                tint = colorScheme.onSurfaceVariantSummary,
                                modifier = Modifier
                                    .padding(end = 12.dp)
                                    .size(24.dp)
                            )
                        },
                        onClick = {
                            uriHandler.openUri("https://github.com/qrjhamron/ApexSU/issues")
                        }
                    )
                    SuperArrow(
                        title = stringResource(R.string.about_changelog),
                        startAction = {
                            Icon(
                                imageVector = Icons.Rounded.NewReleases,
                                contentDescription = null,
                                tint = colorScheme.onSurfaceVariantSummary,
                                modifier = Modifier
                                    .padding(end = 12.dp)
                                    .size(24.dp)
                            )
                        },
                        onClick = {
                            uriHandler.openUri("https://github.com/qrjhamron/ApexSU/releases")
                        }
                    )
                    SuperArrow(
                        title = stringResource(R.string.about_installation_guide),
                        startAction = {
                            Icon(
                                imageVector = Icons.Rounded.Book,
                                contentDescription = null,
                                tint = colorScheme.onSurfaceVariantSummary,
                                modifier = Modifier
                                    .padding(end = 12.dp)
                                    .size(24.dp)
                            )
                        },
                        onClick = {
                            uriHandler.openUri("https://github.com/qrjhamron/ApexSU#readme")
                        }
                    )
                }
            }
            item {
                Card(
                    modifier = Modifier.padding(bottom = 12.dp)
                ) {
                    SuperArrow(
                        title = stringResource(R.string.about_built_on_kernelsu),
                        summary = stringResource(R.string.about_kernelsu_thanks),
                        startAction = {
                            Icon(
                                imageVector = MiuixIcons.Link,
                                contentDescription = null,
                                tint = colorScheme.onSurfaceVariantSummary,
                                modifier = Modifier
                                    .padding(end = 12.dp)
                                    .size(24.dp)
                            )
                        },
                        onClick = {
                            uriHandler.openUri("https://github.com/qrjhamron/ApexSU")
                        }
                    )
                }
            }
            item {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(vertical = 24.dp),
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    Text(
                        text = stringResource(R.string.about_made_with_love),
                        fontSize = 13.sp,
                        color = colorScheme.onSurfaceVariantSummary,
                        textAlign = TextAlign.Center
                    )
                    Text(
                        text = stringResource(R.string.about_license),
                        fontSize = 12.sp,
                        color = colorScheme.onSurfaceVariantSummary,
                        textAlign = TextAlign.Center,
                        modifier = Modifier.padding(top = 4.dp)
                    )
                }
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

data class LinkInfo(
    val fullText: String,
    val url: String
)

fun extractLinks(html: String): List<LinkInfo> {
    val regex = Regex(
        """([^<>\n\r]+?)\s*<b>\s*<a\b[^>]*\bhref\s*=\s*(['"]?)([^'"\s>]+)\2[^>]*>([^<]+)</a>\s*</b>\s*(.*?)\s*(?=<br|\n|$)""",
        RegexOption.MULTILINE
    )

    Log.d("ggc", "extractLinks: $html")

    return regex.findAll(html).mapNotNull { match ->
        try {
            val before = match.groupValues[1].trim()
            val url = match.groupValues[3].trim()
            val title = match.groupValues[4].trim()
            val after = match.groupValues[5].trim()

            val fullText = "$before $title $after"
            Log.d("ggc", "extractLinks: $fullText -> $url")
            LinkInfo(fullText, url)
        } catch (e: Exception) {
            Log.e("ggc", "extractLinks failed: ${e.message}")
            null
        }
    }.toList()
}
