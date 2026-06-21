//! Native camera enumeration and capture via `nokhwa` (AVFoundation / V4L2 / MediaFoundation),
//! exposed to Java as `dev.nexbit.javah264.NativeCamera` / `NativeCameraStream`.
//!
//! Devices are addressed by their **ordinal** in `nokhwa::query` order (not nokhwa's internal
//! `CameraIndex`, which may be a string), so Java only ever deals with `0..n`. An open stream lives
//! on a dedicated owner thread because `nokhwa::Camera` is bound to the thread that created it on
//! some backends, whereas BitCam opens on one thread and grabs frames on another.

use std::collections::HashSet;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread::{self, JoinHandle};

use image::imageops::FilterType;
use jni::objects::{JByteBuffer, JClass, JIntArray, JObject, JObjectArray};
use jni::sys::{jint, jlong, jsize};
use jni::JNIEnv;
use nokhwa::pixel_format::RgbAFormat;
use nokhwa::utils::{ApiBackend, RequestedFormat, RequestedFormatType, Resolution};
use nokhwa::{query, Camera};

use crate::openh264::exceptions::{throw_illegal_argument_exception, throw_runtime_exception};

/// Resolves the nokhwa `CameraIndex` for an ordinal position in query order.
fn camera_index_for(ordinal: jint) -> Result<nokhwa::utils::CameraIndex, String> {
    let cameras = query(ApiBackend::Auto).map_err(|e| format!("Camera query failed: {e}"))?;
    let info = cameras
        .get(ordinal as usize)
        .ok_or_else(|| format!("No camera at ordinal {ordinal}"))?;
    Ok(info.index().clone())
}

#[no_mangle]
pub extern "C" fn Java_dev_nexbit_javah264_NativeCamera_listDevices0<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
) -> JObjectArray<'a> {
    let cameras = match query(ApiBackend::Auto) {
        Ok(c) => c,
        Err(e) => {
            throw_runtime_exception(&mut env, format!("Failed to query cameras: {e}"));
            return JObjectArray::default();
        }
    };
    let string_class = match env.find_class("java/lang/String") {
        Ok(c) => c,
        Err(e) => {
            throw_runtime_exception(&mut env, format!("Couldn't find String class: {e}"));
            return JObjectArray::default();
        }
    };
    let array = match env.new_object_array(cameras.len() as jsize, &string_class, JObject::null()) {
        Ok(a) => a,
        Err(e) => {
            throw_runtime_exception(&mut env, format!("Failed to allocate device array: {e}"));
            return JObjectArray::default();
        }
    };
    for (i, info) in cameras.iter().enumerate() {
        let jstr = match env.new_string(info.human_name()) {
            Ok(s) => s,
            Err(e) => {
                throw_runtime_exception(&mut env, format!("Failed to build device name: {e}"));
                return JObjectArray::default();
            }
        };
        if let Err(e) = env.set_object_array_element(&array, i as i32, jstr) {
            throw_runtime_exception(&mut env, format!("Failed to set device element: {e}"));
            return JObjectArray::default();
        }
    }
    array
}

#[no_mangle]
pub extern "C" fn Java_dev_nexbit_javah264_NativeCamera_listFormats0<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
    ordinal: jint,
) -> JIntArray<'a> {
    let index = match camera_index_for(ordinal) {
        Ok(i) => i,
        Err(e) => {
            throw_runtime_exception(&mut env, e);
            return JIntArray::default();
        }
    };
    // compatible_camera_formats() reports the device's static capabilities; the requested format only
    // has to be valid for construction.
    let requested = RequestedFormat::new::<RgbAFormat>(RequestedFormatType::AbsoluteHighestResolution);
    let mut camera = match Camera::new(index, requested) {
        Ok(c) => c,
        Err(e) => {
            throw_runtime_exception(&mut env, format!("Failed to open camera for formats: {e}"));
            return JIntArray::default();
        }
    };
    let formats = match camera.compatible_camera_formats() {
        Ok(f) => f,
        Err(e) => {
            throw_runtime_exception(&mut env, format!("Failed to read camera formats: {e}"));
            return JIntArray::default();
        }
    };

    let mut seen: HashSet<(i32, i32, i32)> = HashSet::new();
    let mut flat: Vec<jint> = Vec::with_capacity(formats.len() * 3);
    for format in formats {
        let width = format.resolution().width() as jint;
        let height = format.resolution().height() as jint;
        let fps = format.frame_rate() as jint;
        if seen.insert((width, height, fps)) {
            flat.push(width);
            flat.push(height);
            flat.push(fps);
        }
    }

    let array = match env.new_int_array(flat.len() as jsize) {
        Ok(a) => a,
        Err(e) => {
            throw_runtime_exception(&mut env, format!("Failed to allocate format array: {e}"));
            return JIntArray::default();
        }
    };
    if let Err(e) = env.set_int_array_region(&array, 0, &flat) {
        throw_runtime_exception(&mut env, format!("Failed to fill format array: {e}"));
        return JIntArray::default();
    }
    array
}

