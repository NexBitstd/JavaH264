use jni::JNIEnv;

pub fn throw_runtime_exception<T: AsRef<str>>(env: &mut JNIEnv, message: T) {
    let _ = env.throw(("java/lang/RuntimeException", message));
}

pub fn throw_encoder_exception<T: AsRef<str>>(env: &mut JNIEnv, message: T) {
    let _ = env.throw(("dev/nexbit/javah264/exception/EncoderException", message));
}

pub fn throw_illegal_argument_exception<T: AsRef<str>>(env: &mut JNIEnv, message: T) {
    let _ = env.throw(("java/lang/IllegalArgumentException", message));
}

/// Unwraps a `Result`, or throws a Java `RuntimeException` and returns `$default` from the calling
/// JNI function. The crate is built with `panic = "abort"`, so a bare `unwrap()`/`expect()` on a
/// JNI error would abort the whole JVM; this surfaces it as a catchable Java exception instead.
macro_rules! jni_unwrap {
    ($env:expr, $result:expr, $default:expr, $context:expr $(,)?) => {
        match $result {
            Ok(value) => value,
            Err(err) => {
                $crate::openh264::exceptions::throw_runtime_exception($env, format!("{}: {}", $context, err));
                return $default;
            }
        }
    };
}

pub(crate) use jni_unwrap;
