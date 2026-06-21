package dev.nexbit.javah264;

import java.util.Objects;

/** A capture mode a camera supports: resolution plus the maximum frame rate for it. */
public final class CameraVideoFormat {

    private final int width;
    private final int height;
    private final int fps;

    public CameraVideoFormat(int width, int height, int fps) {
        this.width = width;
        this.height = height;
        this.fps = fps;
    }

    public int width() {
        return width;
    }

    public int height() {
        return height;
    }

    public int fps() {
        return fps;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) {
            return true;
        }
        if (!(o instanceof CameraVideoFormat)) {
            return false;
        }
        CameraVideoFormat other = (CameraVideoFormat) o;
        return width == other.width && height == other.height && fps == other.fps;
    }

    @Override
    public int hashCode() {
        return Objects.hash(width, height, fps);
    }

    @Override
    public String toString() {
        return width + "x" + height + "@" + fps;
    }
}
