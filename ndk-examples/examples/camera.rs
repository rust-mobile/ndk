use ndk::{
    camera::{
        metadata::{ColorSpaceTransform, MeteringRectangle, Size, StreamConfiguration},
        tags::{FlashMode, LensFacing, MetadataTag},
        CameraCaptureFailure, CameraCaptureSession, CameraDevice, CameraDeviceError,
        CameraDeviceStateCallbacks, CameraId, CameraManager, CameraMetadata, CameraOutputTarget,
        CaptureCallbacks, CaptureRequest, CaptureSequenceId, CaptureSessionOutput,
        CaptureSessionOutputContainer, CaptureSessionStateCallbacks, RequestTemplate,
    },
    media::image_reader::{ImageFormat, ImageReader},
    native_window::NativeWindow,
};
use ndk_glue::native_window;
use std::{
    fs,
    io::Write,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
fn main() {
    inner_main().unwrap();
}

fn inner_main() -> anyhow::Result<()> {
    let manager = CameraManager::new();
    let config = get_camera_config(&manager)?;

    let state = Arc::new(Mutex::new(CameraState::new(config.stream)));
    let state_ref = CameraStateRef(state.clone());

    let device = manager.open_camera(&config.id, Box::new(state_ref.clone()))?;

    // Use winit's EventLoop for real apps
    while ndk_glue::poll_events() != Some(ndk_glue::Event::WindowCreated) {}
    let window = native_window().as_ref().cloned().unwrap();

    let container = CaptureSessionOutputContainer::new()?;
    let output = CaptureSessionOutput::new(window.clone())?;
    container.add(&output)?;

    let session = device.create_capture_session(&container, Box::new(state_ref.clone()))?;

    {
        let mut camera_state = state.lock().unwrap();

        let capture_request = device.create_capture_request(RequestTemplate::Preview)?;
        let window_target = CameraOutputTarget::new(window)?;
        capture_request.add_target(&window_target)?;
        session.set_repeating_request(Box::new(state_ref.clone()), &[capture_request])?;

        camera_state.device = Some(device);
        camera_state.session = Some(session);
    }

    // let's take a picture after 2 seconds
    thread::sleep(Duration::from_secs(2));

    {
        let mut camera_state = state.lock().unwrap();
        camera_state.take_picture(state_ref)?;
    }
    Ok(())
}

struct CameraConfig {
    stream: StreamConfiguration,
    id: CameraId,
}

fn get_camera_config(manager: &CameraManager) -> anyhow::Result<CameraConfig> {
    let mut selected_stream_config = StreamConfiguration {
        format: 0,
        width: 0,
        height: 0,
        is_input: 0,
    };
    let mut selected_camera: Option<CameraId> = None;

    for camera_id in manager.iter_cameras()? {
        let meta = manager.get_camera_characteristics(&camera_id)?;

        let facing: LensFacing = meta.get(MetadataTag::LENS_FACING)?;
        println!("camera {:?} facing {:?}", camera_id, facing);

        let stream_configs: &[StreamConfiguration] =
            meta.get(MetadataTag::SCALER_AVAILABLE_STREAM_CONFIGURATIONS)?;
        let mut selected = false;
        for stream_config in stream_configs
            .iter()
            .filter(|stream| stream.format() == Ok(ImageFormat::JPEG))
        {
            if stream_config.width > selected_stream_config.width {
                selected_stream_config = *stream_config;
                selected = true;
            }
        }
        if selected {
            selected_camera = Some(camera_id);
        }
        let csc: &ColorSpaceTransform = meta.get(MetadataTag::SENSOR_CALIBRATION_TRANSFORM1)?;
        println!("cst: {:?}", csc);
    }

    Ok(CameraConfig {
        stream: selected_stream_config,
        id: selected_camera.ok_or_else(|| anyhow::anyhow!("could not select camera"))?,
    })
}

struct CameraState {
    device: Option<CameraDevice>,
    session: Option<CameraCaptureSession>,
    queued_picture: bool,
    reader: ImageReader,
    size: Size,
}

impl CameraState {
    fn new(stream_config: StreamConfiguration) -> Self {
        let mut reader = ImageReader::new(
            stream_config.width,
            stream_config.height,
            stream_config.format().unwrap(),
            1,
        )
        .unwrap();

        reader
            .set_image_listener(Box::new(Self::image_available))
            .unwrap();

        Self {
            device: None,
            session: None,
            queued_picture: false,
            reader,
            size: stream_config.size(),
        }
    }

    fn image_available(reader: &ImageReader) {
        let image = reader.acquire_next_image().unwrap().unwrap();
        let data = image.get_plane_data(0).unwrap();

        let mut file = fs::File::create("/sdcard/DCIM/ndk-rs-image.jpeg").unwrap();
        file.write_all(data).unwrap();

        println!("image capture complete");
        // thread::sleep_ms(500);
        ndk_glue::native_activity().finish();
    }

    fn take_picture(&mut self, state: CameraStateRef) -> anyhow::Result<()> {
        let session = self.session.take().unwrap();
        session.stop_repeating()?;

        let reader_window = self.reader.get_window()?;
        let device = self.device.as_ref().unwrap();

        let capture_request = device.create_capture_request(RequestTemplate::StilCapture)?;
        let window_target = CameraOutputTarget::new(reader_window.clone())?;
        capture_request.add_target(&window_target)?;
        capture_request.set(MetadataTag::FLASH_MODE, &FlashMode::SINGLE)?;

        let quarter_width = self.size.width / 4;
        let quarter_height = self.size.height / 4;
        let region = MeteringRectangle {
            xmin: quarter_width,
            ymin: quarter_height,
            xmax: quarter_width * 3,
            ymax: quarter_height * 3,
            weight: 1,
        };
        capture_request.set(MetadataTag::CONTROL_AE_REGIONS, &[region][..])?;
        capture_request.set(MetadataTag::CONTROL_AF_REGIONS, &[region][..])?;

        let container = CaptureSessionOutputContainer::new()?;
        let output = CaptureSessionOutput::new(reader_window)?;
        container.add(&output)?;

        let state_ref = CameraStateRef(state.0.clone());
        let session = device.create_capture_session(&container, Box::new(state_ref))?;

        session.capture(Box::new(state), &[capture_request])?;
        self.session = Some(session);
        self.queued_picture = true;

        Ok(())
    }
}

#[derive(Clone)]
struct CameraStateRef(Arc<Mutex<CameraState>>);

impl CameraDeviceStateCallbacks for CameraStateRef {
    fn on_disconnected(&self, _device: &CameraDevice) {
        println!("camera disconnected!");
    }

    fn on_error(&self, _device: &CameraDevice, error: CameraDeviceError) {
        println!("camera error {:?}", error);
    }
}

impl CaptureSessionStateCallbacks for CameraStateRef {
    fn on_ready(&self, _session: &CameraCaptureSession) {
        println!("on_session_ready");
    }
    fn on_closed(&self, _session: &CameraCaptureSession) {
        println!("on_session_closed");
    }
    fn on_active(&self, _session: &CameraCaptureSession) {
        println!("on_session_active");
    }
}

impl CaptureCallbacks for CameraStateRef {
    fn on_capture_started(
        &self,
        _session: &CameraCaptureSession,
        _request: &CaptureRequest,
        _timestamp: i64,
    ) {
        if self.0.lock().unwrap().queued_picture {
            println!("on_capture_started");
        }
    }
    fn on_capture_progressed(
        &self,
        _session: &CameraCaptureSession,
        _request: &CaptureRequest,
        _result: &CameraMetadata,
    ) {
        if self.0.lock().unwrap().queued_picture {
            println!("on_capture_progressed");
        }
    }
    fn on_capture_completed(
        &self,
        _session: &CameraCaptureSession,
        _request: &CaptureRequest,
        _result: &CameraMetadata,
    ) {
        if self.0.lock().unwrap().queued_picture {
            println!("on_capture_completed");
        }
    }
    fn on_capture_failed(
        &self,
        _session: &CameraCaptureSession,
        _request: &CaptureRequest,
        _failure: CameraCaptureFailure,
    ) {
        println!("on_capture_failed");
    }
    fn on_capture_sequence_completed(
        &self,
        _session: &CameraCaptureSession,
        sequence_id: CaptureSequenceId,
        _frame_number: i64,
    ) {
        println!("on_capture_sequence_completed {}", sequence_id.0);
        let mut state = self.0.lock().unwrap();
        if state.queued_picture {
            // this should drop our state... right?
            state.device = None;
            state.session = None;
        }
    }
    fn on_capture_sequence_aborted(
        &self,
        _session: &CameraCaptureSession,
        _sequence_id: CaptureSequenceId,
    ) {
        println!("on_capture_sequence_aborted");
    }
    fn on_capture_buffer_lost(
        &self,
        _session: &CameraCaptureSession,
        _request: &CaptureRequest,
        _window: &NativeWindow,
        _frame_number: i64,
    ) {
        println!("on_capture_buffer_lost");
    }
}
