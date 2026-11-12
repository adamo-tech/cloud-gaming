package com.adamo.play

import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioTrack
import android.media.MediaCodec
import android.media.MediaFormat
import android.os.Handler
import android.os.HandlerThread
import android.view.Surface
import android.view.SurfaceHolder
import androidx.compose.runtime.mutableStateOf
import java.util.concurrent.LinkedBlockingQueue
import uniffi.adamo_play.Connection
import uniffi.adamo_play.PlayException
import uniffi.adamo_play.VideoFormat
import uniffi.adamo_play.videoFormat

private const val RECEIVE_TIMEOUT_MS = 100u

class GameStream(private val connection: Connection, private val gamepads: Gamepads) : SurfaceHolder.Callback {
    val videoAspectRatio = mutableStateOf(16f / 9f)
    val error = mutableStateOf<String?>(null)
    @Volatile private var isRunning = true
    @Volatile private var videoSurface: Surface? = null
    private var videoThread: Thread? = null
    private val threads = listOf(start(::playAudio), start(::forwardRumble))

    init {
        gamepads.connection = connection
    }

    override fun surfaceCreated(holder: SurfaceHolder) {
        videoSurface = holder.surface
        videoThread = start { decodeVideo(holder.surface) }
    }

    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {}

    override fun surfaceDestroyed(holder: SurfaceHolder) {
        videoSurface = null
        videoThread?.join()
    }

    fun close() {
        isRunning = false
        videoSurface = null
        (threads + listOfNotNull(videoThread)).forEach(Thread::join)
        gamepads.connection = null
        connection.close()
    }

    private fun start(body: () -> Unit): Thread = Thread {
        try {
            body()
        } catch (exception: PlayException) {
            error.value = exception.message
        }
    }.apply { start() }

    private fun decodeVideo(surface: Surface) {
        val callbackThread = HandlerThread("video-output").apply { start() }
        val inputBufferIndices = LinkedBlockingQueue<Int>()
        var decoder: MediaCodec? = null
        var decoderSize: Pair<UInt, UInt>? = null
        while (isRunning && videoSurface == surface) {
            val frame = connection.receiveVideo(RECEIVE_TIMEOUT_MS) ?: continue
            val videoFormat = if (frame.isKeyframe) videoFormat(frame.data) else null
            val (mimeType, width, height) = when (videoFormat) {
                is VideoFormat.Av1 -> Triple(MediaFormat.MIMETYPE_VIDEO_AV1, videoFormat.width, videoFormat.height)
                is VideoFormat.H264 -> Triple(MediaFormat.MIMETYPE_VIDEO_AVC, videoFormat.width, videoFormat.height)
                null -> Triple("", 0u, 0u)
            }
            if (decoder != null && videoFormat != null && Pair(width, height) != decoderSize) {
                decoder.stop()
                decoder.release()
                decoder = null
                inputBufferIndices.clear()
            }
            if (decoder == null) {
                if (videoFormat == null) {
                    connection.requestKeyframe()
                    continue
                }
                decoderSize = Pair(width, height)
                videoAspectRatio.value = width.toFloat() / height.toFloat()
                val format = MediaFormat.createVideoFormat(mimeType, width.toInt(), height.toInt())
                format.setInteger(MediaFormat.KEY_LOW_LATENCY, 1)
                decoder = MediaCodec.createDecoderByType(mimeType)
                decoder.setCallback(RenderOnArrival(inputBufferIndices), Handler(callbackThread.looper))
                decoder.configure(format, surface, null, 0)
                decoder.start()
            }
            val index = inputBufferIndices.take()
            decoder.getInputBuffer(index)!!.put(frame.data)
            decoder.queueInputBuffer(index, 0, frame.data.size, System.nanoTime() / 1000, 0)
        }
        decoder?.stop()
        decoder?.release()
        callbackThread.quitSafely()
    }

    private fun playAudio() {
        var track: AudioTrack? = null
        while (isRunning) {
            val frame = connection.receiveAudio(RECEIVE_TIMEOUT_MS) ?: continue
            if (track == null) {
                track = createAudioTrack(frame.sampleRate.toInt(), frame.channelCount.toInt())
                track.play()
            }
            track.write(frame.pcmBytes, 0, frame.pcmBytes.size)
        }
        track?.release()
    }

    private fun forwardRumble() {
        while (isRunning) {
            connection.receiveRumble(RECEIVE_TIMEOUT_MS)?.let(gamepads::rumble)
        }
    }
}

private class RenderOnArrival(private val inputBufferIndices: LinkedBlockingQueue<Int>) : MediaCodec.Callback() {
    override fun onInputBufferAvailable(codec: MediaCodec, index: Int) {
        inputBufferIndices.put(index)
    }

    override fun onOutputBufferAvailable(codec: MediaCodec, index: Int, info: MediaCodec.BufferInfo) {
        codec.releaseOutputBuffer(index, true)
    }

    override fun onError(codec: MediaCodec, exception: MediaCodec.CodecException) {}

    override fun onOutputFormatChanged(codec: MediaCodec, format: MediaFormat) {}
}

private fun createAudioTrack(sampleRate: Int, channelCount: Int): AudioTrack {
    val channelMask = if (channelCount == 1) AudioFormat.CHANNEL_OUT_MONO else AudioFormat.CHANNEL_OUT_STEREO
    return AudioTrack.Builder()
        .setAudioAttributes(
            AudioAttributes.Builder()
                .setUsage(AudioAttributes.USAGE_GAME)
                .setContentType(AudioAttributes.CONTENT_TYPE_MUSIC)
                .build(),
        )
        .setAudioFormat(
            AudioFormat.Builder()
                .setSampleRate(sampleRate)
                .setChannelMask(channelMask)
                .setEncoding(AudioFormat.ENCODING_PCM_16BIT)
                .build(),
        )
        .setPerformanceMode(AudioTrack.PERFORMANCE_MODE_LOW_LATENCY)
        .setBufferSizeInBytes(AudioTrack.getMinBufferSize(sampleRate, channelMask, AudioFormat.ENCODING_PCM_16BIT))
        .build()
}
