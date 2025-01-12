use crate::audios::RetroAudioCb;
use crate::sync::RetroSync;
use crate::video::RetroVideo;
use crate::{audios::RetroAudio, video::RetroVideoCb};
use generics::erro_handle::ErroHandle;
use retro_core::av_info::AvInfo;
use sdl2::{EventPump, Sdl};
use std::sync::Arc;

pub struct RetroAv {
    pub video: RetroVideo,
    pub audio: RetroAudio,
    sync: RetroSync,
    av_info: Option<Arc<AvInfo>>,
    _sdl: Sdl,
}

impl RetroAv {
    #[doc = "cria uma nova instancia de RetroAv. sempre mantenha a instancia dentro da thread onde foi criada!"]
    pub fn new() -> Result<Self, ErroHandle> {
        let _sdl = {
            match sdl2::init() {
                Ok(sdl) => sdl,
                Err(message) => return Err(ErroHandle { message }),
            }
        };

        let video = RetroVideo::new();
        let audio = RetroAudio::new()?;

        Ok(Self {
            video,
            audio,
            _sdl,
            sync: RetroSync::default(),
            av_info: None,
        })
    }

    pub fn build_window(&mut self, av_info: &Arc<AvInfo>) -> Result<EventPump, ErroHandle> {
        self.video.init(&self._sdl, av_info)?;
        self.av_info.replace(av_info.clone());

        let event_pump = match self._sdl.event_pump() {
            Ok(event_pump) => event_pump,
            Err(message) => return Err(ErroHandle { message }),
        };

        Ok(event_pump)
    }

    pub fn get_new_frame(&mut self) -> Result<(), ErroHandle> {
        if let Some(av_info) = &self.av_info {
            self.audio.resume_new_frame(av_info)?;
            self.video.draw_new_frame()?;
        }

        Ok(())
    }

    pub fn sync(&mut self) -> bool {
        if let Some(av_info) = &self.av_info {
            let fps = av_info.timing.fps.read().unwrap().abs();
            self.sync.sync(fps)
        } else {
            false
        }
    }

    pub fn get_core_cb(&self) -> (RetroVideoCb, RetroAudioCb) {
        let video_cb = self.video.get_core_cb();
        let audio_cb = self.audio.get_core_cb();

        (video_cb, audio_cb)
    }
}
