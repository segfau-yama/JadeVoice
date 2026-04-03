use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::voicevox_core as core;
use tokio::task;
use zip::ZipArchive;

pub type VoiceModelId = [u8; 16];

struct LoadedModel {
    id: VoiceModelId,
    model: *mut core::VoicevoxVoiceModelFile,
    path: String,
}

struct VoicevoxApiInner {
    open_jtalk: *mut core::OpenJtalkRc,
    synthesizer: *mut core::VoicevoxSynthesizer,
    loaded_models: Vec<LoadedModel>,
    style_to_model_path: HashMap<u32, String>,
}

// SAFETY: Access to the FFI pointers is serialized through Mutex in VoicevoxApi.
unsafe impl Send for VoicevoxApiInner {}
// SAFETY: Shared references never access inner state without first taking the Mutex.
unsafe impl Sync for VoicevoxApiInner {}

#[derive(Clone)]
pub struct VoicevoxApi {
    inner: Arc<Mutex<VoicevoxApiInner>>,
}

impl VoicevoxApi {
    pub fn new(dict_dir: &str, onnxruntime_path: &str) -> Result<Self, core::VoicevoxError> {
        let dict_dir =
            CString::new(dict_dir).map_err(|_| core::VoicevoxError::from_message("dict_dir contains NUL byte"))?;
        let onnxruntime_path = CString::new(onnxruntime_path)
            .map_err(|_| core::VoicevoxError::from_message("onnxruntime_path contains NUL byte"))?;

        // SAFETY: All pointers passed to the C API are valid NUL-terminated strings or out-pointers.
        unsafe {
            let mut ort_opts = core::voicevox_make_default_load_onnxruntime_options();
            ort_opts.filename = onnxruntime_path.as_ptr();
            let mut onnxruntime: *const core::VoicevoxOnnxruntime = std::ptr::null();
            core::check_result(core::voicevox_onnxruntime_load_once(ort_opts, &mut onnxruntime))?;

            let mut open_jtalk: *mut core::OpenJtalkRc = std::ptr::null_mut();
            if let Err(err) = core::check_result(core::voicevox_open_jtalk_rc_new(dict_dir.as_ptr(), &mut open_jtalk)) {
                return Err(err);
            }

            let mut synthesizer: *mut core::VoicevoxSynthesizer = std::ptr::null_mut();
            let init_opts = core::voicevox_make_default_initialize_options();
            if let Err(err) = core::check_result(core::voicevox_synthesizer_new(
                onnxruntime,
                open_jtalk,
                init_opts,
                &mut synthesizer,
            )) {
                core::voicevox_open_jtalk_rc_delete(open_jtalk);
                return Err(err);
            }

            Ok(Self {
                inner: Arc::new(Mutex::new(VoicevoxApiInner {
                open_jtalk,
                synthesizer,
                loaded_models: Vec::new(),
                style_to_model_path: HashMap::new(),
                })),
            })
        }
    }

    pub async fn load_model(&self, model_path: &str) -> Result<VoiceModelId, core::VoicevoxError> {
        let inner = Arc::clone(&self.inner);
        let model_path = model_path.to_owned();
        task::spawn_blocking(move || {
            let mut guard = inner
                .lock()
                .map_err(|_| core::VoicevoxError::from_message("voicevox api lock poisoned"))?;
            guard.load_model(&model_path)
        })
        .await
        .map_err(|e| core::VoicevoxError::from_message(format!("tts worker join error: {e}")))?
    }

    pub async fn register_models_from_dir(&self, model_dir: &str) -> Result<(), core::VoicevoxError> {
        let inner = Arc::clone(&self.inner);
        let model_dir = model_dir.to_owned();
        task::spawn_blocking(move || {
            let mut guard = inner
                .lock()
                .map_err(|_| core::VoicevoxError::from_message("voicevox api lock poisoned"))?;
            guard.register_models_from_dir(&model_dir)
        })
        .await
        .map_err(|e| core::VoicevoxError::from_message(format!("tts worker join error: {e}")))?
    }

    pub async fn unload_model(&self, model_id: VoiceModelId) -> Result<(), core::VoicevoxError> {
        let inner = Arc::clone(&self.inner);
        task::spawn_blocking(move || {
            let mut guard = inner
                .lock()
                .map_err(|_| core::VoicevoxError::from_message("voicevox api lock poisoned"))?;
            guard.unload_model(&model_id)
        })
        .await
        .map_err(|e| core::VoicevoxError::from_message(format!("tts worker join error: {e}")))?
    }

