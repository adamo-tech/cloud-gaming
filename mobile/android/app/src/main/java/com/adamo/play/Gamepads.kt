package com.adamo.play

import android.content.Context
import android.content.SharedPreferences
import android.hardware.input.InputManager
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import android.view.InputDevice
import android.view.KeyEvent
import android.view.MotionEvent
import androidx.compose.runtime.mutableStateOf
import kotlin.math.max
import uniffi.adamo_play.Connection
import uniffi.adamo_play.GamepadRumble

const val LEFT_TRIGGER_AXIS = 4
const val RIGHT_TRIGGER_AXIS = 5
const val VIRTUAL_GAMEPAD_INDEX = 0
private const val PHONE_VIBRATION_KEY = "isPhoneVibrationEnabled"
private const val PLAYSTATION_LABELS_KEY = "hasPlayStationLabels"
private const val SHOWS_METRICS_KEY = "showsMetrics"

private val STANDARD_BUTTONS = mapOf(
    KeyEvent.KEYCODE_BUTTON_A to 0,
    KeyEvent.KEYCODE_BUTTON_B to 1,
    KeyEvent.KEYCODE_BUTTON_X to 2,
    KeyEvent.KEYCODE_BUTTON_Y to 3,
    KeyEvent.KEYCODE_BUTTON_L1 to 4,
    KeyEvent.KEYCODE_BUTTON_R1 to 5,
    KeyEvent.KEYCODE_BUTTON_L2 to 6,
    KeyEvent.KEYCODE_BUTTON_R2 to 7,
    KeyEvent.KEYCODE_BUTTON_SELECT to 8,
    KeyEvent.KEYCODE_BUTTON_START to 9,
    KeyEvent.KEYCODE_BUTTON_THUMBL to 10,
    KeyEvent.KEYCODE_BUTTON_THUMBR to 11,
    KeyEvent.KEYCODE_DPAD_UP to 12,
    KeyEvent.KEYCODE_DPAD_DOWN to 13,
    KeyEvent.KEYCODE_DPAD_LEFT to 14,
    KeyEvent.KEYCODE_DPAD_RIGHT to 15,
    KeyEvent.KEYCODE_BUTTON_MODE to 16,
)

class GamepadState {
    val axes = DoubleArray(6)
    val buttons = IntArray(17)
}

class Gamepads(context: Context, private val preferences: SharedPreferences) : InputManager.InputDeviceListener {
    @Volatile var connection: Connection? = null
    val hasPhysicalGamepad = mutableStateOf(false)
    val isPhoneVibrationEnabled = mutableStateOf(preferences.getBoolean(PHONE_VIBRATION_KEY, true))
    val hasPlayStationLabels = mutableStateOf(preferences.getBoolean(PLAYSTATION_LABELS_KEY, false))
    val showsMetrics = mutableStateOf(preferences.getBoolean(SHOWS_METRICS_KEY, false))
    val virtualGamepad = GamepadState()
    private val gamepadIndices = mutableMapOf<Int, Int>()
    private val gamepadStates = mutableMapOf<Int, GamepadState>()
    private val phoneVibrator = context.getSystemService(VibratorManager::class.java).defaultVibrator
    private val inputManager = context.getSystemService(InputManager::class.java)

    init {
        inputManager.registerInputDeviceListener(this, null)
        updatePhysicalGamepads()
    }

    fun onKey(event: KeyEvent): Boolean {
        val button = STANDARD_BUTTONS[event.keyCode]
        if (!event.isFromSource(InputDevice.SOURCE_GAMEPAD) || button == null) return false
        if (event.repeatCount > 0) return true
        val state = stateOf(event.deviceId)
        state.buttons[button] = if (event.action == KeyEvent.ACTION_DOWN) 1 else 0
        send(indexOf(event.deviceId), state)
        return true
    }

