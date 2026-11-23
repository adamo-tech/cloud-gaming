package com.adamo.play

import android.view.HapticFeedbackConstants
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.drag
import androidx.compose.foundation.gestures.waitForUpOrCancellation
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.rotate
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

private val BUTTON_SIZE = 44.dp
private val STICK_SIZE = 116.dp
private val RAISE = 72.dp
private val STICK_BUTTON_SIZE = 34.dp
private val STICK_BUTTON_SPACING = 10.dp
private val IDLE_FILL = Color.White.copy(alpha = 0.1f)
private val IDLE_STROKE = Color.White.copy(alpha = 0.22f)
private val IDLE_CONTENT = Color.White.copy(alpha = 0.9f)

private enum class Glyph(val degrees: Float) { Up(0f), Right(90f), Down(180f), Left(270f), View(0f), Menu(0f), Triangle(0f), Square(0f), Circle(0f), XMark(0f) }

@Composable
fun TouchController(gamepads: Gamepads, modifier: Modifier = Modifier, isStacked: Boolean = false) {
    val gamepad = gamepads.virtualGamepad
    val hasPlayStationLabels by gamepads.hasPlayStationLabels

    fun button(index: Int): (Boolean) -> Unit = { isPressed ->
        gamepad.buttons[index] = if (isPressed) 1 else 0
        gamepads.sendVirtual()
    }

    fun trigger(index: Int, axis: Int): (Boolean) -> Unit = { isPressed ->
        gamepad.buttons[index] = if (isPressed) 1 else 0
        gamepad.axes[axis] = if (isPressed) 1.0 else 0.0
        gamepads.sendVirtual()
    }

    fun stick(xAxis: Int): (Float, Float) -> Unit = { x, y ->
        gamepad.axes[xAxis] = x.toDouble()
        gamepad.axes[xAxis + 1] = y.toDouble()
        gamepads.sendVirtual()
    }

    val faceButton = @Composable { xbox: String, playStation: Glyph, index: Int ->
        if (hasPlayStationLabels) GlyphButton(playStation, button(index)) else LabelButton(xbox, button(index))
    }

    val menuButtons = @Composable {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(14.dp)) {
            GlyphButton(Glyph.View, button(8), 36.dp)
            PadButton(button(16), 40.dp) { Logomark(Modifier.size(18.dp), it) }
            GlyphButton(Glyph.Menu, button(9), 36.dp)
        }
    }

    Column(modifier.fillMaxSize().padding(horizontal = 24.dp, vertical = 16.dp)) {
        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                Shoulder(if (hasPlayStationLabels) "L2" else "LT", trigger(6, LEFT_TRIGGER_AXIS))
                Shoulder(if (hasPlayStationLabels) "L1" else "LB", button(4))
            }
            Spacer(Modifier.weight(1f))
            if (isStacked) {
                menuButtons()
                Spacer(Modifier.weight(1f))
            }
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                Shoulder(if (hasPlayStationLabels) "R1" else "RB", button(5))
                Shoulder(if (hasPlayStationLabels) "R2" else "RT", trigger(7, RIGHT_TRIGGER_AXIS))
            }
        }
        Spacer(Modifier.weight(1f))
        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.Bottom) {
            Row(verticalAlignment = Alignment.Bottom, horizontalArrangement = Arrangement.spacedBy(18.dp)) {
                Column(
                    Modifier.padding(bottom = RAISE - STICK_BUTTON_SPACING - STICK_BUTTON_SIZE),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(STICK_BUTTON_SPACING),
                ) {
                    Stick(stick(0))
                    StickButton("L3", button(10))
                }
                Cross(
                    top = { GlyphButton(Glyph.Up, button(12), cornerRadius = 10.dp) },
                    left = { GlyphButton(Glyph.Left, button(14), cornerRadius = 10.dp) },
                    right = { GlyphButton(Glyph.Right, button(15), cornerRadius = 10.dp) },
                    bottom = { GlyphButton(Glyph.Down, button(13), cornerRadius = 10.dp) },
                )
            }
            Spacer(Modifier.weight(1f))
            if (!isStacked) {
                menuButtons()
                Spacer(Modifier.weight(1f))
            }
            Row(verticalAlignment = Alignment.Bottom, horizontalArrangement = Arrangement.spacedBy(18.dp)) {
                Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(STICK_BUTTON_SPACING)) {
                    StickButton("R3", button(11))
                    Stick(stick(2))
                }
                Box(Modifier.padding(bottom = RAISE)) {
                    Cross(
                        top = { faceButton("Y", Glyph.Triangle, 3) },
                        left = { faceButton("X", Glyph.Square, 2) },
                        right = { faceButton("B", Glyph.Circle, 1) },
                        bottom = { faceButton("A", Glyph.XMark, 0) },
                    )
                }
            }
        }
    }
}

@Composable
private fun Cross(
    top: @Composable () -> Unit,
    left: @Composable () -> Unit,
    right: @Composable () -> Unit,
    bottom: @Composable () -> Unit,
) {
    Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(2.dp)) {
        top()
        Row(horizontalArrangement = Arrangement.spacedBy(BUTTON_SIZE + 4.dp)) {
            left()
            right()
        }
        bottom()
    }
}

@Composable
private fun Shoulder(label: String, onPressedChange: (Boolean) -> Unit) {
    PadButton(onPressedChange, width = 68.dp, height = 38.dp) { Label(label, it, 14.sp) }
}

@Composable
private fun StickButton(label: String, onPressedChange: (Boolean) -> Unit) {
    PadButton(onPressedChange, STICK_BUTTON_SIZE) { Label(label, it, 13.sp) }
}

