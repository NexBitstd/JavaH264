package dev.nexbit.javah264;

import org.jetbrains.annotations.Nullable;
import dev.nexbit.javah264.exception.EncoderException;
import dev.nexbit.javah264.exception.UnknownPlatformException;

import java.io.IOException;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.locks.ReadWriteLock;
import java.util.concurrent.locks.ReentrantReadWriteLock;

public class H264Encoder implements AutoCloseable {

    private final AtomicBoolean closed = new AtomicBoolean();
    // Guards the native pointer: encode takes the read lock, close takes the write lock, so the native
    // encoder is never freed while an encode is in flight on another thread (use-after-free / JVM crash).
    private final ReadWriteLock lock = new ReentrantReadWriteLock();
    private final long pointer;

    public H264Encoder() throws IOException, UnknownPlatformException {
        this(new Builder());
    }

    public H264Encoder(Builder builder) throws IOException, UnknownPlatformException {
        OpenH264Lib.load();
        this.pointer = createEncoder0(
                builder.enableSkipFrame,
                builder.targetBitrate,
                builder.maxFrameRate,
                builder.rateControlMode.ordinal(),
                builder.spsPpsStrategy.ordinal(),
                builder.multipleThreadIdc,
                builder.usageType.ordinal(),
                builder.maxSliceLen != null ? builder.maxSliceLen : -1,
                builder.profile != null ? builder.profile.ordinal() : -1,
                builder.level != null ? builder.level.ordinal() : -1,
                builder.complexity.ordinal(),
                builder.minQp,
                builder.maxQp,
                builder.sceneChangeDetect,
                builder.adaptiveQuantization,
                builder.backgroundDetection,
                builder.longTermReference,
                builder.intraFramePeriod
        );
    }

    public static H264Encoder.Builder builder() {
        return new Builder();
    }

    public byte[] encodeRGBA(int width, int height, byte[] rgba) throws EncoderException {
        checkDims(width, height, 4, rgba.length);
        lock.readLock().lock();
        try {
            assertNotClosed();
            return encodeRGBA0(pointer, width, height, rgba);
        } finally {
            lock.readLock().unlock();
        }
    }

    public byte[] encodeRGB(int width, int height, byte[] rgb) throws EncoderException {
        checkDims(width, height, 3, rgb.length);
        lock.readLock().lock();
        try {
            assertNotClosed();
            return encodeRGB0(pointer, width, height, rgb);
        } finally {
            lock.readLock().unlock();
        }
    }

    public byte[][] encodeSeparateRGBA(int width, int height, byte[] rgba) throws EncoderException {
        checkDims(width, height, 4, rgba.length);
        lock.readLock().lock();
        try {
            assertNotClosed();
            return encodeSeparateRGBA0(pointer, width, height, rgba);
        } finally {
            lock.readLock().unlock();
        }
    }

    public byte[][] encodeSeparateRGB(int width, int height, byte[] rgb) throws EncoderException {
        checkDims(width, height, 3, rgb.length);
        lock.readLock().lock();
        try {
            assertNotClosed();
            return encodeSeparateRGB0(pointer, width, height, rgb);
        } finally {
            lock.readLock().unlock();
        }
    }

    private void checkDims(int width, int height, int pixelLen, int dataLength) {
        if (width < 16) {
            throw new IllegalArgumentException("Width cannot be < 16: " + width);
        }
        if (height < 16) {
            throw new IllegalArgumentException("Height cannot be < 16: " + height);
        }
        if (width * height * pixelLen != dataLength) {
            throw new IllegalArgumentException("width * height * " + pixelLen + " != image data length");
        }
        if ((width & 1) != 0) {
            throw new IllegalArgumentException("Width needs to be a multiple of 2");
        }
        if ((height & 1) != 0) {
            throw new IllegalArgumentException("Height needs to be a multiple of 2");
        }
    }

    private void assertNotClosed() {
        if (closed.get()) {
            throw new IllegalStateException("This H264Encoder instance is closed!");
        }
    }

    @Override
    public void close() {
        if (closed.compareAndSet(false, true)) {
            // Wait for any in-flight encode to finish before freeing the native encoder.
            lock.writeLock().lock();
            try {
                destroyEncoder0(pointer);
            } finally {
                lock.writeLock().unlock();
            }
        }
    }

    private static native long createEncoder0(
            boolean enableSkipFrame,
            int targetBitrate,
            float maxFrameRate,
            int rateControlMode,
            int spsPpsStrategy,
            short multipleThreadIdc,
            int usageType,
            int maxSliceLen,
            int profile,
            int level,
            int complexity,
            byte minQp,
            byte maxQp,
            boolean sceneChangeDetect,
            boolean adaptiveQuantization,
            boolean backgroundDetection,
            boolean longTermReference,
            int intraFramePeriod
    );