    fun onMotion(event: MotionEvent): Boolean {
        if (!event.isFromSource(InputDevice.SOURCE_JOYSTICK)) return false
        val state = stateOf(event.deviceId)
        state.axes[0] = event.getAxisValue(MotionEvent.AXIS_X).toDouble()
        state.axes[1] = event.getAxisValue(MotionEvent.AXIS_Y).toDouble()
        state.axes[2] = event.getAxisValue(MotionEvent.AXIS_Z).toDouble()
        state.axes[3] = event.getAxisValue(MotionEvent.AXIS_RZ).toDouble()
        state.axes[LEFT_TRIGGER_AXIS] = max(event.getAxisValue(MotionEvent.AXIS_LTRIGGER), event.getAxisValue(MotionEvent.AXIS_BRAKE)).toDouble()
        state.axes[RIGHT_TRIGGER_AXIS] = max(event.getAxisValue(MotionEvent.AXIS_RTRIGGER), event.getAxisValue(MotionEvent.AXIS_GAS)).toDouble()
        val hatX = event.getAxisValue(MotionEvent.AXIS_HAT_X)
        val hatY = event.getAxisValue(MotionEvent.AXIS_HAT_Y)
        state.buttons[12] = if (hatY < -0.5f) 1 else 0
        state.buttons[13] = if (hatY > 0.5f) 1 else 0
        state.buttons[14] = if (hatX < -0.5f) 1 else 0
        state.buttons[15] = if (hatX > 0.5f) 1 else 0
        send(indexOf(event.deviceId), state)
        return true
    }

    fun sendVirtual() {
        send(VIRTUAL_GAMEPAD_INDEX, virtualGamepad)
    }

    fun setPhoneVibrationEnabled(isEnabled: Boolean) {
        isPhoneVibrationEnabled.value = isEnabled
        preferences.edit().putBoolean(PHONE_VIBRATION_KEY, isEnabled).apply()
    }

    fun setPlayStationLabels(hasPlayStationLabels: Boolean) {
        this.hasPlayStationLabels.value = hasPlayStationLabels
        preferences.edit().putBoolean(PLAYSTATION_LABELS_KEY, hasPlayStationLabels).apply()
    }

    fun setShowsMetrics(showsMetrics: Boolean) {
        this.showsMetrics.value = showsMetrics
        preferences.edit().putBoolean(SHOWS_METRICS_KEY, showsMetrics).apply()
    }

    fun rumble(rumble: GamepadRumble) {
        val deviceId = gamepadIndices.entries.firstOrNull { it.value == rumble.gamepadIndex.toInt() }?.key
        val gamepadVibrator = deviceId?.let { InputDevice.getDevice(it)?.vibratorManager?.defaultVibrator }
        val vibrator = gamepadVibrator ?: phoneVibrator.takeIf { isPhoneVibrationEnabled.value } ?: return
        vibrate(vibrator, max(rumble.strongMagnitude, rumble.weakMagnitude), rumble.durationMs.toLong())
    }

    override fun onInputDeviceAdded(deviceId: Int) = updatePhysicalGamepads()

    override fun onInputDeviceRemoved(deviceId: Int) = updatePhysicalGamepads()

    override fun onInputDeviceChanged(deviceId: Int) = updatePhysicalGamepads()

    private fun updatePhysicalGamepads() {
        hasPhysicalGamepad.value = inputManager.inputDeviceIds.any { deviceId ->
            InputDevice.getDevice(deviceId)?.supportsSource(InputDevice.SOURCE_GAMEPAD) == true
        }
    }

    private fun indexOf(deviceId: Int): Int = gamepadIndices.getOrPut(deviceId) { gamepadIndices.size }

    private fun stateOf(deviceId: Int): GamepadState = gamepadStates.getOrPut(deviceId) { GamepadState() }

    private fun send(index: Int, state: GamepadState) {
        connection?.sendGamepad(index.toUByte(), state.axes.toList(), state.buttons.toList())
    }
}

private fun vibrate(vibrator: Vibrator, magnitude: Float, durationMs: Long) {
    val amplitude = (magnitude * 255).toInt()
    if (amplitude == 0) {
        vibrator.cancel()
    } else {
        vibrator.vibrate(VibrationEffect.createOneShot(durationMs, amplitude))
    }
}
