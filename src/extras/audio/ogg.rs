use super::super::{ErrorKind, Result};
use crate::audio::{AudioBuffer, AudioSample, GenericAudioSpecs};
use lewton::inside_ogg::OggStreamReader;
use std::io::{BufReader, Cursor};

pub struct OggFile {
    data: Option<Vec<u8>>,
    header: Option<lewton::header::IdentHeader>,
}

impl OggFile {
    pub fn new() -> Self {
        Self {
            data: None,
            header: None,
        }
    }
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    pub fn parse(mut self) -> Result<Self> {
        if let Some(data) = self.data.as_ref() {
            let reader = BufReader::new(Cursor::new(&data[..]));
            let ogg_reader = lewton::inside_ogg::OggStreamReader::new(reader)?;
            self.header = Some(ogg_reader.ident_hdr);
        }
        Ok(self)
    }
}

impl From<lewton::VorbisError> for ErrorKind {
    fn from(err: lewton::VorbisError) -> Self {
        Self::OggParseError(err.to_string())
    }
}

pub struct OggBuffer {
    ogg_data: Vec<u8>,
    header: lewton::header::IdentHeader,
    ogg_reader: Option<OggStreamReader<BufReader<Cursor<&'static [u8]>>>>,
    cur_samples: Vec<i16>,
    cur_index: usize,
}

impl OggBuffer {
    pub fn init_buffer(&mut self) -> Result<()> {
        let reader = OggStreamReader::new(BufReader::new(Cursor::new(unsafe {
            std::mem::transmute(&self.ogg_data[..])
        })))?;
        self.ogg_reader = Some(reader);
        Ok(())
    }
}

impl Drop for OggBuffer {
    fn drop(&mut self) {
        self.ogg_reader = None;
    }
}

impl Into<OggBuffer> for OggFile {
    fn into(self) -> OggBuffer {
        let mut buffer = OggBuffer {
            header: self.header.unwrap(),
            ogg_data: self.data.unwrap_or_default(),
            ogg_reader: None,
            cur_samples: Vec::new(),
            cur_index: 0,
        };
        buffer.init_buffer().expect("ogg parse failed");
        buffer
    }
}

impl GenericAudioSpecs for OggFile {
    fn sample_rate(&self) -> Option<u32> {
        self.header.as_ref().map(|hdr| hdr.audio_sample_rate)
    }
    fn channels(&self) -> Option<u32> {
        self.header.as_ref().map(|hdr| hdr.audio_channels as u32)
    }
    fn bits_per_sample(&self) -> Option<u32> {
        Some(16)
    }
}

impl AudioBuffer<f32> for OggBuffer {
    fn read(&mut self, out: &mut [AudioSample<f32>]) -> usize {
        //obviously we should check if the reader actually exists
        if self.ogg_reader.is_none() {
            return 0;
        }

        //pull in the next vector of samples
        if self.cur_index >= self.cur_samples.len() {
            if let Ok(Some(samples)) = self.ogg_reader.as_mut().unwrap().read_dec_packet_itl() {
                self.cur_index = 0;
                self.cur_samples = samples;
            }
        }
        const NORMALIZATION_FACTOR: f32 = 32767.0;
        let channels = self.header.audio_channels as usize;
        let mut out_index = 0;
        while out_index < out.len() * channels && self.cur_index < self.cur_samples.len() {
            out[out_index / channels].channel[1 - out_index % channels] =
                self.cur_samples[self.cur_index] as f32 / NORMALIZATION_FACTOR;
            out_index += 1;
            self.cur_index += 1;
            //if true we ran out of samples to push, so I pull in the next vector of samples.
            if self.cur_index >= self.cur_samples.len() {
                if let Ok(Some(samples)) = self.ogg_reader.as_mut().unwrap().read_dec_packet_itl() {
                    self.cur_index = 0;
                    self.cur_samples = samples;
                }
            }
        }
        out_index / channels
    }
    fn seek_to_start(&mut self) {
        //reinitalize buffer (looks like an ~O(1) operation , no copying happens here, its just a fancy pointer )
        let _ = self.init_buffer().map(|_| {
            self.cur_index = 0;
            self.cur_samples = Vec::new();
        });
    }
}

impl GenericAudioSpecs for OggBuffer {
    fn sample_rate(&self) -> Option<u32> {
        Some(self.header.audio_sample_rate)
    }
    fn channels(&self) -> Option<u32> {
        Some(self.header.audio_channels as u32)
    }
    fn bits_per_sample(&self) -> Option<u32> {
        Some(16)
    }
}
