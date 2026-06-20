package dev.nexbit.javah264;

import dev.nexbit.javah264.exception.UnknownPlatformException;

import java.io.IOException;

class OpenH264Lib {

    private static volatile boolean loaded;
    private static volatile Exception loadException;

    public static void load() throws UnknownPlatformException, IOException {
        if (!loaded) {
            synchronized (OpenH264Lib.class) {
                if (!loaded) {
                    try {
                        LibraryLoader.load("javah264");
                    } catch (Exception e) {
                        loadException = e;
                        throw e;
                    } finally {
                        loaded = true;
                    }
                }
            }
        }
        if (loadException != null) {
            if (loadException instanceof UnknownPlatformException) {
                throw (UnknownPlatformException) loadException;
            }
            if (loadException instanceof IOException) {
                throw (IOException) loadException;
            }
            throw new RuntimeException(loadException);
        }
    }

}
