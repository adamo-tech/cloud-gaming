import SwiftUI

extension Color {
    static let background = Color(red: 0.035, green: 0.035, blue: 0.043)
    static let card = Color(red: 0.059, green: 0.059, blue: 0.071)
    static let glass = Color.white.opacity(0.06)
    static let glassBorder = Color.white.opacity(0.12)
    static let foreground = Color(red: 0.91, green: 0.906, blue: 0.953)
    static let muted = Color(red: 0.631, green: 0.631, blue: 0.667)
    static let success = Color(red: 0.525, green: 0.937, blue: 0.675)
    static let danger = Color(red: 0.937, green: 0.267, blue: 0.267)
}

extension Font {
    static func display(_ size: CGFloat) -> Font { .system(size: size, weight: .semibold, design: .serif) }
}

struct Logomark: Shape {
    func path(in rect: CGRect) -> Path {
        var path = Path()
        path.move(to: CGPoint(x: 72.5333, y: 0))
        path.addCurve(to: CGPoint(x: 85.3333, y: 12.72), control1: CGPoint(x: 79.6026, y: 0), control2: CGPoint(x: 85.3333, y: 5.69494))
        path.addLine(to: CGPoint(x: 85.3333, y: 40.28))
        path.addCurve(to: CGPoint(x: 98.1333, y: 53), control1: CGPoint(x: 85.3333, y: 47.3051), control2: CGPoint(x: 91.0641, y: 53))
        path.addLine(to: CGPoint(x: 115.2, y: 53))
        path.addCurve(to: CGPoint(x: 128, y: 65.72), control1: CGPoint(x: 122.269, y: 53), control2: CGPoint(x: 128, y: 58.6949))
        path.addLine(to: CGPoint(x: 128, y: 93.28))
        path.addCurve(to: CGPoint(x: 115.2, y: 106), control1: CGPoint(x: 128, y: 100.305), control2: CGPoint(x: 122.269, y: 106))
        path.addLine(to: CGPoint(x: 97.3334, y: 106))
        path.addCurve(to: CGPoint(x: 85.3333, y: 94), control1: CGPoint(x: 90.7059, y: 106), control2: CGPoint(x: 85.3333, y: 100.627))
        path.addLine(to: CGPoint(x: 85.3333, y: 65))
        path.addCurve(to: CGPoint(x: 73.3333, y: 53), control1: CGPoint(x: 85.3333, y: 58.3726), control2: CGPoint(x: 79.9608, y: 53))
        path.addLine(to: CGPoint(x: 54.6667, y: 53))
        path.addCurve(to: CGPoint(x: 42.6667, y: 65), control1: CGPoint(x: 48.0393, y: 53), control2: CGPoint(x: 42.6667, y: 58.3726))
        path.addLine(to: CGPoint(x: 42.6667, y: 94))
        path.addCurve(to: CGPoint(x: 30.6667, y: 106), control1: CGPoint(x: 42.6667, y: 100.627), control2: CGPoint(x: 37.2941, y: 106))
        path.addLine(to: CGPoint(x: 12.8, y: 106))
        path.addCurve(to: CGPoint(x: 0, y: 93.28), control1: CGPoint(x: 5.73076, y: 106), control2: CGPoint(x: 0, y: 100.305))
        path.addLine(to: CGPoint(x: 0, y: 65.72))
        path.addCurve(to: CGPoint(x: 12.8, y: 53), control1: CGPoint(x: 0, y: 58.6949), control2: CGPoint(x: 5.73076, y: 53))
        path.addLine(to: CGPoint(x: 29.8667, y: 53))
        path.addCurve(to: CGPoint(x: 42.6667, y: 40.28), control1: CGPoint(x: 36.9359, y: 53), control2: CGPoint(x: 42.6667, y: 47.3051))
        path.addLine(to: CGPoint(x: 42.6667, y: 12.72))
        path.addCurve(to: CGPoint(x: 55.4667, y: 0), control1: CGPoint(x: 42.6667, y: 5.69494), control2: CGPoint(x: 48.3974, y: 0))
        path.closeSubpath()
        let scale = min(rect.width / 128, rect.height / 106)
        return path.applying(CGAffineTransform(scaleX: scale, y: scale).translatedBy(x: (rect.width / scale - 128) / 2, y: (rect.height / scale - 106) / 2))
    }
}

struct GlassBackground: ViewModifier {
    let cornerRadius: CGFloat

    func body(content: Content) -> some View {
        content
            .background(.ultraThinMaterial.opacity(0.6), in: RoundedRectangle(cornerRadius: cornerRadius, style: .continuous))
            .background(Color.glass, in: RoundedRectangle(cornerRadius: cornerRadius, style: .continuous))
            .overlay(RoundedRectangle(cornerRadius: cornerRadius, style: .continuous).stroke(Color.glassBorder, lineWidth: 1))
    }
}

extension View {
    func glass(cornerRadius: CGFloat) -> some View { modifier(GlassBackground(cornerRadius: cornerRadius)) }
}

struct PrimaryButtonStyle: ButtonStyle {
    @Environment(\.isEnabled) private var isEnabled

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.system(size: 16, weight: .semibold))
            .foregroundStyle(Color.background)
            .frame(maxWidth: .infinity, minHeight: 48)
            .background(Color.white.opacity(configuration.isPressed ? 0.85 : 1), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
            .opacity(isEnabled ? 1 : 0.4)
    }
}