    private static native byte[] encodeRGBA0(long pointer, int width, int height, byte[] rgba) throws EncoderException;

    private static native byte[] encodeRGB0(long pointer, int width, int height, byte[] rgb) throws EncoderException;

    private static native byte[][] encodeSeparateRGBA0(long pointer, int width, int height, byte[] rgba) throws EncoderException;

    private static native byte[][] encodeSeparateRGB0(long pointer, int width, int height, byte[] rgb) throws EncoderException;

    private static native void destroyEncoder0(long pointer);

    public static class Builder {

        private boolean enableSkipFrame = true;
        private int targetBitrate = 120_000;
        private float maxFrameRate = 0.0F;
        private RateControlMode rateControlMode = RateControlMode.Quality;
        private SpsPpsStrategy spsPpsStrategy = SpsPpsStrategy.ConstantId;
        private short multipleThreadIdc = 0;
        private UsageType usageType = UsageType.CameraVideoRealTime;
        @Nullable
        private Integer maxSliceLen = null;
        @Nullable
        private Profile profile = null;
        @Nullable
        private Level level = null;
        private Complexity complexity = Complexity.Medium;
        private byte minQp = 0;
        private byte maxQp = 51;
        private boolean sceneChangeDetect = true;
        private boolean adaptiveQuantization = true;
        private boolean backgroundDetection = true;
        private boolean longTermReference = false;
        private int intraFramePeriod = 0;

        private Builder() {

        }

        public Builder enableSkipFrame(boolean value) {
            this.enableSkipFrame = value;
            return this;
        }

        public Builder targetBitrate(int value) {
            this.targetBitrate = value;
            return this;
        }

        public Builder maxFrameRate(float value) {
            this.maxFrameRate = value;
            return this;
        }

        public Builder rateControlMode(RateControlMode value) {
            this.rateControlMode = value;
            return this;
        }

        public Builder spsPpsStrategy(SpsPpsStrategy value) {
            this.spsPpsStrategy = value;
            return this;
        }

        public Builder multipleThreadIdc(short value) {
            this.multipleThreadIdc = value;
            return this;
        }

        public Builder usageType(UsageType value) {
            this.usageType = value;
            return this;
        }

        public Builder maxSliceLen(int value) {
            this.maxSliceLen = value;
            return this;
        }

        public Builder profile(Profile value) {
            this.profile = value;
            return this;
        }

        public Builder level(Level value) {
            this.level = value;
            return this;
        }

        public Builder complexity(Complexity value) {
            this.complexity = value;
            return this;
        }

        public Builder minQp(byte value) {
            this.minQp = value;
            return this;
        }

        public Builder maxQp(byte value) {
            this.maxQp = value;
            return this;
        }

        public Builder sceneChangeDetect(boolean value) {
            this.sceneChangeDetect = value;
            return this;
        }

        public Builder adaptiveQuantization(boolean value) {
            this.adaptiveQuantization = value;
            return this;
        }

        public Builder backgroundDetection(boolean value) {
            this.backgroundDetection = value;
            return this;
        }

        public Builder longTermReference(boolean value) {
            this.longTermReference = value;
            return this;
        }

        public Builder intraFramePeriod(int value) {
            this.intraFramePeriod = value;
            return this;
        }

        public H264Encoder build() throws IOException, UnknownPlatformException {
            return new H264Encoder(this);
        }

    }

    /// Specifies the mode used by the encoder to control the rate.
    public enum RateControlMode {

        Quality,
        Bitrate,
        Bufferbased,
        Timestamp,
        BitrateModePostSkip,
        Off

    }

    /// Sets the behavior for generating SPS/PPS.
    public enum SpsPpsStrategy {

        ConstantId,
        IncreasingId,
        SpsListing,
        SpsListingAndPpsIncreasing,
        SpsPpsListing

    }

    /// The intended usage scenario for the encoder.
    public enum UsageType {

        CameraVideoRealTime,
        ScreenContentRealTime,
        CameraVideoNonRealTime,
        ScreenContentNonRealTime,
        InputContentTypeAll

    }

    /// The H.264 encoding profile
    public enum Profile {

        Baseline,
        Main,
        Extended,
        High,
        High10,
        High422,
        High444,
        CAVLC444,
        ScalableBaseline,
        ScalableHigh

    }

