import AVFoundation
import CoreMedia
import UIKit
import os

private let receiveTimeoutMs: UInt32 = 100
private let audioPrebufferSeconds = 0.06
private let audioMaxQueuedSeconds = 0.15

final class VideoView: UIView {
    override class var layerClass: AnyClass { AVSampleBufferDisplayLayer.self }
    var displayLayer: AVSampleBufferDisplayLayer { layer as! AVSampleBufferDisplayLayer }
}

final class GameStream: ObservableObject {
    let connection: Connection
    let gamepads: Gamepads
    let videoView = VideoView()
    @Published var error: String?
    @Published var framesPerSecond = 0
    @Published var latencyMs = 0
    private let isRunning = OSAllocatedUnfairLock(initialState: true)
    private let threads = DispatchGroup()

    init(connection: Connection) {
        self.connection = connection
        gamepads = Gamepads(connection: connection)
        videoView.displayLayer.videoGravity = .resizeAspect
        start(playVideo)
        start(playAudio)
        start(forwardRumble)
    }

    func close() {
        isRunning.withLock { $0 = false }
        threads.wait()
        gamepads.stop()
    }

    private var shouldRun: Bool { isRunning.withLock { $0 } }

    private func start(_ body: @escaping () throws -> Void) {
        threads.enter()
        let thread = Thread {
            do {
                try body()
            } catch let PlayError.Failed(message) {
                DispatchQueue.main.async { self.error = message }
            } catch {
                DispatchQueue.main.async { self.error = "\(error)" }
            }
            self.threads.leave()
        }
        thread.qualityOfService = .userInteractive
        thread.start()
    }

    private func playVideo() throws {
        let renderer = videoView.displayLayer.sampleBufferRenderer
        var formatDescription: CMVideoFormatDescription?
        var windowStart = ContinuousClock.now
        var windowFrameCount = 0
        var windowLatencyUs: Int64 = 0
        while shouldRun {
            guard let frame = try connection.receiveVideo(timeoutMs: receiveTimeoutMs) else { continue }
            if renderer.status == .failed {
                renderer.flush()
                formatDescription = nil
            }
            if frame.isKeyframe, let format = videoFormat(keyframe: frame.data) {
                formatDescription = makeFormatDescription(format)
            }
            guard let formatDescription else {
                try connection.requestKeyframe()
                continue
            }
            let isH264 = CMFormatDescriptionGetMediaSubType(formatDescription) == kCMVideoCodecType_H264
            let sample = isH264 ? h264LengthPrefixed(accessUnit: frame.data) : frame.data
            renderer.enqueue(makeSampleBuffer(sample, isKeyframe: frame.isKeyframe, formatDescription))
            windowFrameCount += 1
            windowLatencyUs += Int64(fabricNowUs()) - Int64(frame.captureTimeUs)
            if windowStart.duration(to: .now) >= .seconds(1) {
                let framesPerSecond = windowFrameCount
                let latencyMs = Int(windowLatencyUs / Int64(windowFrameCount) / 1000)
                DispatchQueue.main.async {
                    self.framesPerSecond = framesPerSecond
                    self.latencyMs = latencyMs
                }
                windowStart = .now
                windowFrameCount = 0
                windowLatencyUs = 0
            }
        }
    }

    private func playAudio() throws {
        let session = AVAudioSession.sharedInstance()
        try session.setCategory(.playback)
        try session.setPreferredIOBufferDuration(0.005)
        try session.setActive(true)
        let engine = AVAudioEngine()
        let player = AVAudioPlayerNode()
        engine.attach(player)
        var format: AVAudioFormat?
        var queuedSampleCount = OSAllocatedUnfairLock(initialState: 0)
        var pendingBuffers: [AVAudioPCMBuffer] = []
        while shouldRun {
            guard let frame = try connection.receiveAudio(timeoutMs: receiveTimeoutMs) else { continue }
            if format == nil {
                format = AVAudioFormat(commonFormat: .pcmFormatFloat32, sampleRate: Double(frame.sampleRate), channels: AVAudioChannelCount(frame.channelCount), interleaved: false)
                engine.connect(player, to: engine.mainMixerNode, format: format)
            }
            if !engine.isRunning {
                try engine.start()
                player.play()
                queuedSampleCount = OSAllocatedUnfairLock(initialState: 0)
            }
            pendingBuffers.append(makeAudioBuffer(frame, format!))
            let sampleRate = Double(frame.sampleRate)
            let queuedSamples = queuedSampleCount.withLock { $0 }
            let pendingSamples = pendingBuffers.reduce(0) { $0 + Int($1.frameLength) }
            if queuedSamples == 0 && Double(pendingSamples) < audioPrebufferSeconds * sampleRate { continue }
            if Double(queuedSamples) > audioMaxQueuedSeconds * sampleRate {
                pendingBuffers.removeAll()
                continue
            }
            for buffer in pendingBuffers {
                let sampleCount = Int(buffer.frameLength)
                queuedSampleCount.withLock { $0 += sampleCount }
                player.scheduleBuffer(buffer, completionCallbackType: .dataPlayedBack) { [queuedSampleCount] _ in queuedSampleCount.withLock { $0 -= sampleCount } }
            }
            pendingBuffers.removeAll()
        }
        engine.stop()
    }

