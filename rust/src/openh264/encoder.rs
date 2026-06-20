use jni::JNIEnv;
use jni::objects::{JByteArray, JClass, JObject, JObjectArray};
use jni::sys::{jboolean, jbyte, jfloat, jint, jlong, jshort, jsize};
use openh264::encoder::{BitRate, Complexity, Encoder, EncoderConfig, FrameRate, IntraFramePeriod, Level, Profile, QpRange, RateControlMode, SpsPpsStrategy, UsageType};
use openh264::formats::{RgbSliceU8, RgbaSliceU8, YUVBuffer};
use openh264::{OpenH264API};
use crate::openh264::exceptions::{jni_unwrap, throw_encoder_exception, throw_illegal_argument_exception, throw_runtime_exception};

#[no_mangle]
pub extern "C" fn Java_dev_nexbit_javah264_H264Encoder_createEncoder0(
    mut env: JNIEnv,
    _: JClass,
    enable_skip_frame: jboolean,
    target_bitrate: jint,
    max_frame_rate: jfloat,
    rate_control_mode: jint,
    sps_pps_strategy: jint,
    multiple_thread_idc: jshort,
    usage_type: jint,
    max_slice_len: jint,
    profile: jint,
    level: jint,
    complexity: jint,
    min_qp: jbyte,
    max_qp: jbyte,
    scene_change_detect: jboolean,
    adaptive_quantization: jboolean,
    background_detection: jboolean,
    long_term_reference: jboolean,
    intra_frame_period: jint
) -> jlong {
    let mut config = EncoderConfig::new()
        .skip_frames(enable_skip_frame != 0)
        .bitrate(BitRate::from_bps(target_bitrate as u32))
        .max_frame_rate(FrameRate::from_hz(max_frame_rate))
        .rate_control_mode(match rate_control_mode {
            0 => RateControlMode::Quality,
            1 => RateControlMode::Bitrate,
            2 => RateControlMode::Bufferbased,
            3 => RateControlMode::Timestamp,
            4 => RateControlMode::BitrateModePostSkip,
            5 => RateControlMode::Off,
            _ => {
                throw_illegal_argument_exception(&mut env, format!("Invalid rate control mode: {}", rate_control_mode));
                return 0;
            }
        })
        .sps_pps_strategy(match sps_pps_strategy {
            0 => SpsPpsStrategy::ConstantId,
            1 => SpsPpsStrategy::IncreasingId,
            2 => SpsPpsStrategy::SpsListing,
            3 => SpsPpsStrategy::SpsListingAndPpsIncreasing,
            4 => SpsPpsStrategy::SpsPpsListing,
            _ => {
                throw_illegal_argument_exception(&mut env, format!("Invalid sps pps strategy: {}", rate_control_mode));
                return 0;
            }
        })
        .num_threads(multiple_thread_idc as u16)
        .usage_type(match usage_type {
            0 => UsageType::CameraVideoRealTime,
            1 => UsageType::ScreenContentRealTime,
            2 => UsageType::CameraVideoNonRealTime,
            3 => UsageType::ScreenContentNonRealTime,
            4 => UsageType::InputContentTypeAll,
            _ => {
                throw_illegal_argument_exception(&mut env, format!("Invalid usage type: {}", rate_control_mode));
                return 0;
            }
        });
    if max_slice_len != -1 {
        config = config.max_slice_len(max_slice_len as u32)
    }
    if profile != -1 {
        config = config.profile(match profile {
            0 => Profile::Baseline,
            1 => Profile::Main,
            2 => Profile::Extended,
            3 => Profile::High,
            4 => Profile::High10,
            5 => Profile::High422,
            6 => Profile::High444,
            7 => Profile::CAVLC444,
            8 => Profile::ScalableBaseline,
            9 => Profile::ScalableHigh,
            _ => {
                throw_illegal_argument_exception(&mut env, format!("Invalid profile: {}", rate_control_mode));
                return 0;
            }
        });
    }
    if level != -1 {
        config = config.level(match level {
            0 => Level::Level_1_0,
            1 => Level::Level_1_B,
            2 => Level::Level_1_1,
            3 => Level::Level_1_2,
            4 => Level::Level_1_3,
            5 => Level::Level_2_0,
            6 => Level::Level_2_1,
            7 => Level::Level_2_2,
            8 => Level::Level_3_0,
            9 => Level::Level_3_1,
            10 => Level::Level_3_2,
            11 => Level::Level_4_0,
            12 => Level::Level_4_1,
            13 => Level::Level_4_2,
            14 => Level::Level_5_0,
            15 => Level::Level_5_1,
            16 => Level::Level_5_2,
            _ => {
                throw_illegal_argument_exception(&mut env, format!("Invalid level: {}", rate_control_mode));
                return 0;
            }
        });
    }
    config = config
        .complexity(match complexity {
            0 => Complexity::Low,
            1 => Complexity::Medium,
            2 => Complexity::High,
            _ => {
                throw_illegal_argument_exception(&mut env, format!("Invalid complexity: {}", rate_control_mode));
                return 0;
            }
        })
        .qp(QpRange::new(min_qp as u8, max_qp as u8))
        .scene_change_detect(scene_change_detect != 0)
        .adaptive_quantization(adaptive_quantization != 0)
        .background_detection(background_detection != 0)
        .long_term_reference(long_term_reference != 0)
        .intra_frame_period(IntraFramePeriod::from_num_frames(intra_frame_period as u32));

    let encoder = match Encoder::with_api_config(OpenH264API::from_source(), config) {
        Ok(e) => {e}
        Err(err) => {
            throw_illegal_argument_exception(&mut env, format!("Invalid encoder parameters: {}", err.to_string()));
            return 0;
        }
    };
    let raw = Box::into_raw(Box::new(encoder));
    raw as jlong
}

