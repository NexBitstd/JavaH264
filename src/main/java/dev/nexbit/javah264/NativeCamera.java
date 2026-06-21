package dev.nexbit.javah264;

import dev.nexbit.javah264.exception.UnknownPlatformException;

import java.io.IOException;
import java.util.ArrayList;
import java.util.List;

/**
 * Native camera enumeration backed by nokhwa (AVFoundation / V4L2 / MediaFoundation). Listing devices
 * does not open a capture stream; listing formats opens the device briefly to read its capabilities.
 */
public final class NativeCamera {

    private NativeCamera() {
    }

    /** Cameras in enumeration order; the list index matches {@link CameraDevice#index()}. */
    public static List<CameraDevice> listDevices() throws IOException, UnknownPlatformException {
        OpenH264Lib.load();
        String[] names = listDevices0();
        List<CameraDevice> devices = new ArrayList<>(names.length);
        for (int i = 0; i < names.length; i++) {
            devices.add(new CameraDevice(i, names[i]));
        }
        return devices;
    }

    /** Real capture modes (resolution + max fps) the camera at {@code ordinal} supports. */
    public static List<CameraVideoFormat> listFormats(int ordinal) throws IOException, UnknownPlatformException {
        OpenH264Lib.load();
        int[] flat = listFormats0(ordinal);
        List<CameraVideoFormat> formats = new ArrayList<>(flat.length / 3);
        for (int i = 0; i + 2 < flat.length; i += 3) {
            formats.add(new CameraVideoFormat(flat[i], flat[i + 1], flat[i + 2]));
        }
        return formats;
    }

    private static native String[] listDevices0();

    private static native int[] listFormats0(int ordinal);
}