/// A grab request: write a `target_w x target_h` RGBA frame into the direct buffer at `dst_ptr`.
struct GrabRequest {
    target_w: u32,
    target_h: u32,
    dst_ptr: usize,
    dst_len: usize,
    resp: SyncSender<Result<(u32, u32), String>>,
}

enum CamCommand {
    Grab(GrabRequest),
    Close,
}

/// Handle stored in the jlong: owns the camera thread's command channel.
struct CameraSession {
    tx: SyncSender<CamCommand>,
    join: Option<JoinHandle<()>>,
}

#[no_mangle]
pub extern "C" fn Java_dev_nexbit_javah264_NativeCameraStream_openCamera0(
    mut env: JNIEnv,
    _: JClass,
    ordinal: jint,
    width: jint,
    height: jint,
    fps: jint,
) -> jlong {
    let (cmd_tx, cmd_rx) = sync_channel::<CamCommand>(0);
    let (init_tx, init_rx) = sync_channel::<Result<(), String>>(0);
    let join = thread::spawn(move || camera_thread(ordinal, width, height, fps, cmd_rx, init_tx));

    match init_rx.recv() {
        Ok(Ok(())) => {
            let session = Box::new(CameraSession {
                tx: cmd_tx,
                join: Some(join),
            });
            Box::into_raw(session) as jlong
        }
        Ok(Err(message)) => {
            let _ = join.join();
            throw_runtime_exception(&mut env, message);
            0
        }
        Err(e) => {
            throw_runtime_exception(&mut env, format!("Camera thread failed to start: {e}"));
            0
        }
    }
}

fn camera_thread(
    ordinal: jint,
    width: jint,
    height: jint,
    fps: jint,
    rx: Receiver<CamCommand>,
    init_tx: SyncSender<Result<(), String>>,
) {
    let index = match camera_index_for(ordinal) {
        Ok(i) => i,
        Err(e) => {
            let _ = init_tx.send(Err(e));
            return;
        }
    };

    // Open by resolution (highest frame rate at that resolution) without pinning a FrameFormat — the
    // camera's native pixel format varies (YUYV on macOS, MJPEG/NV12 elsewhere) and a wrong hint makes
    // nokhwa reject the request. The actual capture cadence is driven by the caller, not the device.
    let _ = fps;
    let primary = if width > 0 && height > 0 {
        RequestedFormatType::HighestResolution(Resolution::new(width as u32, height as u32))
    } else {
        RequestedFormatType::AbsoluteHighestResolution
    };

    let mut camera = match Camera::new(index.clone(), RequestedFormat::new::<RgbAFormat>(primary)) {
        Ok(c) => c,
        Err(_) => {
            // The requested resolution isn't offered by this device (e.g. a fixed-format virtual
            // camera) — fall back to its highest resolution and let the grab-time resize hit the target.
            match Camera::new(index, RequestedFormat::new::<RgbAFormat>(RequestedFormatType::AbsoluteHighestResolution)) {
                Ok(c) => c,
                Err(e) => {
                    let _ = init_tx.send(Err(format!("Failed to open camera: {e}")));
                    return;
                }
            }
        }
    };
    if let Err(e) = camera.open_stream() {
        let _ = init_tx.send(Err(format!("Failed to start camera stream: {e}")));
        return;
    }
    let _ = init_tx.send(Ok(()));

    loop {
        match rx.recv() {
            Ok(CamCommand::Grab(request)) => {
                let result = grab_one(&mut camera, &request);
                let _ = request.resp.send(result);
            }
            Ok(CamCommand::Close) | Err(_) => break,
        }
    }
    let _ = camera.stop_stream();
}