    #[allow(dead_code)]
    pub async fn is_model_loaded(&self, model_id: VoiceModelId) -> Result<bool, core::VoicevoxError> {
        let inner = Arc::clone(&self.inner);
        task::spawn_blocking(move || {
            let guard = inner
                .lock()
                .map_err(|_| core::VoicevoxError::from_message("voicevox api lock poisoned"))?;
            Ok(guard.is_model_loaded(&model_id))
        })
        .await
        .map_err(|e| core::VoicevoxError::from_message(format!("tts worker join error: {e}")))?
    }

    #[allow(dead_code)]
    pub async fn model_metas_json(&self, model_id: VoiceModelId) -> Result<String, core::VoicevoxError> {
        let inner = Arc::clone(&self.inner);
        task::spawn_blocking(move || {
            let guard = inner
                .lock()
                .map_err(|_| core::VoicevoxError::from_message("voicevox api lock poisoned"))?;
            guard.model_metas_json(&model_id)
        })
        .await
        .map_err(|e| core::VoicevoxError::from_message(format!("tts worker join error: {e}")))?
    }

    pub async fn tts(&self, text: &str, style_id: u32) -> Result<Vec<u8>, core::VoicevoxError> {
        let inner = Arc::clone(&self.inner);
        let text = text.to_owned();
        task::spawn_blocking(move || {
            let mut guard = inner
                .lock()
                .map_err(|_| core::VoicevoxError::from_message("voicevox api lock poisoned"))?;
            guard.tts(&text, style_id)
        })
        .await
        .map_err(|e| core::VoicevoxError::from_message(format!("tts worker join error: {e}")))?
    }
}

impl VoicevoxApiInner {
    fn load_model(&mut self, model_path: &str) -> Result<VoiceModelId, core::VoicevoxError> {
        if let Some(loaded) = self.loaded_models.iter().find(|loaded| loaded.path == model_path) {
            return Ok(loaded.id);
        }

        let model_path = CString::new(model_path)
            .map_err(|_| core::VoicevoxError::from_message("model_path contains NUL byte"))?;

        // SAFETY: self.synthesizer is valid while self is alive; out-pointers are valid for writes.
        unsafe {
            let mut model: *mut core::VoicevoxVoiceModelFile = std::ptr::null_mut();
            core::check_result(core::voicevox_voice_model_file_open(model_path.as_ptr(), &mut model))?;

            let mut model_id: VoiceModelId = [0; 16];
            core::voicevox_voice_model_file_id(model, &mut model_id);

            if let Err(err) = core::check_result(core::voicevox_synthesizer_load_voice_model(self.synthesizer, model)) {
                core::voicevox_voice_model_file_delete(model);
                return Err(err);
            }

            self.loaded_models.push(LoadedModel {
                id: model_id,
                model,
                path: model_path.to_string_lossy().into_owned(),
            });
            Ok(model_id)
        }
    }

    fn register_models_from_dir(&mut self, model_dir: &str) -> Result<(), core::VoicevoxError> {
        let entries = fs::read_dir(model_dir)
            .map_err(|e| core::VoicevoxError::from_message(format!("failed to read model_dir: {e}")))?;

        for entry in entries {
            let entry =
                entry.map_err(|e| core::VoicevoxError::from_message(format!("failed to read directory entry: {e}")))?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("vvm") {
                self.register_model_file(&path)?;
            }
        }
        Ok(())
    }

