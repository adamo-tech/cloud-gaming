import SwiftUI

struct ConnectView: View {
    let tabletopFold: CGRect?
    let onConnect: (Connection) -> Void
    @AppStorage("address") private var address = ""
    @AppStorage("pairingKey") private var pairingKey = ""
    @AppStorage("isThroughRelay") private var isThroughRelay = false
    @AppStorage("hostName") private var hostName = ""
    @AppStorage("apiKey") private var apiKey = ""
    @State private var isConnecting = false
    @State private var error: String?
    @State private var isPairingKeyVisible = false
    @FocusState private var focusedField: Field?
    @Environment(\.verticalSizeClass) private var verticalSizeClass

    private enum Field { case address, pairingKey }

    var body: some View {
        ZStack {
            Backdrop().ignoresSafeArea()
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
            Picker("Connection", selection: $isThroughRelay) {
                Text("Local network").tag(false)
                Text("Adamo relay").tag(true)
            }
            .pickerStyle(.segmented)
            field("Gaming PC") {
                TextField("", text: isThroughRelay ? $hostName : $address, prompt: Text(isThroughRelay ? "cloud-gaming" : "192.168.1.20:17447").foregroundStyle(Color.muted.opacity(0.6)))
                    .keyboardType(isThroughRelay ? .asciiCapable : .numbersAndPunctuation)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                    .focused($focusedField, equals: .address)
                    .submitLabel(.next)
                    .onSubmit { focusedField = .pairingKey }
            }
            field(isThroughRelay ? "API key" : "Pairing key") {
                HStack(spacing: 8) {
                    Group {
                        if isPairingKeyVisible {
                            TextField("", text: key, prompt: keyPrompt)
                                .font(.system(size: 14, design: .monospaced))
                                .textInputAutocapitalization(.never)
                                .autocorrectionDisabled()
                        } else {
                            SecureField("", text: key, prompt: keyPrompt)
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
            .disabled((isThroughRelay ? hostName.isEmpty || apiKey.isEmpty : address.isEmpty || pairingKey.isEmpty) || isConnecting)
            .padding(.top, 4)
        }
        .padding(24)
        .frame(maxWidth: 360)
        .glass(cornerRadius: 20)
    }

    private var key: Binding<String> {
        isThroughRelay ? $apiKey : $pairingKey
    }

    private var keyPrompt: Text {
        Text(isThroughRelay ? "From your Adamo organization" : "Printed by the host").foregroundStyle(Color.muted.opacity(0.6))
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
        let isThroughRelay = isThroughRelay
        let address = address
        let pairingKey = pairingKey
        let hostName = hostName
        let apiKey = apiKey
        let window = UIApplication.shared.connectedScenes.compactMap { ($0 as? UIWindowScene)?.keyWindow }.first!
        let scale = window.traitCollection.displayScale
        let videoSize = CGSize(width: window.bounds.width * scale, height: (tabletopFold?.minY ?? window.bounds.height) * scale)
        let screenWidth = UInt32(max(videoSize.width, videoSize.height))
        let screenHeight = UInt32(min(videoSize.width, videoSize.height))
        Task {
            do {
                let connection = try await Task.detached {
                    isThroughRelay
                        ? try Connection.connectThroughRelay(apiKey: apiKey, hostName: hostName, screenWidth: screenWidth, screenHeight: screenHeight)
                        : try Connection.connect(address: address, pairingKey: pairingKey, screenWidth: screenWidth, screenHeight: screenHeight)
                }.value
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
