package com.adamo.play

import android.os.Bundle
import android.system.Os
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.WindowManager
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.Color
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat

class MainActivity : ComponentActivity() {
    private lateinit var gamepads: Gamepads
    private var stream by mutableStateOf<GameStream?>(null)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Os.setenv("TMPDIR", cacheDir.absolutePath, true)
        val preferences = getPreferences(MODE_PRIVATE)
        gamepads = Gamepads(this, preferences)
        hideSystemBars()
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        setContent {
            MaterialTheme(colorScheme = darkColorScheme(primary = Color.White, onPrimary = Palette.background, background = Palette.background, surface = Palette.background)) {
                val currentStream = stream
                if (currentStream == null) {
                    ConnectScreen(preferences) { connection ->
                        stream = GameStream(connection, gamepads)
                    }
                } else {
                    PlayScreen(currentStream, gamepads) {
                        currentStream.close()
                        stream = null
                    }
                }
            }
        }
    }

    override fun dispatchKeyEvent(event: KeyEvent): Boolean =
        gamepads.onKey(event) || super.dispatchKeyEvent(event)

    override fun dispatchGenericMotionEvent(event: MotionEvent): Boolean =
        gamepads.onMotion(event) || super.dispatchGenericMotionEvent(event)

    private fun hideSystemBars() {
        val controller = WindowCompat.getInsetsController(window, window.decorView)
        controller.hide(WindowInsetsCompat.Type.systemBars())
        controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
    }
}