    fn register_model_file(&mut self, model_path: &Path) -> Result<(), core::VoicevoxError> {
        let file =
            File::open(model_path).map_err(|e| core::VoicevoxError::from_message(format!("failed to open model file: {e}")))?;
        let mut archive =
            ZipArchive::new(file).map_err(|e| core::VoicevoxError::from_message(format!("failed to read model zip: {e}")))?;
        let mut metas_file = archive
            .by_name("metas.json")
            .map_err(|e| core::VoicevoxError::from_message(format!("metas.json not found in model file: {e}")))?;

        let mut metas_json = String::new();
        metas_file
            .read_to_string(&mut metas_json)
            .map_err(|e| core::VoicevoxError::from_message(format!("failed to read metas.json: {e}")))?;

        let metas: serde_json::Value = serde_json::from_str(&metas_json)
            .map_err(|e| core::VoicevoxError::from_message(format!("failed to parse metas.json: {e}")))?;

        let path_string = model_path.to_string_lossy().into_owned();
        if let Some(speakers) = metas.as_array() {
            for speaker in speakers {
                if let Some(styles) = speaker.get("styles").and_then(|styles| styles.as_array()) {
                    for style in styles {
                        if let Some(style_id) = style.get("id").and_then(|id| id.as_u64()) {
                            self.style_to_model_path
                                .entry(style_id as u32)
                                .or_insert_with(|| path_string.clone());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn ensure_model_for_style(&mut self, style_id: u32) -> Result<(), core::VoicevoxError> {
        if self.style_to_model_path.get(&style_id).is_none() {
            let default_dir = PathBuf::from("/workspaces/vvtest/voicevox_core/models/vvms");
            if default_dir.exists() {
                self.register_models_from_dir(default_dir.to_string_lossy().as_ref())?;
            }
        }

        let model_path = self
            .style_to_model_path
            .get(&style_id)
            .cloned()
            .ok_or_else(|| core::VoicevoxError::from_message(format!("no model found for style_id={style_id}")))?;

        self.load_model(&model_path)?;
        Ok(())
    }

    fn unload_model(&mut self, model_id: &VoiceModelId) -> Result<(), core::VoicevoxError> {
        let index = self
            .loaded_models
            .iter()
            .position(|loaded| &loaded.id == model_id)
            .ok_or_else(|| core::VoicevoxError::from_message("model_id is not loaded"))?;

        // SAFETY: self.synthesizer is valid while self is alive; model_id points to 16-byte model ID.
        unsafe {
            core::check_result(core::voicevox_synthesizer_unload_voice_model(
                self.synthesizer,
                model_id as *const VoiceModelId,
            ))?;
        }

        let loaded = self.loaded_models.swap_remove(index);
        // SAFETY: loaded.model is an owned model pointer opened by this instance.
        unsafe {
            core::voicevox_voice_model_file_delete(loaded.model);
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn is_model_loaded(&self, model_id: &VoiceModelId) -> bool {
        // SAFETY: self.synthesizer is valid while self is alive; model_id points to 16-byte model ID.
        unsafe {
            core::voicevox_synthesizer_is_loaded_voice_model(self.synthesizer, model_id as *const VoiceModelId)
        }
    }

    #[allow(dead_code)]
    fn model_metas_json(&self, model_id: &VoiceModelId) -> Result<String, core::VoicevoxError> {
        let loaded = self
            .loaded_models
            .iter()
            .find(|loaded| &loaded.id == model_id)
            .ok_or_else(|| core::VoicevoxError::from_message("model_id is not loaded"))?;

        // SAFETY: loaded.model is valid while self is alive.
        unsafe {
            let metas_ptr = core::voicevox_voice_model_file_create_metas_json(loaded.model);
            if metas_ptr.is_null() {
                return Err(core::VoicevoxError::from_message("failed to get model metas"));
            }
            let metas = CStr::from_ptr(metas_ptr).to_string_lossy().into_owned();
            core::voicevox_json_free(metas_ptr);
            Ok(metas)
        }
    }

    fn tts(&mut self, text: &str, style_id: u32) -> Result<Vec<u8>, core::VoicevoxError> {
        self.ensure_model_for_style(style_id)?;
        let text = CString::new(text).map_err(|_| core::VoicevoxError::from_message("text contains NUL byte"))?;

        // SAFETY: self.synthesizer is valid while self is alive; out-pointers are valid for writes.
        unsafe {
            let opts = core::voicevox_make_default_tts_options();
            let mut wav_len: usize = 0;
            let mut wav_ptr: *mut u8 = std::ptr::null_mut();

            core::check_result(core::voicevox_synthesizer_tts(
                self.synthesizer,
                text.as_ptr(),
                style_id,
                opts,
                &mut wav_len,
                &mut wav_ptr,
            ))?;

            if wav_ptr.is_null() {
                return Err(core::VoicevoxError::from_message("TTS returned null WAV pointer"));
            }

            let wav = std::slice::from_raw_parts(wav_ptr, wav_len).to_vec();
            core::voicevox_wav_free(wav_ptr);
            Ok(wav)
        }
    }
}

impl Drop for VoicevoxApiInner {
    fn drop(&mut self) {
        // SAFETY: These pointers were obtained from matching constructors and are released once here.
        unsafe {
            for loaded in self.loaded_models.drain(..) {
                core::voicevox_voice_model_file_delete(loaded.model);
            }
        }

        // SAFETY: These pointers were obtained from matching constructors and are released once here.
        unsafe {
            core::voicevox_synthesizer_delete(self.synthesizer);
            core::voicevox_open_jtalk_rc_delete(self.open_jtalk);
        }
    }
}
