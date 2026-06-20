use jni::JNIEnv;
use jni::objects::{JByteArray, JClass, JObject, JObjectArray, JValue};
use jni::sys::{jint, jlong, jsize};
use openh264::decoder::{DecodedYUV, Decoder, DecoderConfig, Flush};
use openh264::{nal_units, OpenH264API};
use openh264::formats::YUVSource;
use crate::openh264::exceptions::{jni_unwrap, throw_illegal_argument_exception, throw_runtime_exception};

#[no_mangle]
pub extern "C" fn Java_dev_nexbit_javah264_H264Decoder_createDecoder0(
    mut env: JNIEnv,
    _: JClass,
    flush_behavior: jint
) -> jlong {
    let config = DecoderConfig::new()
        .flush_after_decode(match flush_behavior {
            0 => Flush::Auto,
            1 => Flush::Flush,
            2 => Flush::NoFlush,
            _ => {
                throw_illegal_argument_exception(&mut env, format!("Invalid flush behaviour: {}", flush_behavior));
                return 0;
            }
        });
    let decoder = match Decoder::with_api_config(OpenH264API::from_source(), config) {
        Ok(d) => {d}
        Err(err) => {
            throw_illegal_argument_exception(&mut env, format!("Invalid decoder parameters: {}", err.to_string()));
            return 0;
        }
    };
    let raw = Box::into_raw(Box::new(decoder));
    raw as jlong
}

#[no_mangle]
pub extern "C" fn Java_dev_nexbit_javah264_H264Decoder_decodeRGBA0<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong,
    packet: JByteArray<'a>
) -> JObject<'a> {
    decode_and_construct(&mut env, ptr, packet, 4, |f, b| {
        f.write_rgba8(b)
    })
}

#[no_mangle]
pub extern "C" fn Java_dev_nexbit_javah264_H264Decoder_decodeRGB0<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong,
    packet: JByteArray<'a>
) -> JObject<'a> {
    decode_and_construct(&mut env, ptr, packet, 3, |f, b| {
        f.write_rgb8(b)
    })
}

fn decode_and_construct<'a>(
    env: &mut JNIEnv<'a>,
    ptr: jlong,
    packet: JByteArray<'a>,
    pixel_size: usize,
    write_fn: fn(&DecodedYUV, &mut [u8]),
) -> JObject<'a> {
    let decoder = unsafe { &mut *(ptr as *mut Decoder) };
    let bytes = match env.convert_byte_array(packet) {
        Ok(b) => b,
        Err(err) => {
            throw_runtime_exception(env, format!("Failed to convert java array: {}", err));
            return JObject::null();
        }
    };
    let decoded = match decoder.decode(&*bytes) {
        Ok(opt) => match opt {
            Some(d) => {d}
            None => {return JObject::null()}
        },
        Err(_) => {return JObject::null()},
    };
    let result_class = match env.find_class("dev/nexbit/javah264/DecodeResult") {
        Ok(c) => c,
        Err(err) => {
            throw_runtime_exception(env, format!("Failed find result class: {}", err));
            return JObject::null();
        }
    };
    match create_result(env, &result_class, &decoded, pixel_size, write_fn) {
        None => {JObject::null()}
        Some(o) => {o}
    }
}

#[no_mangle]
pub extern "C" fn Java_dev_nexbit_javah264_H264Decoder_flushRemainingRGBA0<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong
) -> JObjectArray<'a> {
    flush_remaining_and_construct(&mut env, ptr, 4, |f, b| {
        f.write_rgba8(b)
    })
}

#[no_mangle]
pub extern "C" fn Java_dev_nexbit_javah264_H264Decoder_flushRemainingRGB0<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong
) -> JObjectArray<'a> {
    flush_remaining_and_construct(&mut env, ptr, 3, |f, b| {
        f.write_rgb8(b)
    })
}