@Composable
private fun LabelButton(label: String, onPressedChange: (Boolean) -> Unit) {
    PadButton(onPressedChange) { Label(label, it, 18.sp) }
}

@Composable
private fun GlyphButton(glyph: Glyph, onPressedChange: (Boolean) -> Unit, size: Dp = BUTTON_SIZE, cornerRadius: Dp = size / 2) {
    PadButton(onPressedChange, size, cornerRadius = cornerRadius) { color ->
        Canvas(Modifier.size(16.dp)) { drawGlyph(glyph, color) }
    }
}

@Composable
private fun Label(text: String, color: Color, fontSize: TextUnit) {
    Text(text, color = color, fontSize = fontSize, fontWeight = FontWeight.SemiBold)
}

@Composable
private fun PadButton(
    onPressedChange: (Boolean) -> Unit,
    width: Dp = BUTTON_SIZE,
    height: Dp = width,
    cornerRadius: Dp = height / 2,
    content: @Composable (Color) -> Unit,
) {
    var isPressed by remember { mutableStateOf(false) }
    val view = LocalView.current
    val shape = RoundedCornerShape(cornerRadius)
    Box(
        Modifier
            .size(width, height)
            .clip(shape)
            .background(if (isPressed) Color.White else IDLE_FILL)
            .border(1.dp, if (isPressed) Color.Transparent else IDLE_STROKE, shape)
            .pointerInput(Unit) {
                awaitEachGesture {
                    awaitFirstDown(requireUnconsumed = false)
                    view.performHapticFeedback(HapticFeedbackConstants.VIRTUAL_KEY)
                    isPressed = true
                    onPressedChange(true)
                    waitForUpOrCancellation()
                    isPressed = false
                    onPressedChange(false)
                }
            },
        contentAlignment = Alignment.Center,
    ) {
        content(if (isPressed) Palette.background else IDLE_CONTENT)
    }
}

@Composable
private fun Stick(onMove: (Float, Float) -> Unit, modifier: Modifier = Modifier) {
    var knobOffset by remember { mutableStateOf<Offset?>(null) }
    Canvas(
        modifier
            .size(STICK_SIZE)
            .pointerInput(Unit) {
                val radius = size.width / 2f
                val center = Offset(radius, radius)

                fun moveTo(position: Offset) {
                    val offset = position - center
                    val distance = offset.getDistance()
                    val clampedOffset = if (distance > radius) offset * (radius / distance) else offset
                    knobOffset = clampedOffset
                    onMove(clampedOffset.x / radius, clampedOffset.y / radius)
                }

                awaitEachGesture {
                    val down = awaitFirstDown(requireUnconsumed = false)
                    moveTo(down.position)
                    drag(down.id) { change ->
                        moveTo(change.position)
                        change.consume()
                    }
                    knobOffset = null
                    onMove(0f, 0f)
                }
            },
    ) {
        val radius = size.minDimension / 2
        val lineWidth = 1.dp.toPx()
        drawCircle(Color.White.copy(alpha = 0.06f))
        drawCircle(IDLE_STROKE, style = Stroke(lineWidth))
        drawCircle(Color.White.copy(alpha = 0.08f), radius * 0.55f, style = Stroke(lineWidth))
        drawCircle(if (knobOffset == null) Color.White.copy(alpha = 0.28f) else Color.White, radius / 2, center + (knobOffset ?: Offset.Zero))
    }
}

private fun DrawScope.drawGlyph(glyph: Glyph, color: Color) {
    val unit = size.minDimension / 2
    val stroke = Stroke(width = 2.dp.toPx(), cap = StrokeCap.Round, join = StrokeJoin.Round)
    when (glyph) {
        Glyph.View -> {
            val rectangleSize = Size(unit * 1.3f, unit * 1.05f)
            val cornerRadius = CornerRadius(unit * 0.2f)
            drawRoundRect(color, center + Offset(-unit * 0.85f, -unit * 0.75f), rectangleSize, cornerRadius, stroke)
            drawRoundRect(color, center + Offset(-unit * 0.45f, -unit * 0.3f), rectangleSize, cornerRadius, stroke)
        }
        Glyph.Menu -> for (y in listOf(-0.6f, 0f, 0.6f)) {
            drawLine(color, center + Offset(-unit * 0.8f, unit * y), center + Offset(unit * 0.8f, unit * y), stroke.width, StrokeCap.Round)
        }
        Glyph.Triangle -> {
            val triangle = Path().apply {
                moveTo(center.x, center.y - unit * 0.8f)
                lineTo(center.x + unit * 0.85f, center.y + unit * 0.65f)
                lineTo(center.x - unit * 0.85f, center.y + unit * 0.65f)
                close()
            }
            drawPath(triangle, color, style = stroke)
        }
        Glyph.Square -> drawRect(color, center - Offset(unit * 0.7f, unit * 0.7f), Size(unit * 1.4f, unit * 1.4f), style = stroke)
        Glyph.Circle -> drawCircle(color, unit * 0.8f, center, style = stroke)
        Glyph.XMark -> {
            drawLine(color, center + Offset(-unit * 0.7f, -unit * 0.7f), center + Offset(unit * 0.7f, unit * 0.7f), stroke.width, StrokeCap.Round)
            drawLine(color, center + Offset(unit * 0.7f, -unit * 0.7f), center + Offset(-unit * 0.7f, unit * 0.7f), stroke.width, StrokeCap.Round)
        }
        else -> rotate(glyph.degrees) {
            val chevron = Path().apply {
                moveTo(center.x - unit * 0.7f, center.y + unit * 0.35f)
                lineTo(center.x, center.y - unit * 0.35f)
                lineTo(center.x + unit * 0.7f, center.y + unit * 0.35f)
            }
            drawPath(chevron, color, style = stroke)
        }
    }
}
