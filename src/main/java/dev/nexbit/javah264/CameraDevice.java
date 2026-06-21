package dev.nexbit.javah264;

/**
 * A camera device reported by {@link NativeCamera#listDevices()}. The {@code index} is the ordinal in
 * enumeration order and is what {@link NativeCameraStream} and {@link NativeCamera#listFormats(int)}
 * expect.
 */
public final class CameraDevice {

    private final int index;
    private final String name;

    public CameraDevice(int index, String name) {
        this.index = index;
        this.name = name;
    }

    public int index() {
        return index;
    }

    public String name() {
        return name;
    }

    @Override
    public String toString() {
        return index + ":" + name;
    }
}