fn grab_one(camera: &mut Camera, request: &GrabRequest) -> Result<(u32, u32), String> {
    let buffer = camera.frame().map_err(|e| format!("Failed to grab frame: {e}"))?;
    let decoded = buffer
        .decode_image::<RgbAFormat>()
        .map_err(|e| format!("Failed to decode frame: {e}"))?;
    let (src_w, src_h) = decoded.dimensions();

    let (pixels, out_w, out_h) =
        if request.target_w > 0 && request.target_h > 0 && (src_w != request.target_w || src_h != request.target_h) {
            let resized = image::imageops::resize(&decoded, request.target_w, request.target_h, FilterType::Triangle);
            (resized.into_raw(), request.target_w, request.target_h)
        } else {
            (decoded.into_raw(), src_w, src_h)
        };

    let needed = (out_w as usize) * (out_h as usize) * 4;
    if request.dst_len < needed {
        return Err(format!("Destination buffer too small: {} < {}", request.dst_len, needed));
    }
    // SAFETY: dst_ptr is a JNI direct-buffer address valid for dst_len bytes; the Java caller blocks
    // on `resp` until this write completes, so the buffer stays alive and unmoved.
    unsafe {
        std::ptr::copy_nonoverlapping(pixels.as_ptr(), request.dst_ptr as *mut u8, needed);
    }
    Ok((out_w, out_h))
}

#[no_mangle]
pub extern "C" fn Java_dev_nexbit_javah264_NativeCameraStream_grabFrameInto0(
    mut env: JNIEnv,
    _: JClass,
    handle: jlong,
    dst: JByteBuffer,
    target_w: jint,
    target_h: jint,
) -> jlong {
    if handle == 0 {
        throw_illegal_argument_exception(&mut env, "Camera session handle is null");
        return 0;
    }
    let session = unsafe { &*(handle as *const CameraSession) };

    let dst_ptr = match env.get_direct_buffer_address(&dst) {
        Ok(p) => p as usize,
        Err(e) => {
            throw_illegal_argument_exception(&mut env, format!("dst is not a direct ByteBuffer: {e}"));
            return 0;
        }
    };
    let dst_len = match env.get_direct_buffer_capacity(&dst) {
        Ok(c) => c,
        Err(e) => {
            throw_illegal_argument_exception(&mut env, format!("Failed to read dst capacity: {e}"));
            return 0;
        }
    };

    let (resp_tx, resp_rx) = sync_channel::<Result<(u32, u32), String>>(0);
    let request = GrabRequest {
        target_w: target_w.max(0) as u32,
        target_h: target_h.max(0) as u32,
        dst_ptr,
        dst_len,
        resp: resp_tx,
    };
    if session.tx.send(CamCommand::Grab(request)).is_err() {
        throw_runtime_exception(&mut env, "Camera thread is gone");
        return 0;
    }
    match resp_rx.recv() {
        Ok(Ok((w, h))) => ((w as jlong) << 32) | (h as jlong),
        Ok(Err(message)) => {
            throw_runtime_exception(&mut env, message);
            0
        }
        Err(e) => {
            throw_runtime_exception(&mut env, format!("Camera grab response failed: {e}"));
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn Java_dev_nexbit_javah264_NativeCameraStream_closeCamera0(
    _: JNIEnv,
    _: JClass,
    handle: jlong,
) {
    if handle != 0 {
        let mut session = unsafe { Box::from_raw(handle as *mut CameraSession) };
        let _ = session.tx.send(CamCommand::Close);
        if let Some(join) = session.join.take() {
            let _ = join.join();
        }
    }
}
