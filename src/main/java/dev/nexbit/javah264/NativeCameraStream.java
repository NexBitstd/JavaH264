package dev.nexbit.javah264;

import dev.nexbit.javah264.exception.UnknownPlatformException;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.locks.ReadWriteLock;
import java.util.concurrent.locks.ReentrantReadWriteLock;

/**
 * An open camera capture stream. Frames are grabbed into a caller-provided direct RGBA buffer. The
 * underlying nokhwa camera lives on its own native owner thread, so grabbing and closing are safe
 * from any Java thread.
 */
public final class NativeCameraStream implements AutoCloseable {

    private final AtomicBoolean closed = new AtomicBoolean();
    // Guards the native handle: grab takes the read lock, close takes the write lock, so the native
    // camera is never freed while a grab is in flight on another thread.
    private final ReadWriteLock lock = new ReentrantReadWriteLock();
    private final long handle;

    private int lastWidth;
    private int lastHeight;

    /**
     * Opens camera {@code ordinal} (index from {@link NativeCamera#listDevices()}). A non-positive
     * width/height/fps requests the device's highest resolution.
     */
    public NativeCameraStream(int ordinal, int width, int height, int fps) throws IOException, UnknownPlatformException {
        OpenH264Lib.load();
        this.handle = openCamera0(ordinal, width, height, fps);
        if (this.handle == 0L) {
            throw new IllegalStateException("Failed to open native camera at index " + ordinal);
        }
    }

    /**
     * Grabs the next frame, scaled to {@code targetWidth x targetHeight}, into {@code dst} as RGBA.
     * {@code dst} must be direct and hold at least {@code targetWidth * targetHeight * 4} bytes.
     * Returns {@code false} if no frame was produced.
     */
    public boolean grabInto(ByteBuffer dst, int targetWidth, int targetHeight) {
        if (dst == null || !dst.isDirect()) {
            throw new IllegalArgumentException("Destination buffer must be a direct ByteBuffer");
        }
        lock.readLock().lock();
        try {
            assertNotClosed();
            long packed = grabFrameInto0(handle, dst, targetWidth, targetHeight);
            if (packed == 0L) {
                return false;
            }
            this.lastWidth = (int) (packed >>> 32);
            this.lastHeight = (int) (packed & 0xFFFFFFFFL);
            return true;
        } finally {
            lock.readLock().unlock();
        }
    }

    /** Width of the most recent frame written by {@link #grabInto}. */
    public int lastWidth() {
        return lastWidth;
    }

    /** Height of the most recent frame written by {@link #grabInto}. */
    public int lastHeight() {
        return lastHeight;
    }

    public boolean isOpen() {
        return !closed.get();
    }

    private void assertNotClosed() {
        if (closed.get()) {
            throw new IllegalStateException("This NativeCameraStream is closed");
        }
    }

    @Override
    public void close() {
        if (closed.compareAndSet(false, true)) {
            // Wait for any in-flight grab to finish before freeing the native camera.
            lock.writeLock().lock();
            try {
                closeCamera0(handle);
            } finally {
                lock.writeLock().unlock();
            }
        }
    }

    private static native long openCamera0(int ordinal, int width, int height, int fps);

    private static native long grabFrameInto0(long handle, ByteBuffer dst, int targetWidth, int targetHeight);

    private static native void closeCamera0(long handle);
}
