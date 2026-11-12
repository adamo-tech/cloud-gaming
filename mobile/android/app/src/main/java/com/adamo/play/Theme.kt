package com.adamo.play

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ProvideTextStyle
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlin.math.min

object Palette {
    val background = Color(0xFF09090B)
    val panel = Color(0xCC18181B)
    val glass = Color.White.copy(alpha = 0.06f)
    val glassBorder = Color.White.copy(alpha = 0.12f)
    val foreground = Color(0xFFE8E7F3)
    val muted = Color(0xFFA1A1AA)
    val success = Color(0xFF86EFAC)
    val danger = Color(0xFFEF4444)
}

fun displayStyle(fontSize: TextUnit) = TextStyle(fontFamily = FontFamily.Serif, fontWeight = FontWeight.SemiBold, fontSize = fontSize, color = Palette.foreground)

fun Modifier.glass(shape: Shape): Modifier = background(Palette.panel, shape).background(Palette.glass, shape).border(1.dp, Palette.glassBorder, shape)

@Composable
fun FilledButton(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    isEnabled: Boolean = true,
    containerColor: Color = Color.White,
    contentColor: Color = Palette.background,
    content: @Composable () -> Unit,
) {
    val shape = RoundedCornerShape(12.dp)
    Box(
        modifier
            .fillMaxWidth()
            .height(48.dp)
            .alpha(if (isEnabled) 1f else 0.4f)
            .clip(shape)
            .background(containerColor)
            .clickable(enabled = isEnabled, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        ProvideTextStyle(TextStyle(color = contentColor, fontSize = 16.sp, fontWeight = FontWeight.SemiBold), content)
    }
}

private val logomarkPath = Path().apply {
    moveTo(72.5333f, 0f)
    cubicTo(79.6026f, 0f, 85.3333f, 5.69494f, 85.3333f, 12.72f)
    lineTo(85.3333f, 40.28f)
    cubicTo(85.3333f, 47.3051f, 91.0641f, 53f, 98.1333f, 53f)
    lineTo(115.2f, 53f)
    cubicTo(122.269f, 53f, 128f, 58.6949f, 128f, 65.72f)
    lineTo(128f, 93.28f)
    cubicTo(128f, 100.305f, 122.269f, 106f, 115.2f, 106f)
    lineTo(97.3334f, 106f)
    cubicTo(90.7059f, 106f, 85.3333f, 100.627f, 85.3333f, 94f)
    lineTo(85.3333f, 65f)
    cubicTo(85.3333f, 58.3726f, 79.9608f, 53f, 73.3333f, 53f)
    lineTo(54.6667f, 53f)
    cubicTo(48.0393f, 53f, 42.6667f, 58.3726f, 42.6667f, 65f)
    lineTo(42.6667f, 94f)
    cubicTo(42.6667f, 100.627f, 37.2941f, 106f, 30.6667f, 106f)
    lineTo(12.8f, 106f)
    cubicTo(5.73076f, 106f, 0f, 100.305f, 0f, 93.28f)
    lineTo(0f, 65.72f)
    cubicTo(0f, 58.6949f, 5.73076f, 53f, 12.8f, 53f)
    lineTo(29.8667f, 53f)
    cubicTo(36.9359f, 53f, 42.6667f, 47.3051f, 42.6667f, 40.28f)
    lineTo(42.6667f, 12.72f)
    cubicTo(42.6667f, 5.69494f, 48.3974f, 0f, 55.4667f, 0f)
    close()
}

@Composable
fun Logomark(modifier: Modifier, color: Color = Color.White) {
    Canvas(modifier) {
        val logomarkScale = min(size.width / 128f, size.height / 106f)
        translate((size.width - 128f * logomarkScale) / 2, (size.height - 106f * logomarkScale) / 2) {
            scale(logomarkScale, pivot = Offset.Zero) { drawPath(logomarkPath, color) }
        }
    }
}
