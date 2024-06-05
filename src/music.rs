use std::error::Error;

use soloud::{Backend, Handle, Soloud, SoloudError, SoloudFlag, Wav};

pub struct Music {
    pub soloud: Soloud,
    current_handle: Option<Handle>,
    bpm: f64,
    start_time: f64,
}
impl Music {
    pub fn new() -> Result<Self, SoloudError> {
        Ok(Music {
            soloud: Soloud::new(SoloudFlag::empty(), Backend::Auto, 44100, 1024, 2)?,
            current_handle: None,
            bpm: 0.0,
            start_time: 0.0,
        })
    }
    pub fn play(&mut self, source: &Wav, bpm: f64, start_time: f64) {
        if let Some(handle) = self.current_handle {
            self.soloud.stop(handle);
        }
        self.current_handle = Some(self.soloud.play(source));
        self.bpm = bpm;
        self.start_time = start_time;
    }
    /*pub fn play_offset(&mut self, source: &Wav, bpm: f64, start_time: f64, offset: f64) {
        if let Some(handle) = self.current_handle {
            self.soloud.stop(handle);
        }
        self.current_handle = Some(self.soloud.play(source));
        self.bpm = bpm;
        self.start_time = start_time - offset;
    }*/
    pub fn seek(&mut self, beats: f64) -> Result<(), Box<dyn Error>> {
        if let Some(handle) = self.current_handle {
            let seconds = beats / self.bpm * 60.0;
            self.soloud.seek(handle, seconds)?;
            self.start_time -= seconds;
        }
        Ok(())
    }
    pub fn finished(&self) -> bool {
        if let Some(handle) = self.current_handle {
            !self.soloud.is_valid_voice_handle(handle)
        } else {
            return true;
        }
    }
    pub fn beat(&self) -> f64 {
        if let Some(handle) = self.current_handle {
            let raw_time = self.soloud.stream_time(handle);
            (raw_time - self.start_time) * self.bpm / 60.0
        } else {
            0.0
        }
    }
}
