use std::error::Error;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::os::raw::{c_char, c_uint};

pub type VoicevoxResultCode = i32;
pub const VOICEVOX_RESULT_OK: VoicevoxResultCode = 0;
pub type VoicevoxVoiceModelId = *const [u8; 16];

#[repr(C)]
pub struct VoicevoxOnnxruntime(u8);
#[repr(C)]
pub struct OpenJtalkRc(u8);
#[repr(C)]
pub struct VoicevoxSynthesizer(u8);
#[repr(C)]
pub struct VoicevoxVoiceModelFile(u8);

#[repr(C)]
pub struct VoicevoxLoadOnnxruntimeOptions {
    pub filename: *const c_char,
}

#[repr(C)]
pub struct VoicevoxInitializeOptions {
    pub acceleration_mode: i32,
    pub cpu_num_threads: u16,
}

#[repr(C)]
pub struct VoicevoxTtsOptions {
    pub enable_interrogative_upspeak: bool,
}

#[derive(Debug, Clone)]
pub struct VoicevoxError {
    pub code: Option<VoicevoxResultCode>,
    pub message: String,
}

impl VoicevoxError {
    pub fn from_message(message: impl Into<String>) -> Self {
        Self {
            code: None,
            message: message.into(),
        }
    }

    pub fn from_code(code: VoicevoxResultCode) -> Self {
        Self {
            code: Some(code),
            message: error_message(code),
        }
    }
}

impl Display for VoicevoxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.code {
            Some(code) => write!(f, "VOICEVOX error {}: {}", code, self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl Error for VoicevoxError {}

pub fn check_result(code: VoicevoxResultCode) -> Result<(), VoicevoxError> {
    if code == VOICEVOX_RESULT_OK {
        Ok(())
    } else {
        Err(VoicevoxError::from_code(code))
    }
}

pub fn error_message(code: VoicevoxResultCode) -> String {
    // SAFETY: Returns a static NUL-terminated message pointer for a result code.
    unsafe {
        let ptr = voicevox_error_result_to_message(code);
        if ptr.is_null() {
            return "unknown VOICEVOX error".to_string();
        }
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

unsafe extern "C" {
    pub fn voicevox_make_default_load_onnxruntime_options() -> VoicevoxLoadOnnxruntimeOptions;
    pub fn voicevox_onnxruntime_load_once(
        options: VoicevoxLoadOnnxruntimeOptions,
        out_onnxruntime: *mut *const VoicevoxOnnxruntime,
    ) -> VoicevoxResultCode;
    pub fn voicevox_make_default_initialize_options() -> VoicevoxInitializeOptions;
    pub fn voicevox_open_jtalk_rc_new(
        open_jtalk_dic_dir: *const c_char,
        out_open_jtalk: *mut *mut OpenJtalkRc,
    ) -> VoicevoxResultCode;
    pub fn voicevox_synthesizer_new(
        onnxruntime: *const VoicevoxOnnxruntime,
        open_jtalk: *const OpenJtalkRc,
        options: VoicevoxInitializeOptions,
        out_synthesizer: *mut *mut VoicevoxSynthesizer,
    ) -> VoicevoxResultCode;
    pub fn voicevox_voice_model_file_open(
        path: *const c_char,
        out_model: *mut *mut VoicevoxVoiceModelFile,
    ) -> VoicevoxResultCode;
    pub fn voicevox_voice_model_file_id(
        model: *const VoicevoxVoiceModelFile,
        output_voice_model_id: *mut [u8; 16],
    );
    pub fn voicevox_synthesizer_load_voice_model(
        synthesizer: *const VoicevoxSynthesizer,
        model: *const VoicevoxVoiceModelFile,
    ) -> VoicevoxResultCode;
    pub fn voicevox_synthesizer_unload_voice_model(
        synthesizer: *const VoicevoxSynthesizer,
        model_id: VoicevoxVoiceModelId,
    ) -> VoicevoxResultCode;
    pub fn voicevox_synthesizer_is_loaded_voice_model(
        synthesizer: *const VoicevoxSynthesizer,
        model_id: VoicevoxVoiceModelId,
    ) -> bool;
    pub fn voicevox_make_default_tts_options() -> VoicevoxTtsOptions;
    pub fn voicevox_synthesizer_tts(
        synthesizer: *const VoicevoxSynthesizer,
        text: *const c_char,
        style_id: c_uint,
        options: VoicevoxTtsOptions,
        output_wav_length: *mut usize,
        output_wav: *mut *mut u8,
    ) -> VoicevoxResultCode;
    pub fn voicevox_wav_free(wav: *mut u8);
    pub fn voicevox_voice_model_file_delete(model: *mut VoicevoxVoiceModelFile);
    pub fn voicevox_synthesizer_delete(synthesizer: *mut VoicevoxSynthesizer);
    pub fn voicevox_open_jtalk_rc_delete(open_jtalk: *mut OpenJtalkRc);
    pub fn voicevox_error_result_to_message(result_code: VoicevoxResultCode) -> *const c_char;
    #[allow(dead_code)]
    pub fn voicevox_voice_model_file_create_metas_json(model: *const VoicevoxVoiceModelFile) -> *mut c_char;
    #[allow(dead_code)]
    pub fn voicevox_json_free(json: *mut c_char);
}
