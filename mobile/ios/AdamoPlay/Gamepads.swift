import CoreHaptics
import GameController

let phoneVibrationKey = "isPhoneVibrationEnabled"

final class Gamepads: ObservableObject {
    @Published private(set) var hasPhysicalGamepad = false
    private let connection: Connection
    private var touchAxes = [Double](repeating: 0, count: 6)
    private var touchButtons = [Int32](repeating: 0, count: 17)
    private var hapticEngines: [ObjectIdentifier?: CHHapticEngine] = [:]
    private var hapticPlayers: [ObjectIdentifier?: CHHapticPatternPlayer] = [:]
    private var observers: [NSObjectProtocol] = []

    init(connection: Connection) {
        self.connection = connection
        for name in [Notification.Name.GCControllerDidConnect, .GCControllerDidDisconnect] {
            observers.append(NotificationCenter.default.addObserver(forName: name, object: nil, queue: .main) { [weak self] _ in self?.update() })
        }
        update()
    }

    func stop() {
        observers.forEach(NotificationCenter.default.removeObserver)
    }

    func setTouchButton(_ index: Int, isPressed: Bool) {
        touchButtons[index] = isPressed ? 1 : 0
        sendTouch()
    }

    func setTouchTrigger(_ index: Int, isPressed: Bool) {
        touchButtons[index] = isPressed ? 1 : 0
        touchAxes[index - 2] = isPressed ? 1 : 0
        sendTouch()
    }

    func setTouchStick(xAxis: Int, x: Double, y: Double) {
        touchAxes[xAxis] = x
        touchAxes[xAxis + 1] = y
        sendTouch()
    }

    func rumble(_ rumble: GamepadRumble) throws {
        let controllers = GCController.controllers()
        let index = Int(rumble.gamepadIndex)
        let controller = index < controllers.count ? controllers[index] : nil
        let key = controller.map(ObjectIdentifier.init)
        try hapticPlayers[key]?.stop(atTime: CHHapticTimeImmediate)
        let intensity = max(rumble.strongMagnitude, rumble.weakMagnitude)
        if intensity == 0 || (controller == nil && !UserDefaults.standard.bool(forKey: phoneVibrationKey)) { return }
        let engine = try hapticEngines[key] ?? makeHapticEngine(controller)
        hapticEngines[key] = engine
        let event = CHHapticEvent(eventType: .hapticContinuous, parameters: [CHHapticEventParameter(parameterID: .hapticIntensity, value: intensity)], relativeTime: 0, duration: TimeInterval(rumble.durationMs) / 1000)
        let player = try engine.makePlayer(with: CHHapticPattern(events: [event], parameters: []))
        try player.start(atTime: CHHapticTimeImmediate)
        hapticPlayers[key] = player
    }

    private func update() {
        let controllers = GCController.controllers()
        hasPhysicalGamepad = !controllers.isEmpty
        for (index, controller) in controllers.enumerated() {
            controller.extendedGamepad?.valueChangedHandler = { [weak self] gamepad, _ in self?.send(gamepad, index: index) }
        }
    }

    private func sendTouch() {
        try? connection.sendGamepad(gamepadIndex: 0, axes: touchAxes, buttons: touchButtons)
    }

    private func send(_ gamepad: GCExtendedGamepad, index: Int) {
        let axes = [
            gamepad.leftThumbstick.xAxis.value, -gamepad.leftThumbstick.yAxis.value,
            gamepad.rightThumbstick.xAxis.value, -gamepad.rightThumbstick.yAxis.value,
            gamepad.leftTrigger.value, gamepad.rightTrigger.value,
        ].map(Double.init)
        let standardButtons: [GCControllerButtonInput?] = [
            gamepad.buttonA, gamepad.buttonB, gamepad.buttonX, gamepad.buttonY,
            gamepad.leftShoulder, gamepad.rightShoulder, gamepad.leftTrigger, gamepad.rightTrigger,
            gamepad.buttonOptions, gamepad.buttonMenu, gamepad.leftThumbstickButton, gamepad.rightThumbstickButton,
            gamepad.dpad.up, gamepad.dpad.down, gamepad.dpad.left, gamepad.dpad.right, gamepad.buttonHome,
        ]
        let buttons = standardButtons.map { Int32($0?.isPressed == true ? 1 : 0) }
        try? connection.sendGamepad(gamepadIndex: UInt8(index), axes: axes, buttons: buttons)
    }
}

private func makeHapticEngine(_ controller: GCController?) throws -> CHHapticEngine {
    let engine = try controller?.haptics?.createEngine(withLocality: .default) ?? CHHapticEngine()
    try engine.start()
    return engine
}
