import SwiftUI

private let buttonSize: CGFloat = 44
private let stickSize: CGFloat = 116
private let raise: CGFloat = 72

struct TouchController: View {
    let gamepads: Gamepads

    var body: some View {
        VStack {
            HStack {
                HStack(spacing: 10) {
                    Shoulder(label: "LT") { gamepads.setTouchTrigger(6, isPressed: $0) }
                    Shoulder(label: "LB") { gamepads.setTouchButton(4, isPressed: $0) }
                }
                Spacer()
                HStack(spacing: 10) {
                    Shoulder(label: "RB") { gamepads.setTouchButton(5, isPressed: $0) }
                    Shoulder(label: "RT") { gamepads.setTouchTrigger(7, isPressed: $0) }
                }
            }
            Spacer()
            HStack(alignment: .bottom) {
                HStack(alignment: .bottom, spacing: 18) {
                    Stick { gamepads.setTouchStick(xAxis: 0, x: $0, y: $1) }.padding(.bottom, raise)
                    Cross(
                        top: PadButton(symbol: "chevron.up", cornerRadius: 10) { gamepads.setTouchButton(12, isPressed: $0) },
                        left: PadButton(symbol: "chevron.left", cornerRadius: 10) { gamepads.setTouchButton(14, isPressed: $0) },
                        right: PadButton(symbol: "chevron.right", cornerRadius: 10) { gamepads.setTouchButton(15, isPressed: $0) },
                        bottom: PadButton(symbol: "chevron.down", cornerRadius: 10) { gamepads.setTouchButton(13, isPressed: $0) }
                    )
                }
                Spacer()
                HStack(spacing: 14) {
                    PadButton(symbol: "rectangle.on.rectangle", size: 36) { gamepads.setTouchButton(8, isPressed: $0) }
                    PadButton(isLogomark: true, size: 40) { gamepads.setTouchButton(16, isPressed: $0) }
                    PadButton(symbol: "line.3.horizontal", size: 36) { gamepads.setTouchButton(9, isPressed: $0) }
                }
                Spacer()
                HStack(alignment: .bottom, spacing: 18) {
                    Stick { gamepads.setTouchStick(xAxis: 2, x: $0, y: $1) }
                    Cross(
                        top: PadButton(text: "Y") { gamepads.setTouchButton(3, isPressed: $0) },
                        left: PadButton(text: "X") { gamepads.setTouchButton(2, isPressed: $0) },
                        right: PadButton(text: "B") { gamepads.setTouchButton(1, isPressed: $0) },
                        bottom: PadButton(text: "A") { gamepads.setTouchButton(0, isPressed: $0) }
                    )
                    .padding(.bottom, raise)
                }
            }
        }
        .padding(.horizontal, 24)
        .padding(.vertical, 16)
    }
}

private struct Cross: View {
    let top: PadButton
    let left: PadButton
    let right: PadButton
    let bottom: PadButton

    var body: some View {
        VStack(spacing: 2) {
            top
            HStack(spacing: buttonSize + 4) {
                left
                right
            }
            bottom
        }
    }
}

private struct PadButton: View {
    var text: String?
    var symbol: String?
    var isLogomark = false
    var size = buttonSize
    var cornerRadius: CGFloat?
    let onPressedChange: (Bool) -> Void
    @State private var isPressed = false

    var body: some View {
        let shape = RoundedRectangle(cornerRadius: cornerRadius ?? size / 2, style: .continuous)
        label
            .foregroundStyle(isPressed ? Color.background : Color.white.opacity(0.9))
            .frame(width: size, height: size)
            .background(isPressed ? Color.white : Color.white.opacity(0.1), in: shape)
            .overlay(shape.stroke(Color.white.opacity(isPressed ? 0 : 0.22), lineWidth: 1))
            .contentShape(shape)
            .overlay(PressArea { isPressed in
                self.isPressed = isPressed
                onPressedChange(isPressed)
            })
            .sensoryFeedback(.impact(weight: .light), trigger: isPressed) { _, isPressed in isPressed }
    }

    @ViewBuilder private var label: some View {
        if let text {
            Text(text).font(.system(size: size * 0.4, weight: .semibold))
        } else if let symbol {
            Image(systemName: symbol).font(.system(size: size * 0.36, weight: .semibold))
        } else if isLogomark {
            Logomark().frame(width: size * 0.45, height: size * 0.45)
        }
    }
}

private struct Shoulder: View {
    let label: String
    let onPressedChange: (Bool) -> Void
    @State private var isPressed = false

    var body: some View {
        let shape = Capsule()
        Text(label)
            .font(.system(size: 14, weight: .semibold))
            .foregroundStyle(isPressed ? Color.background : Color.white.opacity(0.9))
            .frame(width: 68, height: 38)
            .background(isPressed ? Color.white : Color.white.opacity(0.1), in: shape)
            .overlay(shape.stroke(Color.white.opacity(isPressed ? 0 : 0.22), lineWidth: 1))
            .contentShape(shape)
            .overlay(PressArea { isPressed in
                self.isPressed = isPressed
                onPressedChange(isPressed)
            })
            .sensoryFeedback(.impact(weight: .light), trigger: isPressed) { _, isPressed in isPressed }
    }
}

private struct PressArea: UIViewRepresentable {
    let onPressedChange: (Bool) -> Void

    func makeUIView(context: Context) -> PressView {
        PressView()
    }

    func updateUIView(_ view: PressView, context: Context) {
        view.onPressedChange = onPressedChange
    }
}

private final class PressView: UIView {
    var onPressedChange: (Bool) -> Void = { _ in }

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        onPressedChange(true)
    }

    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
        onPressedChange(false)
    }

    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) {
        onPressedChange(false)
    }
}

private struct Stick: View {
    let onMove: (Double, Double) -> Void
    @GestureState private var knobOffset: CGSize?

    var body: some View {
        let radius = stickSize / 2
        let offset = knobOffset ?? .zero
        ZStack {
            Circle().fill(Color.white.opacity(0.06))
            Circle().stroke(Color.white.opacity(0.22), lineWidth: 1)
            Circle().stroke(Color.white.opacity(0.08), lineWidth: 1).padding(radius * 0.45)
            Circle()
                .fill(knobOffset == nil ? Color.white.opacity(0.28) : Color.white)
                .frame(width: radius, height: radius)
                .offset(offset)
        }
        .frame(width: stickSize, height: stickSize)
        .contentShape(Circle())
        .gesture(
            DragGesture(minimumDistance: 0)
                .updating($knobOffset) { value, knobOffset, _ in
                    let x = value.location.x - radius
                    let y = value.location.y - radius
                    let scale = min(1, radius / max(hypot(x, y), 1))
                    knobOffset = CGSize(width: x * scale, height: y * scale)
                }
        )
        .onChange(of: knobOffset) { _, knobOffset in onMove(Double((knobOffset?.width ?? 0) / radius), Double((knobOffset?.height ?? 0) / radius)) }
    }
}
