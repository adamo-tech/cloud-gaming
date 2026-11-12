import SwiftUI

struct PlayView: View {
    @ObservedObject var stream: GameStream
    @ObservedObject var gamepads: Gamepads
    let tabletopFold: CGRect?
    let onExit: () -> Void
    @State private var showsTouchControls = true
    @AppStorage(phoneVibrationKey) private var isPhoneVibrationEnabled = true
    @State private var isMenuOpen = false
    @State private var showsHint = true

    var body: some View {
        ZStack {
            Color.black.ignoresSafeArea()
            Backdrop()
                .padding(.top, tabletopFold?.maxY ?? 0)
                .ignoresSafeArea()
            VideoLayer(view: stream.videoView)
                .frame(height: tabletopFold?.minY)
                .frame(maxHeight: .infinity, alignment: .top)
                .ignoresSafeArea()
            Color.clear
                .contentShape(Rectangle())
                .ignoresSafeArea()
                .onLongPressGesture(minimumDuration: 0.6) { isMenuOpen = true }
            if showsTouchControls && !isMenuOpen {
                if let tabletopFold {
                    TouchController(gamepads: gamepads, isStacked: true)
                        .padding(.top, tabletopFold.maxY)
                        .ignoresSafeArea(edges: .top)
                } else {
                    TouchController(gamepads: gamepads)
                }
            }
            if showsHint {
                Text("Hold anywhere for the menu")
                    .font(.system(size: 13, weight: .medium))
                    .foregroundStyle(Color.foreground)
                    .padding(.horizontal, 14)
                    .padding(.vertical, 8)
                    .glass(cornerRadius: 999)
                    .frame(maxHeight: .infinity, alignment: .top)
                    .padding(.top, 12)
                    .transition(.opacity)
                    .allowsHitTesting(false)
            }
            if isMenuOpen {
                menu
            }
            if let error = stream.error {
                errorCard(error)
            }
        }
        .statusBarHidden()
        .persistentSystemOverlays(.hidden)
        .onChange(of: gamepads.hasPhysicalGamepad, initial: true) { _, hasPhysicalGamepad in showsTouchControls = !hasPhysicalGamepad }
        .task {
            try? await Task.sleep(for: .seconds(4))
            withAnimation(.easeOut(duration: 0.6)) { showsHint = false }
        }
    }

    private var menu: some View {
        ZStack {
            Color.black.opacity(0.55).ignoresSafeArea().onTapGesture { isMenuOpen = false }
            VStack(alignment: .leading, spacing: 18) {
                HStack(spacing: 12) {
                    Logomark().fill(.white).frame(width: 28, height: 24)
                    Text("Adamo Play").font(.display(26)).foregroundStyle(Color.foreground)
                    Spacer()
                    Circle().fill(Color.success).frame(width: 8, height: 8)
                    Text("Connected").font(.system(size: 13)).foregroundStyle(Color.muted)
                }
                VStack(spacing: 0) {
                    setting("Touch controls", detail: "Turn off when a controller connects", isOn: $showsTouchControls)
                    Divider().overlay(Color.glassBorder)
                    setting("Phone vibration", detail: "Rumble on the phone when no controller is connected", isOn: $isPhoneVibrationEnabled)
                }
                .padding(.horizontal, 16)
                .background(Color.white.opacity(0.04), in: RoundedRectangle(cornerRadius: 14, style: .continuous))
                .overlay(RoundedRectangle(cornerRadius: 14, style: .continuous).stroke(Color.glassBorder, lineWidth: 1))
                HStack(spacing: 12) {
                    Button("Disconnect", role: .destructive, action: onExit)
                        .font(.system(size: 16, weight: .semibold))
                        .foregroundStyle(Color.danger)
                        .frame(maxWidth: .infinity, minHeight: 48)
                        .background(Color.danger.opacity(0.12), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
                    Button("Resume") { isMenuOpen = false }.buttonStyle(PrimaryButtonStyle())
                }
            }
            .padding(24)
            .frame(width: 420)
            .glass(cornerRadius: 22)
        }
        .transition(.opacity)
    }

    private func setting(_ title: String, detail: String?, isOn: Binding<Bool>) -> some View {
        Button { isOn.wrappedValue.toggle() } label: {
            Toggle(isOn: isOn) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(title).font(.system(size: 15, weight: .medium)).foregroundStyle(Color.foreground)
                    if let detail {
                        Text(detail).font(.system(size: 12)).foregroundStyle(Color.muted)
                    }
                }
            }
            .tint(Color.success)
            .allowsHitTesting(false)
            .padding(.vertical, 12)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }

    private func errorCard(_ error: String) -> some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Connection lost").font(.display(24)).foregroundStyle(Color.foreground)
            Text(error).font(.system(size: 14)).foregroundStyle(Color.muted)
            Button("Back", action: onExit).buttonStyle(PrimaryButtonStyle())
        }
        .padding(24)
        .frame(width: 380)
        .glass(cornerRadius: 22)
    }
}

struct FoldReader: UIViewRepresentable {
    @Binding var fold: CGRect?

    func makeUIView(context: Context) -> FoldView {
        let view = FoldView()
        view.isUserInteractionEnabled = false
        view.onFoldChange = { fold in DispatchQueue.main.async { self.fold = fold } }
        if #available(iOS 27.1, *) {
            view.addInteraction(UIHingeInteraction { [weak view] _, update in view?.isUnfolded = [.partiallyOpen, .fullyOpen].contains(update.hinge?.status) })
        }
        return view
    }

    func updateUIView(_ view: FoldView, context: Context) {}
}

final class FoldView: UIView {
    var onFoldChange: (CGRect?) -> Void = { _ in }
    var isUnfolded = false {
        didSet { setNeedsLayout() }
    }

    override func layoutSubviews() {
        super.layoutSubviews()
        if #available(iOS 27.1, *) {
            onFoldChange(isUnfolded ? reservedRegions(kind: .division, options: .includeInactive).first?.frame : nil)
        }
    }
}

private struct VideoLayer: UIViewRepresentable {
    let view: VideoView

    func makeUIView(context: Context) -> VideoView { view }

    func updateUIView(_ view: VideoView, context: Context) {}
}
