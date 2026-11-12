package com.adamo.play

import android.content.SharedPreferences
import android.view.WindowManager
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.requiredSize
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
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
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
fun ConnectScreen(preferences: SharedPreferences, onConnect: (Connection) -> Unit) {
    var address by remember { mutableStateOf(preferences.getString("address", "")!!) }
    var pairingKey by remember { mutableStateOf(preferences.getString("pairingKey", "")!!) }
    var isConnecting by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()
    val glowRadius = with(LocalDensity.current) { 520.dp.toPx() }
    val screen = LocalContext.current.getSystemService(WindowManager::class.java).maximumWindowMetrics.bounds

    val play: () -> Unit = {
        preferences.edit().putString("address", address).putString("pairingKey", pairingKey).apply()
        isConnecting = true
        error = null
        scope.launch {
            try {
                onConnect(withContext(Dispatchers.IO) { Connection.connect(address, pairingKey, maxOf(screen.width(), screen.height()).toUInt(), minOf(screen.width(), screen.height()).toUInt()) })
            } catch (exception: PlayException) {
                error = exception.message
            }
            isConnecting = false
        }
    }

    Box(
        Modifier
            .fillMaxSize()
            .background(Palette.background)
            .background(Brush.radialGradient(listOf(Color.White.copy(alpha = 0.07f), Color.Transparent), center = Offset.Zero, radius = glowRadius)),
        contentAlignment = Alignment.Center,
    ) {
        Logomark(Modifier.offset((-260).dp, 120.dp).requiredSize(520.dp, 430.dp), Color.White.copy(alpha = 0.025f))
        Row(Modifier.padding(32.dp), horizontalArrangement = Arrangement.spacedBy(48.dp), verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f, fill = false).widthIn(max = 320.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                Logomark(Modifier.size(44.dp, 36.dp))
                Text("Adamo Play", style = displayStyle(44.sp))
                Text("Play the games on your PC from anywhere, with a controller or the touch screen.", color = Palette.muted, fontSize = 16.sp)
            }
            Column(Modifier.width(360.dp).glass(RoundedCornerShape(20.dp)).padding(24.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                Field("Gaming PC", address, { address = it }, "192.168.1.20:17447", KeyboardType.Uri)
                Field("Pairing key", pairingKey, { pairingKey = it }, "Printed by the host", KeyboardType.Password, PasswordVisualTransformation())
                error?.let { Text(it, color = Palette.danger, fontSize = 13.sp) }
                FilledButton(play, Modifier.padding(top = 4.dp), isEnabled = address.isNotEmpty() && pairingKey.isNotEmpty() && !isConnecting) {
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
