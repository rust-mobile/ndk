#![allow(non_camel_case_types)]

use super::metadata::CameraEnum;
use num_enum::{IntoPrimitive, TryFromPrimitive};

/// A metadata tag is the key used for reading or writing camera parameters.
/// Use the associated constants for predefined tags.
#[repr(C)]
#[derive(Debug, Eq, Clone, Copy, PartialEq)]
pub struct MetadataTag(pub ffi::acamera_metadata_tag);

impl MetadataTag {
    pub const COLOR_CORRECTION_MODE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_COLOR_CORRECTION_MODE);
    pub const COLOR_CORRECTION_TRANSFORM: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_COLOR_CORRECTION_TRANSFORM);
    pub const COLOR_CORRECTION_GAINS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_COLOR_CORRECTION_GAINS);
    pub const COLOR_CORRECTION_ABERRATION_MODE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_COLOR_CORRECTION_ABERRATION_MODE);
    pub const COLOR_CORRECTION_AVAILABLE_ABERRATION_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_COLOR_CORRECTION_AVAILABLE_ABERRATION_MODES);

    pub const CONTROL_AE_ANTIBANDING_MODE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AE_ANTIBANDING_MODE);
    pub const CONTROL_AE_EXPOSURE_COMPENSATION: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AE_EXPOSURE_COMPENSATION);
    pub const CONTROL_AE_LOCK: Self = Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AE_LOCK);
    pub const CONTROL_AE_MODE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AE_MODE);
    pub const CONTROL_AE_REGIONS: Self = Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AE_REGIONS);
    pub const CONTROL_AE_TARGET_FPS_RANGE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AE_TARGET_FPS_RANGE);
    pub const CONTROL_AE_PRECAPTURE_TRIGGER: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AE_PRECAPTURE_TRIGGER);
    pub const CONTROL_AF_MODE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AF_MODE);
    pub const CONTROL_AF_REGIONS: Self = Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AF_REGIONS);
    pub const CONTROL_AF_TRIGGER: Self = Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AF_TRIGGER);
    pub const CONTROL_AWB_LOCK: Self = Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AWB_LOCK);
    pub const CONTROL_AWB_MODE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AWB_MODE);
    pub const CONTROL_AWB_REGIONS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AWB_REGIONS);
    pub const CONTROL_CAPTURE_INTENT: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_CAPTURE_INTENT);
    pub const CONTROL_EFFECT_MODE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_EFFECT_MODE);
    pub const CONTROL_MODE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_MODE);
    pub const CONTROL_SCENE_MODE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_SCENE_MODE);
    pub const CONTROL_VIDEO_STABILIZATION_MODE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_VIDEO_STABILIZATION_MODE);
    pub const CONTROL_AE_AVAILABLE_ANTIBANDING_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AE_AVAILABLE_ANTIBANDING_MODES);
    pub const CONTROL_AE_AVAILABLE_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AE_AVAILABLE_MODES);
    pub const CONTROL_AE_AVAILABLE_TARGET_FPS_RANGES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AE_AVAILABLE_TARGET_FPS_RANGES);
    pub const CONTROL_AE_COMPENSATION_RANGE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AE_COMPENSATION_RANGE);
    pub const CONTROL_AE_COMPENSATION_STEP: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AE_COMPENSATION_STEP);
    pub const CONTROL_AF_AVAILABLE_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AF_AVAILABLE_MODES);
    pub const CONTROL_AVAILABLE_EFFECTS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AVAILABLE_EFFECTS);
    pub const CONTROL_AVAILABLE_SCENE_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AVAILABLE_SCENE_MODES);
    pub const CONTROL_AVAILABLE_VIDEO_STABILIZATION_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AVAILABLE_VIDEO_STABILIZATION_MODES);
    pub const CONTROL_AWB_AVAILABLE_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AWB_AVAILABLE_MODES);
    pub const CONTROL_MAX_REGIONS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_MAX_REGIONS);
    pub const CONTROL_AE_STATE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AE_STATE);
    pub const CONTROL_AF_STATE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AF_STATE);
    pub const CONTROL_AWB_STATE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AWB_STATE);
    pub const CONTROL_AE_LOCK_AVAILABLE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AE_LOCK_AVAILABLE);
    pub const CONTROL_AWB_LOCK_AVAILABLE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AWB_LOCK_AVAILABLE);
    pub const CONTROL_AVAILABLE_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AVAILABLE_MODES);
    pub const CONTROL_POST_RAW_SENSITIVITY_BOOST_RANGE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_POST_RAW_SENSITIVITY_BOOST_RANGE);
    pub const CONTROL_POST_RAW_SENSITIVITY_BOOST: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_POST_RAW_SENSITIVITY_BOOST);
    pub const CONTROL_ENABLE_ZSL: Self = Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_ENABLE_ZSL);
    pub const CONTROL_AF_SCENE_CHANGE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AF_SCENE_CHANGE);
    pub const CONTROL_AVAILABLE_BOKEH_MAX_SIZES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AVAILABLE_BOKEH_MAX_SIZES);
    pub const CONTROL_AVAILABLE_BOKEH_ZOOM_RATIO_RANGES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_AVAILABLE_BOKEH_ZOOM_RATIO_RANGES);
    pub const CONTROL_BOKEH_MODE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_BOKEH_MODE);
    pub const CONTROL_ZOOM_RATIO_RANGE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_ZOOM_RATIO_RANGE);
    pub const CONTROL_ZOOM_RATIO: Self = Self(ffi::acamera_metadata_tag_ACAMERA_CONTROL_ZOOM_RATIO);

    pub const EDGE_MODE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_EDGE_MODE);
    pub const EDGE_AVAILABLE_EDGE_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_EDGE_AVAILABLE_EDGE_MODES);

    pub const FLASH_MODE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_FLASH_MODE);
    pub const FLASH_STATE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_FLASH_STATE);

    pub const FLASH_INFO_AVAILABLE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_FLASH_INFO_AVAILABLE);

    pub const HOT_PIXEL_MODE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_HOT_PIXEL_MODE);
    pub const HOT_PIXEL_AVAILABLE_HOT_PIXEL_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_HOT_PIXEL_AVAILABLE_HOT_PIXEL_MODES);

    pub const JPEG_GPS_COORDINATES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_JPEG_GPS_COORDINATES);
    pub const JPEG_GPS_PROCESSING_METHOD: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_JPEG_GPS_PROCESSING_METHOD);
    pub const JPEG_GPS_TIMESTAMP: Self = Self(ffi::acamera_metadata_tag_ACAMERA_JPEG_GPS_TIMESTAMP);
    pub const JPEG_ORIENTATION: Self = Self(ffi::acamera_metadata_tag_ACAMERA_JPEG_ORIENTATION);
    pub const JPEG_QUALITY: Self = Self(ffi::acamera_metadata_tag_ACAMERA_JPEG_QUALITY);
    pub const JPEG_THUMBNAIL_QUALITY: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_JPEG_THUMBNAIL_QUALITY);
    pub const JPEG_THUMBNAIL_SIZE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_JPEG_THUMBNAIL_SIZE);
    pub const JPEG_AVAILABLE_THUMBNAIL_SIZES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_JPEG_AVAILABLE_THUMBNAIL_SIZES);

    pub const LENS_APERTURE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_LENS_APERTURE);
    pub const LENS_FILTER_DENSITY: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_LENS_FILTER_DENSITY);
    pub const LENS_FOCAL_LENGTH: Self = Self(ffi::acamera_metadata_tag_ACAMERA_LENS_FOCAL_LENGTH);
    pub const LENS_FOCUS_DISTANCE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_LENS_FOCUS_DISTANCE);
    pub const LENS_OPTICAL_STABILIZATION_MODE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_LENS_OPTICAL_STABILIZATION_MODE);
    pub const LENS_FACING: Self = Self(ffi::acamera_metadata_tag_ACAMERA_LENS_FACING);
    pub const LENS_POSE_ROTATION: Self = Self(ffi::acamera_metadata_tag_ACAMERA_LENS_POSE_ROTATION);
    pub const LENS_POSE_TRANSLATION: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_LENS_POSE_TRANSLATION);
    pub const LENS_FOCUS_RANGE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_LENS_FOCUS_RANGE);
    pub const LENS_STATE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_LENS_STATE);
    pub const LENS_INTRINSIC_CALIBRATION: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_LENS_INTRINSIC_CALIBRATION);
    pub const LENS_RADIAL_DISTORTION: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_LENS_RADIAL_DISTORTION);
    pub const LENS_POSE_REFERENCE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_LENS_POSE_REFERENCE);
    pub const LENS_DISTORTION: Self = Self(ffi::acamera_metadata_tag_ACAMERA_LENS_DISTORTION);

    pub const LENS_INFO_AVAILABLE_APERTURES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_LENS_INFO_AVAILABLE_APERTURES);
    pub const LENS_INFO_AVAILABLE_FILTER_DENSITIES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_LENS_INFO_AVAILABLE_FILTER_DENSITIES);
    pub const LENS_INFO_AVAILABLE_FOCAL_LENGTHS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_LENS_INFO_AVAILABLE_FOCAL_LENGTHS);
    pub const LENS_INFO_AVAILABLE_OPTICAL_STABILIZATION: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_LENS_INFO_AVAILABLE_OPTICAL_STABILIZATION);
    pub const LENS_INFO_HYPERFOCAL_DISTANCE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_LENS_INFO_HYPERFOCAL_DISTANCE);
    pub const LENS_INFO_MINIMUM_FOCUS_DISTANCE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_LENS_INFO_MINIMUM_FOCUS_DISTANCE);
    pub const LENS_INFO_SHADING_MAP_SIZE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_LENS_INFO_SHADING_MAP_SIZE);
    pub const LENS_INFO_FOCUS_DISTANCE_CALIBRATION: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_LENS_INFO_FOCUS_DISTANCE_CALIBRATION);

    pub const NOISE_REDUCTION_MODE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_NOISE_REDUCTION_MODE);
    pub const NOISE_REDUCTION_AVAILABLE_NOISE_REDUCTION_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_NOISE_REDUCTION_AVAILABLE_NOISE_REDUCTION_MODES);

    pub const REQUEST_MAX_NUM_OUTPUT_STREAMS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_REQUEST_MAX_NUM_OUTPUT_STREAMS);
    pub const REQUEST_PIPELINE_DEPTH: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_REQUEST_PIPELINE_DEPTH);
    pub const REQUEST_PIPELINE_MAX_DEPTH: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_REQUEST_PIPELINE_MAX_DEPTH);
    pub const REQUEST_PARTIAL_RESULT_COUNT: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_REQUEST_PARTIAL_RESULT_COUNT);
    pub const REQUEST_AVAILABLE_CAPABILITIES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_REQUEST_AVAILABLE_CAPABILITIES);
    pub const REQUEST_AVAILABLE_REQUEST_KEYS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_REQUEST_AVAILABLE_REQUEST_KEYS);
    pub const REQUEST_AVAILABLE_RESULT_KEYS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_REQUEST_AVAILABLE_RESULT_KEYS);
    pub const REQUEST_AVAILABLE_CHARACTERISTICS_KEYS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_REQUEST_AVAILABLE_CHARACTERISTICS_KEYS);
    pub const REQUEST_AVAILABLE_SESSION_KEYS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_REQUEST_AVAILABLE_SESSION_KEYS);
    pub const REQUEST_AVAILABLE_PHYSICAL_CAMERA_REQUEST_KEYS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_REQUEST_AVAILABLE_PHYSICAL_CAMERA_REQUEST_KEYS);

    pub const SCALER_CROP_REGION: Self = Self(ffi::acamera_metadata_tag_ACAMERA_SCALER_CROP_REGION);
    pub const SCALER_AVAILABLE_MAX_DIGITAL_ZOOM: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SCALER_AVAILABLE_MAX_DIGITAL_ZOOM);
    pub const SCALER_AVAILABLE_STREAM_CONFIGURATIONS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SCALER_AVAILABLE_STREAM_CONFIGURATIONS);
    pub const SCALER_AVAILABLE_MIN_FRAME_DURATIONS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SCALER_AVAILABLE_MIN_FRAME_DURATIONS);
    pub const SCALER_AVAILABLE_STALL_DURATIONS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SCALER_AVAILABLE_STALL_DURATIONS);
    pub const SCALER_CROPPING_TYPE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SCALER_CROPPING_TYPE);
    pub const SCALER_AVAILABLE_RECOMMENDED_STREAM_CONFIGURATIONS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SCALER_AVAILABLE_RECOMMENDED_STREAM_CONFIGURATIONS);
    pub const SCALER_AVAILABLE_RECOMMENDED_INPUT_OUTPUT_FORMATS_MAP: Self = Self(
        ffi::acamera_metadata_tag_ACAMERA_SCALER_AVAILABLE_RECOMMENDED_INPUT_OUTPUT_FORMATS_MAP,
    );

    pub const SENSOR_EXPOSURE_TIME: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_EXPOSURE_TIME);
    pub const SENSOR_FRAME_DURATION: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_FRAME_DURATION);
    pub const SENSOR_SENSITIVITY: Self = Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_SENSITIVITY);
    pub const SENSOR_REFERENCE_ILLUMINANT1: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_REFERENCE_ILLUMINANT1);
    pub const SENSOR_REFERENCE_ILLUMINANT2: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_REFERENCE_ILLUMINANT2);
    pub const SENSOR_CALIBRATION_TRANSFORM1: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_CALIBRATION_TRANSFORM1);
    pub const SENSOR_CALIBRATION_TRANSFORM2: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_CALIBRATION_TRANSFORM2);
    pub const SENSOR_COLOR_TRANSFORM1: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_COLOR_TRANSFORM1);
    pub const SENSOR_COLOR_TRANSFORM2: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_COLOR_TRANSFORM2);
    pub const SENSOR_FORWARD_MATRIX1: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_FORWARD_MATRIX1);
    pub const SENSOR_FORWARD_MATRIX2: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_FORWARD_MATRIX2);
    pub const SENSOR_BLACK_LEVEL_PATTERN: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_BLACK_LEVEL_PATTERN);
    pub const SENSOR_MAX_ANALOG_SENSITIVITY: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_MAX_ANALOG_SENSITIVITY);
    pub const SENSOR_ORIENTATION: Self = Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_ORIENTATION);
    pub const SENSOR_TIMESTAMP: Self = Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_TIMESTAMP);
    pub const SENSOR_NEUTRAL_COLOR_POINT: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_NEUTRAL_COLOR_POINT);
    pub const SENSOR_NOISE_PROFILE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_NOISE_PROFILE);
    pub const SENSOR_GREEN_SPLIT: Self = Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_GREEN_SPLIT);
    pub const SENSOR_TEST_PATTERN_DATA: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_TEST_PATTERN_DATA);
    pub const SENSOR_TEST_PATTERN_MODE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_TEST_PATTERN_MODE);
    pub const SENSOR_AVAILABLE_TEST_PATTERN_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_AVAILABLE_TEST_PATTERN_MODES);
    pub const SENSOR_ROLLING_SHUTTER_SKEW: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_ROLLING_SHUTTER_SKEW);
    pub const SENSOR_OPTICAL_BLACK_REGIONS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_OPTICAL_BLACK_REGIONS);
    pub const SENSOR_DYNAMIC_BLACK_LEVEL: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_DYNAMIC_BLACK_LEVEL);
    pub const SENSOR_DYNAMIC_WHITE_LEVEL: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_DYNAMIC_WHITE_LEVEL);

    pub const SENSOR_INFO_ACTIVE_ARRAY_SIZE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_INFO_ACTIVE_ARRAY_SIZE);
    pub const SENSOR_INFO_SENSITIVITY_RANGE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_INFO_SENSITIVITY_RANGE);
    pub const SENSOR_INFO_COLOR_FILTER_ARRANGEMENT: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_INFO_COLOR_FILTER_ARRANGEMENT);
    pub const SENSOR_INFO_EXPOSURE_TIME_RANGE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_INFO_EXPOSURE_TIME_RANGE);
    pub const SENSOR_INFO_MAX_FRAME_DURATION: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_INFO_MAX_FRAME_DURATION);
    pub const SENSOR_INFO_PHYSICAL_SIZE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_INFO_PHYSICAL_SIZE);
    pub const SENSOR_INFO_PIXEL_ARRAY_SIZE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_INFO_PIXEL_ARRAY_SIZE);
    pub const SENSOR_INFO_WHITE_LEVEL: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_INFO_WHITE_LEVEL);
    pub const SENSOR_INFO_TIMESTAMP_SOURCE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_INFO_TIMESTAMP_SOURCE);
    pub const SENSOR_INFO_LENS_SHADING_APPLIED: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_INFO_LENS_SHADING_APPLIED);
    pub const SENSOR_INFO_PRE_CORRECTION_ACTIVE_ARRAY_SIZE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SENSOR_INFO_PRE_CORRECTION_ACTIVE_ARRAY_SIZE);

    pub const SHADING_MODE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_SHADING_MODE);
    pub const SHADING_AVAILABLE_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_SHADING_AVAILABLE_MODES);

    pub const STATISTICS_FACE_DETECT_MODE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_FACE_DETECT_MODE);
    pub const STATISTICS_HOT_PIXEL_MAP_MODE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_HOT_PIXEL_MAP_MODE);
    pub const STATISTICS_FACE_IDS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_FACE_IDS);
    pub const STATISTICS_FACE_LANDMARKS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_FACE_LANDMARKS);
    pub const STATISTICS_FACE_RECTANGLES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_FACE_RECTANGLES);
    pub const STATISTICS_FACE_SCORES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_FACE_SCORES);
    pub const STATISTICS_LENS_SHADING_MAP: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_LENS_SHADING_MAP);
    pub const STATISTICS_SCENE_FLICKER: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_SCENE_FLICKER);
    pub const STATISTICS_HOT_PIXEL_MAP: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_HOT_PIXEL_MAP);
    pub const STATISTICS_LENS_SHADING_MAP_MODE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_LENS_SHADING_MAP_MODE);
    pub const STATISTICS_OIS_DATA_MODE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_OIS_DATA_MODE);
    pub const STATISTICS_OIS_TIMESTAMPS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_OIS_TIMESTAMPS);
    pub const STATISTICS_OIS_X_SHIFTS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_OIS_X_SHIFTS);
    pub const STATISTICS_OIS_Y_SHIFTS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_OIS_Y_SHIFTS);

    pub const STATISTICS_INFO_AVAILABLE_FACE_DETECT_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_INFO_AVAILABLE_FACE_DETECT_MODES);
    pub const STATISTICS_INFO_MAX_FACE_COUNT: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_INFO_MAX_FACE_COUNT);
    pub const STATISTICS_INFO_AVAILABLE_HOT_PIXEL_MAP_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_INFO_AVAILABLE_HOT_PIXEL_MAP_MODES);
    pub const STATISTICS_INFO_AVAILABLE_LENS_SHADING_MAP_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_INFO_AVAILABLE_LENS_SHADING_MAP_MODES);
    pub const STATISTICS_INFO_AVAILABLE_OIS_DATA_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_STATISTICS_INFO_AVAILABLE_OIS_DATA_MODES);

    pub const TONEMAP_CURVE_BLUE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_TONEMAP_CURVE_BLUE);
    pub const TONEMAP_CURVE_GREEN: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_TONEMAP_CURVE_GREEN);
    pub const TONEMAP_CURVE_RED: Self = Self(ffi::acamera_metadata_tag_ACAMERA_TONEMAP_CURVE_RED);
    pub const TONEMAP_MODE: Self = Self(ffi::acamera_metadata_tag_ACAMERA_TONEMAP_MODE);
    pub const TONEMAP_MAX_CURVE_POINTS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_TONEMAP_MAX_CURVE_POINTS);
    pub const TONEMAP_AVAILABLE_TONE_MAP_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_TONEMAP_AVAILABLE_TONE_MAP_MODES);
    pub const TONEMAP_GAMMA: Self = Self(ffi::acamera_metadata_tag_ACAMERA_TONEMAP_GAMMA);
    pub const TONEMAP_PRESET_CURVE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_TONEMAP_PRESET_CURVE);

    pub const INFO_SUPPORTED_HARDWARE_LEVEL: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_INFO_SUPPORTED_HARDWARE_LEVEL);
    pub const INFO_VERSION: Self = Self(ffi::acamera_metadata_tag_ACAMERA_INFO_VERSION);

    pub const BLACK_LEVEL_LOCK: Self = Self(ffi::acamera_metadata_tag_ACAMERA_BLACK_LEVEL_LOCK);

    pub const SYNC_FRAME_NUMBER: Self = Self(ffi::acamera_metadata_tag_ACAMERA_SYNC_FRAME_NUMBER);
    pub const SYNC_MAX_LATENCY: Self = Self(ffi::acamera_metadata_tag_ACAMERA_SYNC_MAX_LATENCY);

    pub const DEPTH_AVAILABLE_DEPTH_STREAM_CONFIGURATIONS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_DEPTH_AVAILABLE_DEPTH_STREAM_CONFIGURATIONS);
    pub const DEPTH_AVAILABLE_DEPTH_MIN_FRAME_DURATIONS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_DEPTH_AVAILABLE_DEPTH_MIN_FRAME_DURATIONS);
    pub const DEPTH_AVAILABLE_DEPTH_STALL_DURATIONS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_DEPTH_AVAILABLE_DEPTH_STALL_DURATIONS);
    pub const DEPTH_DEPTH_IS_EXCLUSIVE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_DEPTH_DEPTH_IS_EXCLUSIVE);
    pub const DEPTH_AVAILABLE_RECOMMENDED_DEPTH_STREAM_CONFIGURATIONS: Self = Self(
        ffi::acamera_metadata_tag_ACAMERA_DEPTH_AVAILABLE_RECOMMENDED_DEPTH_STREAM_CONFIGURATIONS,
    );
    pub const DEPTH_AVAILABLE_DYNAMIC_DEPTH_STREAM_CONFIGURATIONS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_DEPTH_AVAILABLE_DYNAMIC_DEPTH_STREAM_CONFIGURATIONS);
    pub const DEPTH_AVAILABLE_DYNAMIC_DEPTH_MIN_FRAME_DURATIONS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_DEPTH_AVAILABLE_DYNAMIC_DEPTH_MIN_FRAME_DURATIONS);
    pub const DEPTH_AVAILABLE_DYNAMIC_DEPTH_STALL_DURATIONS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_DEPTH_AVAILABLE_DYNAMIC_DEPTH_STALL_DURATIONS);

    pub const LOGICAL_MULTI_CAMERA_PHYSICAL_IDS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_LOGICAL_MULTI_CAMERA_PHYSICAL_IDS);
    pub const LOGICAL_MULTI_CAMERA_SENSOR_SYNC_TYPE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_LOGICAL_MULTI_CAMERA_SENSOR_SYNC_TYPE);
    pub const LOGICAL_MULTI_CAMERA_ACTIVE_PHYSICAL_ID: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_LOGICAL_MULTI_CAMERA_ACTIVE_PHYSICAL_ID);

    pub const DISTORTION_CORRECTION_MODE: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_DISTORTION_CORRECTION_MODE);
    pub const DISTORTION_CORRECTION_AVAILABLE_MODES: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_DISTORTION_CORRECTION_AVAILABLE_MODES);

    pub const HEIC_AVAILABLE_HEIC_STREAM_CONFIGURATIONS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_HEIC_AVAILABLE_HEIC_STREAM_CONFIGURATIONS);
    pub const HEIC_AVAILABLE_HEIC_MIN_FRAME_DURATIONS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_HEIC_AVAILABLE_HEIC_MIN_FRAME_DURATIONS);
    pub const HEIC_AVAILABLE_HEIC_STALL_DURATIONS: Self =
        Self(ffi::acamera_metadata_tag_ACAMERA_HEIC_AVAILABLE_HEIC_STALL_DURATIONS);
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ColorCorrectionMode {
    TRANSFORM_MATRIX = 0,
    FAST = 1,
    HIGH_QUALITY = 2,
}

