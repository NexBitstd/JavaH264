package dev.nexbit.javah264;

public class DecodeResult {

    private final int width;
    private final int height;
    private final long timestamp;
    private final byte[] image;

    public DecodeResult(int width, int height, long timestamp, byte[] image) {
        this.width = width;
        this.height = height;
        this.timestamp = timestamp;
        this.image = image;
    }

    public int getWidth() {
        return width;
    }

    public int getHeight() {
        return height;
    }

    public byte[] getImage() {
        return image;
    }

    public long getTimestamp() {
        return timestamp;
    }

}