    private func forwardRumble() throws {
        while shouldRun {
            if let rumble = try connection.receiveRumble(timeoutMs: receiveTimeoutMs) {
                DispatchQueue.main.async { try? self.gamepads.rumble(rumble) }
            }
        }
    }
}

private func makeFormatDescription(_ format: VideoFormat) -> CMVideoFormatDescription {
    var formatDescription: CMVideoFormatDescription?
    switch format {
    case let .av1(width, height, codecConfiguration):
        let extensions = [kCMFormatDescriptionExtension_SampleDescriptionExtensionAtoms: ["av1C": codecConfiguration]] as CFDictionary
        CMVideoFormatDescriptionCreate(allocator: nil, codecType: kCMVideoCodecType_AV1, width: Int32(width), height: Int32(height), extensions: extensions, formatDescriptionOut: &formatDescription)
    case let .h264(_, _, sequenceParameterSet, pictureParameterSet):
        sequenceParameterSet.withUnsafeBytes { sequenceBytes in
            pictureParameterSet.withUnsafeBytes { pictureBytes in
                let parameterSets = [sequenceBytes, pictureBytes].map { $0.bindMemory(to: UInt8.self).baseAddress! }
                let parameterSetSizes = [sequenceBytes.count, pictureBytes.count]
                _ = CMVideoFormatDescriptionCreateFromH264ParameterSets(allocator: nil, parameterSetCount: 2, parameterSetPointers: parameterSets, parameterSetSizes: parameterSetSizes, nalUnitHeaderLength: 4, formatDescriptionOut: &formatDescription)
            }
        }
    }
    return formatDescription!
}

private func makeSampleBuffer(_ data: Data, isKeyframe: Bool, _ formatDescription: CMVideoFormatDescription) -> CMSampleBuffer {
    var blockBuffer: CMBlockBuffer?
    CMBlockBufferCreateWithMemoryBlock(allocator: nil, memoryBlock: nil, blockLength: data.count, blockAllocator: nil, customBlockSource: nil, offsetToData: 0, dataLength: data.count, flags: kCMBlockBufferAssureMemoryNowFlag, blockBufferOut: &blockBuffer)
    data.withUnsafeBytes { bytes in
        _ = CMBlockBufferReplaceDataBytes(with: bytes.baseAddress!, blockBuffer: blockBuffer!, offsetIntoDestination: 0, dataLength: bytes.count)
    }
    var sampleBuffer: CMSampleBuffer?
    var sampleSize = data.count
    CMSampleBufferCreateReady(allocator: nil, dataBuffer: blockBuffer, formatDescription: formatDescription, sampleCount: 1, sampleTimingEntryCount: 0, sampleTimingArray: nil, sampleSizeEntryCount: 1, sampleSizeArray: &sampleSize, sampleBufferOut: &sampleBuffer)
    let attachments = CMSampleBufferGetSampleAttachmentsArray(sampleBuffer!, createIfNecessary: true)! as NSArray
    let attachment = attachments[0] as! NSMutableDictionary
    attachment[kCMSampleAttachmentKey_DisplayImmediately] = true
    attachment[kCMSampleAttachmentKey_NotSync] = !isKeyframe
    return sampleBuffer!
}

private func makeAudioBuffer(_ frame: AudioFrame, _ format: AVAudioFormat) -> AVAudioPCMBuffer {
    let channelCount = Int(frame.channelCount)
    let sampleCount = frame.pcmBytes.count / MemoryLayout<Int16>.size
    let frameLength = sampleCount / channelCount
    let buffer = AVAudioPCMBuffer(pcmFormat: format, frameCapacity: AVAudioFrameCount(frameLength))!
    buffer.frameLength = AVAudioFrameCount(frameLength)
    let channels = buffer.floatChannelData!
    frame.pcmBytes.withUnsafeBytes { bytes in
        for index in 0..<sampleCount {
            let sample = Int16(littleEndian: bytes.loadUnaligned(fromByteOffset: index * MemoryLayout<Int16>.size, as: Int16.self))
            channels[index % channelCount][index / channelCount] = Float(sample) / Float(Int16.max)
        }
    }
    return buffer
}