#[no_mangle]
pub extern "C" fn Java_dev_nexbit_javah264_H264Encoder_encodeRGBA0<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong,
    width: jint,
    height: jint,
    rgba: JByteArray<'a>
) -> JByteArray<'a> {
    encode_and_construct(&mut env, ptr, width, height, rgba, |data, dims| {
        YUVBuffer::from_rgb_source(RgbaSliceU8::new(data, dims))
    })
}

#[no_mangle]
pub extern "C" fn Java_dev_nexbit_javah264_H264Encoder_encodeRGB0<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong,
    width: jint,
    height: jint,
    rgb: JByteArray<'a>
) -> JByteArray<'a> {
    encode_and_construct(&mut env, ptr, width, height, rgb, |data, dims| {
        YUVBuffer::from_rgb_source(RgbSliceU8::new(data, dims))
    })
}

fn encode_and_construct<'a>(
    env: &mut JNIEnv<'a>,
    ptr: jlong,
    width: jint,
    height: jint,
    data: JByteArray<'a>,
    yuv_source_fn: fn(&[u8], (usize, usize)) -> YUVBuffer
) -> JByteArray<'a> {
    let encoder = unsafe { &mut *(ptr as *mut Encoder) };
    let bytes = match env.convert_byte_array(data) {
        Ok(b) => b,
        Err(err) => {
            throw_runtime_exception(env, format!("Failed to convert java array: {}", err));
            return JByteArray::default();
        }
    };
    let yuv_source = yuv_source_fn(&bytes, (width as usize, height as usize));
    match encoder.encode(&yuv_source) {
        Ok(bitstream) => {
            let vec = bitstream.to_vec();
            jni_unwrap!(env, env.byte_array_from_slice(&vec), JByteArray::default(), "Failed to convert encoded frame to java array")
        }
        Err(err) => {
            throw_encoder_exception(env, format!("Failed to encode: {}", err));
            JByteArray::default()
        }
    }
}

#[no_mangle]
pub extern "C" fn Java_dev_nexbit_javah264_H264Encoder_encodeSeparateRGBA0<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong,
    width: jint,
    height: jint,
    rgba: JByteArray<'a>
) -> JObjectArray<'a> {
    encode_and_construct_separate(&mut env, ptr, width, height, rgba, |data, dims| {
        YUVBuffer::from_rgb_source(RgbaSliceU8::new(data, dims))
    })
}

#[no_mangle]
pub extern "C" fn Java_dev_nexbit_javah264_H264Encoder_encodeSeparateRGB0<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
    ptr: jlong,
    width: jint,
    height: jint,
    rgb: JByteArray<'a>
) -> JObjectArray<'a> {
    encode_and_construct_separate(&mut env, ptr, width, height, rgb, |data, dims| {
        YUVBuffer::from_rgb_source(RgbSliceU8::new(data, dims))
    })
}

fn encode_and_construct_separate<'a>(
    env: &mut JNIEnv<'a>,
    ptr: jlong,
    width: jint,
    height: jint,
    data: JByteArray<'a>,
    yuv_source_fn: fn(&[u8], (usize, usize)) -> YUVBuffer
) -> JObjectArray<'a> {
    let encoder = unsafe { &mut *(ptr as *mut Encoder) };
    let bytes = match env.convert_byte_array(data) {
        Ok(b) => b,
        Err(err) => {
            throw_runtime_exception(env, format!("Failed to convert java array: {}", err));
            return JObjectArray::default();
        }
    };
    let yuv_source = yuv_source_fn(&bytes, (width as usize, height as usize));
    match encoder.encode(&yuv_source) {
        Ok(bitstream) => {
            let mut nal_unit_vec: Vec<JByteArray> = vec![];
            for l in 0..bitstream.num_layers() {
                let layer = match bitstream.layer(l) {
                    Some(layer) => layer,
                    None => continue,
                };
                for n in 0..layer.nal_count() {
                    let nal = match layer.nal_unit(n) {
                        Some(nal) => nal,
                        None => continue,
                    };
                    match env.byte_array_from_slice(&nal) {
                        Ok(arr) => {
                            nal_unit_vec.push(arr)
                        }
                        Err(err) => {
                            throw_runtime_exception(env, format!("Failed to convert to java array: {}", err));
                            return JObjectArray::default();
                        }
                    }
                }
            }
            let byte_array_class = jni_unwrap!(env, env.find_class("[B"), JObjectArray::default(), "Couldn't find byte[] class");
            let return_array = jni_unwrap!(env, env.new_object_array(nal_unit_vec.len() as jsize, byte_array_class, JObject::null()), JObjectArray::default(), "Failed to allocate NAL array");
            for (i, item) in nal_unit_vec.into_iter().enumerate() {
                jni_unwrap!(env, env.set_object_array_element(&return_array, i as i32, item), JObjectArray::default(), "Failed to set NAL array element");
            };
            return_array
        }
        Err(err) => {
            throw_encoder_exception(env, format!("Failed to encode: {}", err));
            JObjectArray::default()
        }
    }
}

#[no_mangle]
pub extern "C" fn Java_dev_nexbit_javah264_H264Encoder_destroyEncoder0(
    _: JNIEnv,
    _: JClass,
    ptr: jlong
) {
    if ptr != 0 {
        unsafe {
            drop(Box::from_raw(ptr as *mut Encoder));
        };
    }
}
