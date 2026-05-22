package com.qrj.apexsu.ui.webui

import android.app.Activity
import android.content.pm.ApplicationInfo
import android.os.Handler
import android.os.Looper
import android.util.Log
import android.view.Window
import android.webkit.JavascriptInterface
import android.widget.Toast
import androidx.core.content.pm.PackageInfoCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import com.qrj.apexsu.ui.util.listModules
import com.qrj.apexsu.ui.util.withNewRootShell
import com.qrj.apexsu.ui.viewmodel.SuperUserViewModel
import org.json.JSONArray
import org.json.JSONObject
import java.io.File

private const val TAG = "ApexSU.WebUI"
private val ALLOWLISTED_ACTIONS = setOf(
    "moduleInfo",
    "listPackages",
    "getPackagesInfo",
    "runModuleAction",
)

internal fun isAllowlistedWebUiAction(action: String): Boolean = ALLOWLISTED_ACTIONS.contains(action)

class WebViewInterface(
    private val state: WebUIState,
    private val allowPrivilegedOperations: Boolean,
) {
    private val webView get() = state.webView!!
    private val modDir get() = state.modDir

    // Legacy arbitrary shell APIs are intentionally disabled.
    @JavascriptInterface
    fun exec(cmd: String): String = blocked("exec", cmd)

    @JavascriptInterface
    fun exec(cmd: String, callbackFunc: String) {
        emitCallback(callbackFunc, 126, "", "Action blocked by security policy: exec")
    }

    @JavascriptInterface
    fun exec(
        cmd: String,
        options: String?,
        callbackFunc: String,
    ) {
        emitCallback(callbackFunc, 126, "", "Action blocked by security policy: exec")
    }

    @JavascriptInterface
    fun spawn(command: String, args: String, options: String?, callbackFunc: String) {
        emitCallback(callbackFunc, 126, "", "Action blocked by security policy: spawn")
    }

    private fun blocked(action: String, detail: String): String {
        Log.w(TAG, "Blocked legacy WebUI action=$action detail=$detail")
        return "Action blocked by security policy"
    }

    @JavascriptInterface
    fun invoke(action: String, payload: String?, callbackFunc: String) {
        if (!isAllowlistedWebUiAction(action)) {
            val msg = "Unsupported WebUI action: $action"
            Log.w(TAG, msg)
            emitCallback(callbackFunc, 1, "", msg)
            return
        }
        when (action) {
            "moduleInfo" -> emitCallback(callbackFunc, 0, moduleInfo(), "")
            "listPackages" -> emitCallback(callbackFunc, 0, listPackages(payload ?: "all"), "")
            "getPackagesInfo" -> emitCallback(callbackFunc, 0, getPackagesInfo(payload ?: "[]"), "")
            "runModuleAction" -> runModuleAction(callbackFunc)
            else -> emitCallback(callbackFunc, 1, "", "Unknown action")
        }
    }

    private fun runModuleAction(callbackFunc: String) {
        if (!allowPrivilegedOperations) {
            Log.w(TAG, "Blocked privileged WebUI operation (capability disabled): runModuleAction")
            emitCallback(
                callbackFunc,
                126,
                "",
                "Privileged WebUI operations are disabled in settings",
            )
            return
        }

        val moduleId = File(modDir).name
        state.requestPrivilegedConfirm("Run module action script for '$moduleId'?") { confirmed ->
            if (!confirmed) {
                Log.i(TAG, "User denied privileged WebUI operation: runModuleAction moduleId=$moduleId")
                emitCallback(callbackFunc, 130, "", "User denied privileged action")
                return@requestPrivilegedConfirm
            }

            val actionScript = "$modDir/action.sh"
            Log.i(TAG, "Executing privileged WebUI operation: runModuleAction moduleId=$moduleId script=$actionScript")
            val result = withNewRootShell(true) {
                newJob().add("if [ -f '$actionScript' ]; then sh '$actionScript'; else exit 127; fi")
                    .to(ArrayList(), ArrayList())
                    .exec()
            }
            emitCallback(
                callbackFunc,
                result.code,
                result.out.joinToString("\n"),
                result.err.joinToString("\n"),
            )
        }
    }

    private fun emitCallback(callbackFunc: String, code: Int, stdout: String, stderr: String) {
        val jsCode =
            "javascript:(function(){try{${callbackFunc}(${code},${JSONObject.quote(stdout)},${JSONObject.quote(stderr)});}catch(e){console.error(e);}})();"
        webView.post { webView.loadUrl(jsCode) }
    }

    @JavascriptInterface
    fun toast(msg: String) {
        webView.post {
            Toast.makeText(webView.context, msg, Toast.LENGTH_SHORT).show()
        }
    }

    @JavascriptInterface
    fun fullScreen(enable: Boolean) {
        val context = webView.context
        if (context is Activity) {
            Handler(Looper.getMainLooper()).post {
                if (enable) {
                    hideSystemUI(context.window)
                } else {
                    showSystemUI(context.window)
                }
            }
        }
        enableEdgeToEdge(enable)
    }

    @JavascriptInterface
    fun enableEdgeToEdge(enable: Boolean = true) {
        state.isInsetsEnabled = enable
    }

    @JavascriptInterface
    fun moduleInfo(): String {
        val moduleInfos = JSONArray(listModules())
        val currentModuleInfo = JSONObject()
        currentModuleInfo.put("moduleDir", modDir)
        val moduleId = File(modDir).name
        for (i in 0 until moduleInfos.length()) {
            val currentInfo = moduleInfos.getJSONObject(i)
            if (currentInfo.getString("id") != moduleId) {
                continue
            }
            val keys = currentInfo.keys()
            for (key in keys) {
                currentModuleInfo.put(key, currentInfo.get(key))
            }
            break
        }
        return currentModuleInfo.toString()
    }

    @JavascriptInterface
    fun listPackages(type: String): String {
        val packageNames = SuperUserViewModel.apps
            .filter { appInfo ->
                val flags = appInfo.packageInfo.applicationInfo?.flags ?: 0
                when (type.lowercase()) {
                    "system" -> (flags and ApplicationInfo.FLAG_SYSTEM) != 0
                    "user" -> (flags and ApplicationInfo.FLAG_SYSTEM) == 0
                    else -> true
                }
            }
            .map { it.packageName }
            .sorted()

        val jsonArray = JSONArray()
        for (pkgName in packageNames) {
            jsonArray.put(pkgName)
        }
        return jsonArray.toString()
    }

    @JavascriptInterface
    fun getPackagesInfo(packageNamesJson: String): String {
        val packageNames = JSONArray(packageNamesJson)
        val jsonArray = JSONArray()
        val appMap = SuperUserViewModel.apps.associateBy { it.packageName }
        for (i in 0 until packageNames.length()) {
            val pkgName = packageNames.getString(i)
            val appInfo = appMap[pkgName]
            if (appInfo != null) {
                val pkg = appInfo.packageInfo
                val app = pkg.applicationInfo
                val obj = JSONObject()
                obj.put("packageName", pkg.packageName)
                obj.put("versionName", pkg.versionName ?: "")
                obj.put("versionCode", PackageInfoCompat.getLongVersionCode(pkg))
                obj.put("appLabel", appInfo.label)
                obj.put(
                    "isSystem",
                    if (app != null) ((app.flags and ApplicationInfo.FLAG_SYSTEM) != 0) else JSONObject.NULL
                )
                obj.put("uid", app?.uid ?: JSONObject.NULL)
                jsonArray.put(obj)
            } else {
                val obj = JSONObject()
                obj.put("packageName", pkgName)
                obj.put("error", "Package not found or inaccessible")
                jsonArray.put(obj)
            }
        }
        return jsonArray.toString()
    }

    @JavascriptInterface
    fun exit() {
        state.requestExit()
    }
}

fun hideSystemUI(window: Window) =
    WindowInsetsControllerCompat(window, window.decorView).let { controller ->
        controller.hide(WindowInsetsCompat.Type.systemBars())
        controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
    }

fun showSystemUI(window: Window) =
    WindowInsetsControllerCompat(window, window.decorView).show(WindowInsetsCompat.Type.systemBars())
