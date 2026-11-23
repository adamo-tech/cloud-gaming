package com.adamo.play

import android.graphics.Rect
import android.view.SurfaceView
import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import kotlinx.coroutines.delay

@Composable
fun PlayScreen(stream: GameStream, gamepads: Gamepads, tabletopFold: Rect?, onExit: () -> Unit) {
    val videoAspectRatio by stream.videoAspectRatio
    val error by stream.error
    val hasPhysicalGamepad by gamepads.hasPhysicalGamepad
    val showsMetrics by gamepads.showsMetrics
    val framesPerSecond by stream.framesPerSecond
    val latencyMs by stream.latencyMs
    var showsTouchControls by remember { mutableStateOf(true) }
    var isMenuOpen by remember { mutableStateOf(false) }
    var showsHint by remember { mutableStateOf(true) }
    LaunchedEffect(hasPhysicalGamepad) { showsTouchControls = !hasPhysicalGamepad }
    LaunchedEffect(Unit) {
        delay(4000)
        showsHint = false
    }
    val foldEdges = with(LocalDensity.current) { tabletopFold?.let { it.top.toDp() to it.bottom.toDp() } }
    BackHandler { isMenuOpen = !isMenuOpen }
    Box(Modifier.fillMaxSize().background(Color.Black), contentAlignment = Alignment.Center) {
        Backdrop(Modifier.padding(top = foldEdges?.second ?: 0.dp))
        Box(Modifier.align(Alignment.TopCenter).fillMaxWidth().then(foldEdges?.let { Modifier.height(it.first) } ?: Modifier.fillMaxHeight()), contentAlignment = Alignment.Center) {
            AndroidView(
                factory = { context -> SurfaceView(context).apply { holder.addCallback(stream) } },
                modifier = Modifier.aspectRatio(videoAspectRatio),
            )
        }
        Box(Modifier.fillMaxSize().pointerInput(Unit) { detectTapGestures(onLongPress = { isMenuOpen = true }) })
        if (showsTouchControls && !isMenuOpen) {
            if (foldEdges != null) TouchController(gamepads, Modifier.padding(top = foldEdges.second), isStacked = true) else TouchController(gamepads)
        }
        AnimatedVisibility(showsHint, Modifier.align(Alignment.TopCenter).padding(top = 12.dp), exit = fadeOut(tween(600))) {
            Text(
                "Hold anywhere for the menu",
                Modifier.glass(CircleShape).padding(horizontal = 14.dp, vertical = 8.dp),
                color = Palette.foreground,
                fontSize = 13.sp,
                fontWeight = FontWeight.Medium,
            )
        }
        if (showsMetrics) {
            Text(
                "$framesPerSecond fps · $latencyMs ms",
                Modifier.align(Alignment.BottomEnd).padding(12.dp).glass(CircleShape).padding(horizontal = 14.dp, vertical = 8.dp),
                color = Palette.foreground,
                fontSize = 13.sp,
                fontWeight = FontWeight.Medium,
            )
        }
        if (isMenuOpen) {
            Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.55f)).pointerInput(Unit) { detectTapGestures { isMenuOpen = false } })
            Menu(
                showsTouchControls = showsTouchControls,
                onTouchControlsChange = { showsTouchControls = it },
                gamepads = gamepads,
                onResume = { isMenuOpen = false },
                onExit = onExit,
            )
        }
        error?.let { ErrorCard(it, onExit) }
    }
}

@Composable
private fun Menu(
    showsTouchControls: Boolean,
    onTouchControlsChange: (Boolean) -> Unit,
    gamepads: Gamepads,
    onResume: () -> Unit,
    onExit: () -> Unit,
) {
    val isPhoneVibrationEnabled by gamepads.isPhoneVibrationEnabled
    val hasPlayStationLabels by gamepads.hasPlayStationLabels
    val showsMetrics by gamepads.showsMetrics
    val sectionShape = RoundedCornerShape(14.dp)
    Column(
        Modifier.width(420.dp).glass(RoundedCornerShape(22.dp)).pointerInput(Unit) {}.padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(18.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Logomark(Modifier.size(28.dp, 24.dp))
            Text("Adamo Play", style = displayStyle(26.sp))
            Spacer(Modifier.weight(1f))
            Box(Modifier.size(8.dp).background(Palette.success, CircleShape))
            Text("Connected", color = Palette.muted, fontSize = 13.sp)
        }
        Column(Modifier.background(Color.White.copy(alpha = 0.04f), sectionShape).padding(horizontal = 16.dp)) {
            Setting("Touch controls", "Turn off when a controller connects", showsTouchControls, onTouchControlsChange)
            HorizontalDivider(color = Palette.glassBorder)
            Setting("Phone vibration", "Rumble on the phone when no controller is connected", isPhoneVibrationEnabled, gamepads::setPhoneVibrationEnabled)
            HorizontalDivider(color = Palette.glassBorder)
            Setting("PlayStation buttons", "Label the on-screen buttons with shapes and L1, R1", hasPlayStationLabels, gamepads::setPlayStationLabels)
            HorizontalDivider(color = Palette.glassBorder)
            Setting("Show metrics", "Frame rate and latency from the gaming PC", showsMetrics, gamepads::setShowsMetrics)
        }
        Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            FilledButton(onExit, Modifier.weight(1f), containerColor = Palette.danger.copy(alpha = 0.12f), contentColor = Palette.danger) { Text("Disconnect") }
            FilledButton(onResume, Modifier.weight(1f)) { Text("Resume") }
        }
    }
}

@Composable
private fun Setting(title: String, detail: String, isOn: Boolean, onChange: (Boolean) -> Unit) {
    Row(
        Modifier.fillMaxWidth().toggleable(isOn, role = Role.Switch, onValueChange = onChange).padding(vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f)) {
            Text(title, color = Palette.foreground, fontSize = 15.sp, fontWeight = FontWeight.Medium)
            Text(detail, color = Palette.muted, fontSize = 12.sp)
        }
        Switch(
            checked = isOn,
            onCheckedChange = null,
            colors = SwitchDefaults.colors(
                checkedThumbColor = Color.White,
                checkedTrackColor = Palette.success,
                checkedBorderColor = Color.Transparent,
                uncheckedThumbColor = Color.White,
                uncheckedTrackColor = Color.White.copy(alpha = 0.16f),
                uncheckedBorderColor = Color.Transparent,
            ),
        )
    }
}

@Composable
private fun ErrorCard(error: String, onExit: () -> Unit) {
    Column(
        Modifier.width(380.dp).glass(RoundedCornerShape(22.dp)).padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Text("Connection lost", style = displayStyle(24.sp))
        Text(error, color = Palette.muted, fontSize = 14.sp)
        FilledButton(onExit) { Text("Back") }
    }
}