fn flush_remaining_and_construct<'a>(
    env: &mut JNIEnv<'a>,
    ptr: jlong,
    pixel_size: usize,
    write_fn: fn(&DecodedYUV, &mut [u8])
) -> JObjectArray<'a> {
    let decoder = unsafe { &mut *(ptr as *mut Decoder) };
    let result_class = match env.find_class("dev/nexbit/javah264/DecodeResult") {
        Ok(c) => c,
        Err(err) => {
            throw_runtime_exception(env, format!("Failed find result class: {}", err));
            return JObjectArray::default();
        }
    };
    match decoder.flush_remaining() {
        Ok(v) => {
            let return_array = jni_unwrap!(env, env.new_object_array(v.len() as jsize, &result_class, JObject::null()), JObjectArray::default(), "Failed to allocate result array");
            for (i, item) in v.iter().enumerate() {
                match create_result(env, &result_class, &item, pixel_size, write_fn) {
                    Some(o) => {
                        jni_unwrap!(env, env.set_object_array_element(&return_array, i as i32, o), JObjectArray::default(), "Failed to set result array element");
                    }
                    None => {return JObjectArray::default()}
                }
            }
            return_array
        }
        Err(_) => {
            jni_unwrap!(env, env.new_object_array(0, result_class, JObject::null()), JObjectArray::default(), "Failed to allocate empty result array")
        }
    }
}

fn create_result<'a>(
    env: &mut JNIEnv<'a>,
    result_class: &JClass<'a>,
    decoded: &DecodedYUV,
    pixel_size: usize,
    write_fn: fn(&DecodedYUV, &mut [u8])
) -> Option<JObject<'a>> {
    let (width, height) = decoded.dimensions();
    let mut buffer = vec![0u8; width * height * pixel_size];
    write_fn(&decoded, &mut buffer);
    let byte_array = match env.byte_array_from_slice(&buffer) {
        Ok(arr) => arr,
        Err(err) => {
            throw_runtime_exception(env, format!("Failed to convert to java array: {}", err));
            return None;
        }
    };
    match env.new_object(
        result_class,
        "(IIJ[B)V",
        &[
            JValue::from(width as i32),
            JValue::from(height as i32),
            JValue::from(decoded.timestamp().as_millis() as i64),
            JValue::Object(&JObject::from(byte_array)),
        ],
    ) {
        Ok(obj) => Some(obj),
        Err(err) => {
            throw_runtime_exception(env, format!("Failed to create return object: {}", err));
            None
        }
    }
}

#[no_mangle]
pub extern "C" fn Java_dev_nexbit_javah264_H264Decoder_destroyDecoder0(
    _: JNIEnv,
    _: JClass,
    ptr: jlong
) {
    if ptr != 0 {
        unsafe {
            drop(Box::from_raw(ptr as *mut Decoder));
        };
    }
}

#[no_mangle]
pub extern "C" fn Java_dev_nexbit_javah264_H264Decoder_nalUnits0<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
    bitstream: JByteArray<'a>
) -> JObjectArray<'a> {
    let bytes = match env.convert_byte_array(bitstream) {
        Ok(b) => {b}
        Err(err) => {
            throw_runtime_exception(&mut env, format!("Failed to convert java array: {}", err));
            return JObjectArray::default();
        }
    };
    let mut nal_unit_vec: Vec<JByteArray> = vec![];
    for nal_unit in nal_units(&*bytes) {
        match env.byte_array_from_slice(&nal_unit) {
            Ok(arr) => {
                nal_unit_vec.push(arr)
            }
            Err(err) => {
                throw_runtime_exception(&mut env, format!("Failed to convert to java array: {}", err));
                return JObjectArray::default();
            }
        }
    }
    let byte_array_class = jni_unwrap!(&mut env, env.find_class("[B"), JObjectArray::default(), "Couldn't find byte[] class");
    let return_array = jni_unwrap!(&mut env, env.new_object_array(nal_unit_vec.len() as jsize, byte_array_class, JObject::null()), JObjectArray::default(), "Failed to allocate NAL array");
    for (i, item) in nal_unit_vec.into_iter().enumerate() {
        jni_unwrap!(&mut env, env.set_object_array_element(&return_array, i as i32, item), JObjectArray::default(), "Failed to set NAL array element");
    };
    return_array
}
