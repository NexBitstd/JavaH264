package dev.nexbit.javah264;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import dev.nexbit.javah264.exception.EncoderException;
import dev.nexbit.javah264.exception.UnknownPlatformException;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.Objects;

import static org.junit.jupiter.api.Assertions.*;

public class JavaH264Test {

    @Test
    @DisplayName("Decode and encode")
    void decodeAndEncode() throws IOException, UnknownPlatformException, EncoderException {
        byte[] h264data;
        try (InputStream in = JavaH264Test.class.getClassLoader().getResourceAsStream("multi_512x512.h264")) {
            h264data = readAllBytes(Objects.requireNonNull(in));
        }
        boolean atLeastOneFrameDecoded = false;
        try (H264Decoder decoder = new H264Decoder(); H264Encoder encoder = new H264Encoder()) {
            for (byte[] nalUnit : H264Decoder.nalUnits(h264data)) {
                DecodeResult decodeResult = decoder.decodeRGBA(nalUnit);
                if (decodeResult != null) {
                    atLeastOneFrameDecoded = true;
                    encoder.encodeRGBA(decodeResult.getWidth(), decodeResult.getWidth(), decodeResult.getImage());
                }
            }
            atLeastOneFrameDecoded |= decoder.flushRemainingRGBA().length != 0;
        }
        assertTrue(atLeastOneFrameDecoded);
    }

    @Test
    @DisplayName("Decode and encodeSeparate")
    void decodeAndEncodeSeparate() throws IOException, UnknownPlatformException, EncoderException {
        byte[] h264data;
        try (InputStream in = JavaH264Test.class.getClassLoader().getResourceAsStream("multi_512x512.h264")) {
            h264data = readAllBytes(Objects.requireNonNull(in));
        }
        boolean atLeastOneFrameDecoded = false;
        try (H264Decoder decoder = new H264Decoder(); H264Encoder encoder = new H264Encoder()) {
            for (byte[] nalUnit : H264Decoder.nalUnits(h264data)) {
                DecodeResult decodeResult = decoder.decodeRGBA(nalUnit);
                if (decodeResult != null) {
                    atLeastOneFrameDecoded = true;
                    encoder.encodeSeparateRGBA(decodeResult.getWidth(), decodeResult.getWidth(), decodeResult.getImage());
                }
            }
            atLeastOneFrameDecoded |= decoder.flushRemainingRGBA().length != 0;
        }
        assertTrue(atLeastOneFrameDecoded);
    }

    private static byte[] readAllBytes(InputStream inputStream) throws IOException {
        final int bufLen = 1024;
        byte[] buf = new byte[bufLen];
        int readLen;
        IOException exception = null;

        try {
            ByteArrayOutputStream outputStream = new ByteArrayOutputStream();

            while ((readLen = inputStream.read(buf, 0, bufLen)) != -1)
                outputStream.write(buf, 0, readLen);

            return outputStream.toByteArray();
        } catch (IOException e) {
            exception = e;
            throw e;
        } finally {
            if (exception == null) inputStream.close();
            else try {
                inputStream.close();
            } catch (IOException e) {
                exception.addSuppressed(e);
            }
        }
    }

}
