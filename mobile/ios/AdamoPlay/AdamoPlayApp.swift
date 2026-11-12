import SwiftUI

@main
struct AdamoPlayApp: App {
    @UIApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @State private var stream: GameStream?
    @State private var fold: CGRect?

    init() {
        UserDefaults.standard.register(defaults: [phoneVibrationKey: true])
    }

    var body: some Scene {
        WindowGroup {
            let tabletopFold = fold.flatMap { $0.width > $0.height ? $0 : nil }
            Group {
                if let stream {
                    PlayView(stream: stream, gamepads: stream.gamepads, tabletopFold: tabletopFold) {
                        stream.close()
                        self.stream = nil
                    }
                } else {
                    ConnectView(tabletopFold: tabletopFold) { connection in stream = GameStream(connection: connection) }
                }
            }
            .background(FoldReader(fold: $fold).ignoresSafeArea())
            .preferredColorScheme(.dark)
            .animation(.easeOut(duration: 0.2), value: stream == nil)
            .onChange(of: stream == nil || fold != nil) { _, isFree in setOrientations(isFree ? .allButUpsideDown : .landscape) }
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