    /// H.264 encoding levels with their corresponding capabilities.
    ///
    /// | Level   | Max Resolution (Pixels) | Max Frame Rate (fps) | Max Bitrate (Main Profile) | Max Bitrate (High Profile) |
    /// |---------|--------------------------|-----------------------|-----------------------------|-----------------------------|
    /// | 1.0     | 176x144 (QCIF)          | 15                   | 64 kbps                    | 80 kbps                    |
    /// | 1.1     | 176x144 (QCIF)          | 30                   | 192 kbps                   | 240 kbps                   |
    /// | 1.2     | 320x240 (QVGA)          | 30                   | 384 kbps                   | 480 kbps                   |
    /// | 2.0     | 352x288 (CIF)           | 30                   | 2 Mbps                     | 2.5 Mbps                   |
    /// | 3.0     | 720x576 (SD)            | 30                   | 10 Mbps                    | 12.5 Mbps                  |
    /// | 3.1     | 1280x720 (HD)           | 30                   | 14 Mbps                    | 17.5 Mbps                  |
    /// | 4.0     | 1920x1080 (Full HD)     | 30                   | 20 Mbps                    | 25 Mbps                    |
    /// | 4.1     | 1920x1080 (Full HD)     | 60                   | 50 Mbps                    | 62.5 Mbps                  |
    /// | 5.0     | 3840x2160 (4K)          | 30                   | 135 Mbps                   | 168.75 Mbps                |
    /// | 5.1     | 3840x2160 (4K)          | 60                   | 240 Mbps                   | 300 Mbps                   |
    /// | 5.2     | 4096x2160 (4K Cinema)   | 60                   | 480 Mbps                   | 600 Mbps                   |
    public enum Level {

        /// Level 1.0: Max resolution 176x144 (QCIF), 15 fps, 64 kbps (Main), 80 kbps (High)
        Level_1_0,
        /// Level 1.B: Specialized low-complexity baseline level.
        Level_1_B,
        /// Level 1.1: Max resolution 176x144 (QCIF), 30 fps, 192 kbps (Main), 240 kbps (High)
        Level_1_1,
        /// Level 1.2: Max resolution 320x240 (QVGA), 30 fps, 384 kbps (Main), 480 kbps (High)
        Level_1_2,
        /// Level 1.3: Reserved in standard, similar to Level 2.0.
        Level_1_3,
        /// Level 2.0: Max resolution 352x288 (CIF), 30 fps, 2 Mbps (Main), 2.5 Mbps (High)
        Level_2_0,
        /// Level 2.1: Max resolution 352x288 (CIF), 30 fps, 4 Mbps (Main), 5 Mbps (High)
        Level_2_1,
        /// Level 2.2: Max resolution 352x288 (CIF), 30 fps, 10 Mbps (Main), 12.5 Mbps (High)
        Level_2_2,
        /// Level 3.0: Max resolution 720x576 (SD), 30 fps, 10 Mbps (Main), 12.5 Mbps (High)
        Level_3_0,
        /// Level 3.1: Max resolution 1280x720 (HD), 30 fps, 14 Mbps (Main), 17.5 Mbps (High)
        Level_3_1,
        /// Level 3.2: Max resolution 1280x720 (HD), 60 fps, 20 Mbps (Main), 25 Mbps (High)
        Level_3_2,
        /// Level 4.0: Max resolution 1920x1080 (Full HD), 30 fps, 20 Mbps (Main), 25 Mbps (High)
        Level_4_0,
        /// Level 4.1: Max resolution 1920x1080 (Full HD), 60 fps, 50 Mbps (Main), 62.5 Mbps (High)
        Level_4_1,
        /// Level 4.2: Max resolution 1920x1080 (Full HD), 120 fps, 100 Mbps (Main), 125 Mbps (High)
        Level_4_2,
        /// Level 5.0: Max resolution 3840x2160 (4K), 30 fps, 135 Mbps (Main), 168.75 Mbps (High)
        Level_5_0,
        /// Level 5.1: Max resolution 3840x2160 (4K), 60 fps, 240 Mbps (Main), 300 Mbps (High)
        Level_5_1,
        /// Level 5.2: Max resolution 4096x2160 (4K Cinema), 60 fps, 480 Mbps (Main), 600 Mbps (High)
        Level_5_2

    }

    /// Complexity of the encoder (speed vs. quality).
    public enum Complexity {

        /// The lowest complexity, the fastest speed.
        Low,
        /// Medium complexity, medium speed, medium quality.
        Medium,
        /// High complexity, lowest speed, high quality.
        High

    }

}