impl CameraEnum for ColorCorrectionMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ColorCorrectionAberrationMode {
    OFF = 0,
    FAST = 1,
    HIGH_QUALITY = 2,
}

impl CameraEnum for ColorCorrectionAberrationMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlAeAntibandingMode {
    OFF = 0,
    _50HZ = 1,
    _60HZ = 2,
    AUTO = 3,
}

impl CameraEnum for ControlAeAntibandingMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlAeLock {
    OFF = 0,
    ON = 1,
}

impl CameraEnum for ControlAeLock {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlAeMode {
    OFF = 0,
    ON = 1,
    ON_AUTO_FLASH = 2,
    ON_ALWAYS_FLASH = 3,
    ON_AUTO_FLASH_REDEYE = 4,
    ON_EXTERNAL_FLASH = 5,
}

impl CameraEnum for ControlAeMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlAePrecaptureTrigger {
    IDLE = 0,
    START = 1,
    CANCEL = 2,
}

impl CameraEnum for ControlAePrecaptureTrigger {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlAfMode {
    OFF = 0,
    AUTO = 1,
    MACRO = 2,
    CONTINUOUS_VIDEO = 3,
    CONTINUOUS_PICTURE = 4,
    EDOF = 5,
}

impl CameraEnum for ControlAfMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlAfTrigger {
    IDLE = 0,
    START = 1,
    CANCEL = 2,
}

impl CameraEnum for ControlAfTrigger {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlAwbLock {
    OFF = 0,
    ON = 1,
}

impl CameraEnum for ControlAwbLock {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlAwbMode {
    OFF = 0,
    AUTO = 1,
    INCANDESCENT = 2,
    FLUORESCENT = 3,
    WARM_FLUORESCENT = 4,
    DAYLIGHT = 5,
    CLOUDY_DAYLIGHT = 6,
    TWILIGHT = 7,
    SHADE = 8,
}

impl CameraEnum for ControlAwbMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlCaptureIntent {
    CUSTOM = 0,
    PREVIEW = 1,
    STILL_CAPTURE = 2,
    VIDEO_RECORD = 3,
    VIDEO_SNAPSHOT = 4,
    ZERO_SHUTTER_LAG = 5,
    MANUAL = 6,
    MOTION_TRACKING = 7,
}

impl CameraEnum for ControlCaptureIntent {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlEffectMode {
    OFF = 0,
    MONO = 1,
    NEGATIVE = 2,
    SOLARIZE = 3,
    SEPIA = 4,
    POSTERIZE = 5,
    WHITEBOARD = 6,
    BLACKBOARD = 7,
    AQUA = 8,
}

impl CameraEnum for ControlEffectMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlMode {
    OFF = 0,
    AUTO = 1,
    USE_SCENE_MODE = 2,
    OFF_KEEP_STATE = 3,
}

impl CameraEnum for ControlMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlSceneMode {
    DISABLED = 0,
    FACE_PRIORITY = 1,
    ACTION = 2,
    PORTRAIT = 3,
    LANDSCAPE = 4,
    NIGHT = 5,
    NIGHT_PORTRAIT = 6,
    THEATRE = 7,
    BEACH = 8,
    SNOW = 9,
    SUNSET = 10,
    STEADYPHOTO = 11,
    FIREWORKS = 12,
    SPORTS = 13,
    PARTY = 14,
    CANDLELIGHT = 15,
    BARCODE = 16,
    HDR = 18,
}

impl CameraEnum for ControlSceneMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlVideoStabilizationMode {
    OFF = 0,
    ON = 1,
}

impl CameraEnum for ControlVideoStabilizationMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlAeState {
    INACTIVE = 0,
    SEARCHING = 1,
    CONVERGED = 2,
    LOCKED = 3,
    FLASH_REQUIRED = 4,
    PRECAPTURE = 5,
}

impl CameraEnum for ControlAeState {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlAfState {
    INACTIVE = 0,
    PASSIVE_SCAN = 1,
    PASSIVE_FOCUSED = 2,
    ACTIVE_SCAN = 3,
    FOCUSED_LOCKED = 4,
    NOT_FOCUSED_LOCKED = 5,
    PASSIVE_UNFOCUSED = 6,
}

impl CameraEnum for ControlAfState {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlAwbState {
    INACTIVE = 0,
    SEARCHING = 1,
    CONVERGED = 2,
    LOCKED = 3,
}

impl CameraEnum for ControlAwbState {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlAeLockAvailable {
    FALSE = 0,
    TRUE = 1,
}

impl CameraEnum for ControlAeLockAvailable {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlAwbLockAvailable {
    FALSE = 0,
    TRUE = 1,
}

impl CameraEnum for ControlAwbLockAvailable {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlEnableZsl {
    FALSE = 0,
    TRUE = 1,
}

impl CameraEnum for ControlEnableZsl {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlAfSceneChange {
    NOT_DETECTED = 0,
    DETECTED = 1,
}
impl CameraEnum for ControlAfSceneChange {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ControlBokehMode {
    OFF = 0,
    STILL_CAPTURE = 1,
    CONTINUOUS = 2,
}

impl CameraEnum for ControlBokehMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum EdgeMode {
    OFF = 0,
    FAST = 1,
    HIGH_QUALITY = 2,
    ZERO_SHUTTER_LAG = 3,
}

impl CameraEnum for EdgeMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum FlashMode {
    OFF = 0,
    SINGLE = 1,
    TORCH = 2,
}

impl CameraEnum for FlashMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum FlashState {
    UNAVAILABLE = 0,
    CHARGING = 1,
    READY = 2,
    FIRED = 3,
    PARTIAL = 4,
}

impl CameraEnum for FlashState {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum FlashInfoAvailable {
    FALSE = 0,
    TRUE = 1,
}

impl CameraEnum for FlashInfoAvailable {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum HotPixelMode {
    OFF = 0,
    FAST = 1,
    HIGH_QUALITY = 2,
}

impl CameraEnum for HotPixelMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum LensOpticalStabilizationMode {
    OFF = 0,
    ON = 1,
}

impl CameraEnum for LensOpticalStabilizationMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum LensFacing {
    FRONT = 0,
    BACK = 1,
    EXTERNAL = 2,
}

impl CameraEnum for LensFacing {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum LensState {
    STATIONARY = 0,
    MOVING = 1,
}

impl CameraEnum for LensState {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum LensPoseReference {
    PRIMARY_CAMERA = 0,
    GYROSCOPE = 1,
    UNDEFINED = 2,
}
impl CameraEnum for LensPoseReference {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum LensInfoFocusDistanceCalibration {
    UNCALIBRATED = 0,
    APPROXIMATE = 1,
    CALIBRATED = 2,
}

impl CameraEnum for LensInfoFocusDistanceCalibration {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum NoiseReductionMode {
    OFF = 0,
    FAST = 1,
    HIGH_QUALITY = 2,
    MINIMAL = 3,
    ZERO_SHUTTER_LAG = 4,
}

impl CameraEnum for NoiseReductionMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RequestAvailableCapabilities {
    BACKWARD_COMPATIBLE = 0,
    MANUAL_SENSOR = 1,
    MANUAL_POST_PROCESSING = 2,
    RAW = 3,
    READ_SENSOR_SETTINGS = 5,
    BURST_CAPTURE = 6,
    DEPTH_OUTPUT = 8,
    MOTION_TRACKING = 10,
    LOGICAL_MULTI_CAMERA = 11,
    MONOCHROME = 12,
    SECURE_IMAGE_DATA = 13,
    SYSTEM_CAMERA = 14,
}
impl CameraEnum for RequestAvailableCapabilities {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ScalerAvailableStreamConfigurations {
    OUTPUT = 0,
    INPUT = 1,
}

impl CameraEnum for ScalerAvailableStreamConfigurations {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ScalerCroppingType {
    CENTER_ONLY = 0,
    FREEFORM = 1,
}
impl CameraEnum for ScalerCroppingType {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum ScalerAvailableRecommendedStreamConfigurations {
    PREVIEW = 0x0,
    RECORD = 0x1,
    VIDEO_SNAPSHOT = 0x2,
    SNAPSHOT = 0x3,
    ZSL = 0x4,
    RAW = 0x5,
    LOW_LATENCY_SNAPSHOT = 0x6,
    PUBLIC_END = 0x7,
    VENDOR_START = 0x18,
}

impl CameraEnum for ScalerAvailableRecommendedStreamConfigurations {
    type Entry = i32;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum SensorReferenceIlluminant1 {
    DAYLIGHT = 1,
    FLUORESCENT = 2,
    TUNGSTEN = 3,
    FLASH = 4,
    FINE_WEATHER = 9,
    CLOUDY_WEATHER = 10,
    SHADE = 11,
    DAYLIGHT_FLUORESCENT = 12,
    DAY_WHITE_FLUORESCENT = 13,
    COOL_WHITE_FLUORESCENT = 14,
    WHITE_FLUORESCENT = 15,
    STANDARD_A = 17,
    STANDARD_B = 18,
    STANDARD_C = 19,
    D55 = 20,
    D65 = 21,
    D75 = 22,
    D50 = 23,
    ISO_STUDIO_TUNGSTEN = 24,
}

impl CameraEnum for SensorReferenceIlluminant1 {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum SensorTestPatternMode {
    OFF = 0,
    SOLID_COLOR = 1,
    COLOR_BARS = 2,
    COLOR_BARS_FADE_TO_GRAY = 3,
    PN9 = 4,
    CUSTOM1 = 256,
}

impl CameraEnum for SensorTestPatternMode {
    type Entry = i32;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum SensorInfoColorFilterArrangement {
    RGGB = 0,
    GRBG = 1,
    GBRG = 2,
    BGGR = 3,
    RGB = 4,
    MONO = 5,
    NIR = 6,
}

impl CameraEnum for SensorInfoColorFilterArrangement {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum SensorInfoTimestampSource {
    UNKNOWN = 0,
    REALTIME = 1,
}

impl CameraEnum for SensorInfoTimestampSource {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum SensorInfoLensShadingApplied {
    FALSE = 0,
    TRUE = 1,
}

impl CameraEnum for SensorInfoLensShadingApplied {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ShadingMode {
    OFF = 0,
    FAST = 1,
    HIGH_QUALITY = 2,
}

impl CameraEnum for ShadingMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum StatisticsFaceDetectMode {
    OFF = 0,
    SIMPLE = 1,
    FULL = 2,
}

impl CameraEnum for StatisticsFaceDetectMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum StatisticsHotPixelMapMode {
    OFF = 0,
    ON = 1,
}

impl CameraEnum for StatisticsHotPixelMapMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum StatisticsSceneFlicker {
    NONE = 0,
    _50HZ = 1,
    _60HZ = 2,
}

impl CameraEnum for StatisticsSceneFlicker {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum StatisticsLensShadingMapMode {
    OFF = 0,
    ON = 1,
}

impl CameraEnum for StatisticsLensShadingMapMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum StatisticsOisDataMode {
    OFF = 0,
    ON = 1,
}

impl CameraEnum for StatisticsOisDataMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum TonemapMode {
    CONTRAST_CURVE = 0,
    FAST = 1,
    HIGH_QUALITY = 2,
    GAMMA_VALUE = 3,
    PRESET_CURVE = 4,
}
impl CameraEnum for TonemapMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum TonemapPresetCurve {
    SRGB = 0,
    REC709 = 1,
}

impl CameraEnum for TonemapPresetCurve {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum InfoSupportedHardwareLevel {
    LIMITED = 0,
    FULL = 1,
    LEGACY = 2,
    _3 = 3,
    EXTERNAL = 4,
}

impl CameraEnum for InfoSupportedHardwareLevel {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum BlackLevelLock {
    OFF = 0,
    ON = 1,
}

impl CameraEnum for BlackLevelLock {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(i64)]
pub enum SyncFrameNumber {
    CONVERGING = -1,
    UNKNOWN = -2,
}

impl CameraEnum for SyncFrameNumber {
    type Entry = i64;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum SyncMaxLatency {
    PER_FRAME_CONTROL = 0,
    UNKNOWN = -1,
}
impl CameraEnum for SyncMaxLatency {
    type Entry = i32;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum DepthAvailableDepthStreamConfigurations {
    OUTPUT = 0,
    INPUT = 1,
}

impl CameraEnum for DepthAvailableDepthStreamConfigurations {
    type Entry = i32;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum DepthDepthIsExclusive {
    FALSE = 0,
    TRUE = 1,
}

impl CameraEnum for DepthDepthIsExclusive {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum DepthAvailableDynamicDepthStreamConfigurations {
    OUTPUT = 0,
    INPUT = 1,
}

impl CameraEnum for DepthAvailableDynamicDepthStreamConfigurations {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum LogicalMultiCameraSensorSyncType {
    APPROXIMATE = 0,
    CALIBRATED = 1,
}

impl CameraEnum for LogicalMultiCameraSensorSyncType {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum DistortionCorrectionMode {
    OFF = 0,
    FAST = 1,
    HIGH_QUALITY = 2,
}

impl CameraEnum for DistortionCorrectionMode {
    type Entry = u8;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum HeicAvailableHeicStreamConfigurations {
    OUTPUT = ffi::acamera_metadata_enum_acamera_heic_available_heic_stream_configurations_ACAMERA_HEIC_AVAILABLE_HEIC_STREAM_CONFIGURATIONS_OUTPUT as _,
    INPUT = ffi::acamera_metadata_enum_acamera_heic_available_heic_stream_configurations_ACAMERA_HEIC_AVAILABLE_HEIC_STREAM_CONFIGURATIONS_INPUT as _,
}

impl CameraEnum for HeicAvailableHeicStreamConfigurations {
    type Entry = i32;
}
