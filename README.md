# JavaH264 (nexbit fork)

Java bindings for the [OpenH264 library](https://www.openh264.org/) written in Rust using JNI.

This is a fork of [DimasKama/JavaH264](https://github.com/DimasKama/JavaH264) (MIT), maintained for use in
[BitCam](https://github.com/Dalynkaa/BitCam). Original work © DimasKama; modifications © Zakhar Stupak.
See [LICENSE](LICENSE) (MIT + the bundled Cisco OpenH264 BSD notice).

## Changes from upstream

- Package renamed to `dev.nexbit.javah264` (JNI symbols updated accordingly).
- **Thread safety:** `H264Decoder`/`H264Encoder` guard the native pointer with a `ReadWriteLock`, so
  `close()` can no longer free the native instance while a decode/encode is in flight on another thread
  (this was a use-after-free that could crash the JVM).
- **Panic safety:** every `unwrap()`/`expect()` on the JNI boundary now throws a Java exception instead
  of panicking — under `panic = "abort"` a panic would take down the whole JVM.
- Published under `dev.nexbit:javah264` (see below).

## Usage

In your `build.gradle`:
```gradle
repositories {
    maven {
        name = "JavaH264 (GitHub Packages)"
        url = "https://maven.pkg.github.com/NexBitstd/JavaH264"
        credentials {
            username = System.getenv("GITHUB_ACTOR")
            password = System.getenv("GITHUB_TOKEN") // PAT with read:packages
        }
    }
}

dependencies {
    implementation "dev.nexbit:javah264:${javah264_version}"
}
```

Native libraries for all supported platforms are bundled inside the jar and extracted at runtime by
`LibraryLoader`.

## Building

The native part is built per-platform by GitHub Actions (`.github/workflows/`) and the resulting
`.so/.dylib/.dll` are dropped into `src/main/resources/natives/<platform>/` before the Gradle build
assembles the jar. To build a single native locally:
```bash
cd rust && cargo build --release --lib
```

## Credits
- [JavaH264](https://github.com/DimasKama/JavaH264) by DimasKama — the project this is forked from
- [OpenH264](https://www.openh264.org/)
- [openh264-rs](https://github.com/ralfbiedert/openh264-rs/)
- [jni-rs](https://github.com/jni-rs/jni-rs/)
