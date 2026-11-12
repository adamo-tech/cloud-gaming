import SwiftUI

struct ConnectView: View {
    let onConnect: (Connection) -> Void
    @AppStorage("address") private var address = ""
    @AppStorage("pairingKey") private var pairingKey = ""
    @State private var isConnecting = false
    @State private var error: String?
    @State private var isPairingKeyVisible = false
    @FocusState private var focusedField: Field?
    @Environment(\.verticalSizeClass) private var verticalSizeClass

    private enum Field { case address, pairingKey }

    var body: some View {
        ZStack {
            Color.background.ignoresSafeArea()
            RadialGradient(colors: [.white.opacity(0.07), .clear], center: .topLeading, startRadius: 0, endRadius: 520).ignoresSafeArea()
            Logomark().fill(.white.opacity(0.025)).frame(width: 520).offset(x: -260, y: 120)
            let layout = verticalSizeClass == .compact ? AnyLayout(HStackLayout(spacing: 48)) : AnyLayout(VStackLayout(alignment: .leading, spacing: 32))
            layout {
                VStack(alignment: .leading, spacing: 16) {
                    Logomark().fill(.white).frame(width: 44, height: 36)
                    Text("Adamo Play").font(.display(44)).foregroundStyle(Color.foreground)
                    Text("Play the games on your PC from anywhere, with a controller or the touch screen.")
                        .font(.system(size: 16))
                        .foregroundStyle(Color.muted)
                        .fixedSize(horizontal: false, vertical: true)
                }
                .frame(maxWidth: 320, alignment: .leading)
                form
            }
            .padding(32)
        }
        .onTapGesture { focusedField = nil }
    }

    private var form: some View {
        VStack(alignment: .leading, spacing: 16) {
            field("Gaming PC") {
                TextField("", text: $address, prompt: Text("192.168.1.20:17447").foregroundStyle(Color.muted.opacity(0.6)))
                    .keyboardType(.numbersAndPunctuation)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                    .focused($focusedField, equals: .address)
                    .submitLabel(.next)
                    .onSubmit { focusedField = .pairingKey }
            }
            field("Pairing key") {
                HStack(spacing: 8) {
                    Group {
                        if isPairingKeyVisible {
                            TextField("", text: $pairingKey, prompt: pairingKeyPrompt)
                                .font(.system(size: 14, design: .monospaced))
                                .textInputAutocapitalization(.never)
                                .autocorrectionDisabled()
                        } else {
                            SecureField("", text: $pairingKey, prompt: pairingKeyPrompt)
                        }
                    }
                    .focused($focusedField, equals: .pairingKey)
                    .submitLabel(.go)
                    .onSubmit(play)
                    Button { isPairingKeyVisible.toggle() } label: {
                        Image(systemName: isPairingKeyVisible ? "eye.slash" : "eye").foregroundStyle(Color.muted)
                    }
                }
            }
            if let error {
                Text(error).font(.system(size: 13)).foregroundStyle(Color.danger).fixedSize(horizontal: false, vertical: true)
            }
            Button(action: play) {
                if isConnecting {
                    ProgressView().tint(Color.background)
                } else {
                    Text("Play")
                }
            }
            .buttonStyle(PrimaryButtonStyle())
            .disabled(address.isEmpty || pairingKey.isEmpty || isConnecting)
            .padding(.top, 4)
        }
        .padding(24)
        .frame(maxWidth: 360)
        .glass(cornerRadius: 20)
    }

    private var pairingKeyPrompt: Text {
        Text("Printed by the host").foregroundStyle(Color.muted.opacity(0.6))
    }

    private func field(_ label: String, @ViewBuilder input: () -> some View) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(label).font(.system(size: 12, weight: .medium)).foregroundStyle(Color.muted)
            input()
                .font(.system(size: 15))
                .foregroundStyle(Color.foreground)
                .padding(.horizontal, 12)
                .frame(height: 44)
                .background(Color.white.opacity(0.04), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
                .overlay(RoundedRectangle(cornerRadius: 10, style: .continuous).stroke(Color.glassBorder, lineWidth: 1))
        }
    }

    private func play() {
        focusedField = nil
        isConnecting = true
        error = nil
        let address = address
        let pairingKey = pairingKey
        Task {
            do {
                let connection = try await Task.detached { try Connection.connect(address: address, pairingKey: pairingKey) }.value
                onConnect(connection)
            } catch let PlayError.Failed(message) {
                error = message
            } catch {
                self.error = "\(error)"
            }
            isConnecting = false
        }
    }
}
