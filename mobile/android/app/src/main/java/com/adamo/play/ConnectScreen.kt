package com.adamo.play

import android.content.SharedPreferences
import android.graphics.Rect
import android.view.WindowManager
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.adamo_play.Connection
import uniffi.adamo_play.PlayException

@Composable
fun ConnectScreen(preferences: SharedPreferences, tabletopFold: Rect?, onConnect: (Connection) -> Unit) {
    var address by remember { mutableStateOf(preferences.getString("address", "")!!) }
    var pairingKey by remember { mutableStateOf(preferences.getString("pairingKey", "")!!) }
    var isThroughRelay by remember { mutableStateOf(preferences.getBoolean("isThroughRelay", false)) }
    var hostName by remember { mutableStateOf(preferences.getString("hostName", "")!!) }
    var apiKey by remember { mutableStateOf(preferences.getString("apiKey", "")!!) }
    var isConnecting by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()
    val windowManager = LocalContext.current.getSystemService(WindowManager::class.java)

    val play: () -> Unit = {
        preferences.edit()
            .putString("address", address)
            .putString("pairingKey", pairingKey)
            .putBoolean("isThroughRelay", isThroughRelay)
            .putString("hostName", hostName)
            .putString("apiKey", apiKey)
            .apply()
        isConnecting = true
        error = null
        val window = windowManager.currentWindowMetrics.bounds
        val videoWidth = window.width()
        val videoHeight = tabletopFold?.top ?: window.height()
        scope.launch {
            try {
                val screenWidth = maxOf(videoWidth, videoHeight).toUInt()
                val screenHeight = minOf(videoWidth, videoHeight).toUInt()
                onConnect(withContext(Dispatchers.IO) {
                    if (isThroughRelay) {
                        Connection.connectThroughRelay(apiKey, hostName, screenWidth, screenHeight)
                    } else {
                        Connection.connect(address, pairingKey, screenWidth, screenHeight)
                    }
                })
            } catch (exception: PlayException) {
                error = exception.message
            }
            isConnecting = false
        }
    }

    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        Backdrop()
        Row(Modifier.padding(32.dp), horizontalArrangement = Arrangement.spacedBy(48.dp), verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f, fill = false).widthIn(max = 320.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                Logomark(Modifier.size(44.dp, 36.dp))
                Text("Adamo Play", style = displayStyle(44.sp))
                Text("Play the games on your PC from anywhere, with a controller or the touch screen.", color = Palette.muted, fontSize = 16.sp)
            }
            Column(Modifier.width(360.dp).glass(RoundedCornerShape(20.dp)).padding(24.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                ConnectionPicker(isThroughRelay) { isThroughRelay = it }
                if (isThroughRelay) {
                    Field("Gaming PC", hostName, { hostName = it }, "cloud-gaming", KeyboardType.Ascii)
                    Field("API key", apiKey, { apiKey = it }, "From your Adamo organization", KeyboardType.Password, PasswordVisualTransformation())
                } else {
                    Field("Gaming PC", address, { address = it }, "192.168.1.20:17447", KeyboardType.Uri)
                    Field("Pairing key", pairingKey, { pairingKey = it }, "Printed by the host", KeyboardType.Password, PasswordVisualTransformation())
                }
                error?.let { Text(it, color = Palette.danger, fontSize = 13.sp) }
                FilledButton(play, Modifier.padding(top = 4.dp), isEnabled = (if (isThroughRelay) hostName.isNotEmpty() && apiKey.isNotEmpty() else address.isNotEmpty() && pairingKey.isNotEmpty()) && !isConnecting) {
                    if (isConnecting) {
                        CircularProgressIndicator(Modifier.size(20.dp), color = Palette.background, strokeWidth = 2.dp)
                    } else {
                        Text("Play")
                    }
                }
            }
        }
    }
}

@Composable
private fun ConnectionPicker(isThroughRelay: Boolean, onChange: (Boolean) -> Unit) {
    val shape = RoundedCornerShape(10.dp)
    Row(Modifier.fillMaxWidth().height(36.dp).background(Color.White.copy(alpha = 0.04f), shape).border(1.dp, Palette.glassBorder, shape).padding(2.dp)) {
        for ((label, isRelayOption) in listOf("Local network" to false, "Adamo relay" to true)) {
            val isSelected = isRelayOption == isThroughRelay
            Box(
                Modifier
                    .weight(1f)
                    .fillMaxHeight()
                    .clip(RoundedCornerShape(8.dp))
                    .background(if (isSelected) Color.White.copy(alpha = 0.14f) else Color.Transparent)
                    .clickable { onChange(isRelayOption) },
                contentAlignment = Alignment.Center,
            ) {
                Text(label, color = if (isSelected) Palette.foreground else Palette.muted, fontSize = 13.sp, fontWeight = FontWeight.Medium)
            }
        }
    }
}

@Composable
private fun Field(
    label: String,
    value: String,
    onValueChange: (String) -> Unit,
    placeholder: String,
    keyboardType: KeyboardType,
    visualTransformation: VisualTransformation = VisualTransformation.None,
) {
    val shape = RoundedCornerShape(10.dp)
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Text(label, color = Palette.muted, fontSize = 12.sp, fontWeight = FontWeight.Medium)
        BasicTextField(
            value = value,
            onValueChange = onValueChange,
            singleLine = true,
            textStyle = TextStyle(color = Palette.foreground, fontSize = 15.sp),
            cursorBrush = SolidColor(Color.White),
            keyboardOptions = KeyboardOptions(keyboardType = keyboardType),
            visualTransformation = visualTransformation,
            modifier = Modifier
                .fillMaxWidth()
                .height(44.dp)
                .background(Color.White.copy(alpha = 0.04f), shape)
                .border(1.dp, Palette.glassBorder, shape),
            decorationBox = { textField ->
                Box(Modifier.fillMaxSize().padding(horizontal = 12.dp), contentAlignment = Alignment.CenterStart) {
                    if (value.isEmpty()) Text(placeholder, color = Palette.muted.copy(alpha = 0.6f), fontSize = 15.sp)
                    textField()
                }
            },
        )
    }
}
