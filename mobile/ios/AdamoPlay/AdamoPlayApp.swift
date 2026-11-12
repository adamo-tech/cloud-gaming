import SwiftUI

@main
struct AdamoPlayApp: App {
    @UIApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @State private var stream: GameStream?

    init() {
        UserDefaults.standard.register(defaults: [phoneVibrationKey: true])
    }

    var body: some Scene {
        WindowGroup {
            Group {
                if let stream {
                    PlayView(stream: stream, gamepads: stream.gamepads) {
                        stream.close()
                        self.stream = nil
                    }
                } else {
                    ConnectView { connection in stream = GameStream(connection: connection) }
                }
            }
            .preferredColorScheme(.dark)
            .animation(.easeOut(duration: 0.2), value: stream == nil)
            .onChange(of: stream == nil) { _, isDisconnected in setOrientations(isDisconnected ? .allButUpsideDown : .landscape) }
        }
    }
}

final class AppDelegate: NSObject, UIApplicationDelegate {
    static var orientations = UIInterfaceOrientationMask.allButUpsideDown

    func application(_ application: UIApplication, supportedInterfaceOrientationsFor window: UIWindow?) -> UIInterfaceOrientationMask {
        Self.orientations
    }
}

private func setOrientations(_ orientations: UIInterfaceOrientationMask) {
    AppDelegate.orientations = orientations
    let scene = UIApplication.shared.connectedScenes.first as? UIWindowScene
    scene?.keyWindow?.rootViewController?.setNeedsUpdateOfSupportedInterfaceOrientations()
    scene?.requestGeometryUpdate(.iOS(interfaceOrientations: orientations))
}
