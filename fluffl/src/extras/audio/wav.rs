use super::super::{ErrorKind, Result};
use super::{AudioBuffer, AudioSample, PcmConverter};
use crate::audio::GenericAudioSpecs;

use std::io::Read;
use std::mem;
use std::slice;

macro_rules! convert_and_copy {
    ( $src_type:ident, $dst_type:ident, $data_slice:ident , $convert_closure:expr ) => {{
        return unsafe {
            slice::from_raw_parts(
                $data_slice.as_ptr() as *const AudioSample<$src_type>,
                $data_slice.len() / mem::size_of::<AudioSample<$src_type>>(),
            )
        }
        .iter()
        .map(|&src_sample| {
            AudioSample::from([
                $convert_closure(src_sample.channel[0]),
                $convert_closure(src_sample.channel[1]),
            ])
        })
        .collect();
    }};
}

// the wav header according to: http://soundfile.sapp.org/doc/WaveFormat/
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct WavHeader {
    pub chunk_id: u32,
    pub chunk_size: u32,
    pub format: u32,
    pub subchunk1_id: u32,
    pub subchunk1_id_size: u32,
    pub audio_format: u16,
    pub num_channels: u16,
    pub sample_rate: u32,
    pub byte_rate: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
    pub subchunk2_id: u32,
    pub subchunk2_size: u32,
}

#[derive(Default)]
pub struct WavFile {
    data: Option<Vec<u8>>,
    header: Option<WavHeader>,
}

impl WavFile {
    pub fn new() -> Self {
        Self {
            data: None,
            header: None,
        }
    }
    pub fn header(&self) -> Option<WavHeader> {
        self.header
    }

    pub fn with_data(mut self, wav_data: Vec<u8>) -> Self {
        self.data = Some(wav_data);
        self
    }

    pub fn data(&self) -> Option<&Vec<u8>> {
        self.data.as_ref()
    }

    pub fn data_mut(&mut self) -> Option<&mut Vec<u8>> {
        self.data.as_mut()
    }
    pub fn parse(mut self) -> Result<Self> {
        let res = self.data.as_ref().map(|data| {
            let mut data_slice = &data[..];
            let mut header_bytes = [0u8; 44];
            data_slice.read(&mut header_bytes).map(|bytes_read| {
                let header = unsafe { *header_bytes.as_ptr().cast::<WavHeader>() };
                (header, bytes_read)
            })
        });

        self.header = match res {
            Some(Ok((header, 44))) => Some(header),
            _ => {
                return Err(wav_parse_error(
                    "File not large enough! Failed to read wav header!",
                ));
            }
        };
        if let Some(WavHeader {
            bits_per_sample,
            audio_format,
            ..
        }) = self.header
        {
            if bits_per_sample != 16 && bits_per_sample != 8 {
                return Err(wav_parse_error(
                    "Only 8 and 16 bit precision pcm is supported",
                ));
            }
            if audio_format != 1 {
                return Err(wav_parse_error("Only uncompressed wav files is supported"));
            }
        }

        Ok(self)
    }
}

impl GenericAudioSpecs for WavFile {
    fn sample_rate(&self) -> Option<u32> {
        self.header.map(|hdr| hdr.sample_rate)
    }

    fn bits_per_sample(&self) -> Option<u32> {
        self.header.map(|hdr| hdr.bits_per_sample as u32)
    }
    fn channels(&self) -> Option<u32> {
        self.header.map(|hdr| hdr.num_channels as u32)
    }
}

impl PcmConverter<f32> for WavFile {
    #[allow(unreachable_code)]
    fn samples(self) -> Vec<AudioSample<f32>> {
        if let Some(&header) = self.header.as_ref() {
            if let Some(data_vec) = &self.data {
                let data_slice = &data_vec[44..];
                if header.bits_per_sample == 8 {
                    let convert_closure = |input| input as f32 / 255.0f32;
                    return convert_and_copy!(u8, f32, data_slice, convert_closure);
                } else if header.bits_per_sample == 16 {
                    let convert_closure = |input| input as f32 / 32767.0;
                    return convert_and_copy!(i16, f32, data_slice, convert_closure);
                }
            }
        }
        Vec::new()
    }
}

impl<T> From<Vec<AudioSample<T>>> for WavBuffer<T> {
    fn from(samples: Vec<AudioSample<T>>) -> Self {
        Self {
            samples,
            sample_index: 0,
        }
    }
}

impl From<WavFile> for WavBuffer<f32> {
    fn from(wf: WavFile) -> Self {
        Self {
            samples: wf.samples(),
            sample_index: 0,
        }
    }
}

fn wav_parse_error(msg: &str) -> ErrorKind {
    ErrorKind::WavParseError(String::from(msg))
}

#[derive(Default)]
pub struct WavBuffer<T> {
    pub samples: Vec<AudioSample<T>>,
    pub sample_index: usize,
}

impl AudioBuffer<f32> for WavBuffer<f32> {
    fn read(&mut self, out: &mut [AudioSample<f32>]) -> usize {
        if self.sample_index >= self.samples.len() {
            return 0;
        }

        let buff = &self.samples[self.sample_index..];
        let out_len = out.len();
        let buff_len = buff.len();
        let mut samples_read = 0;

        buff.iter()
            .enumerate()
            .take_while(|&(i, _)| i < out_len.min(buff_len))
            .for_each(|(i, &s)| {
                out[i] = s;
                samples_read += 1;
            });

        self.sample_index += samples_read;

        samples_read
    }

    fn seek_to_start(&mut self) {
        self.sample_index = 0;
    }
}

impl AudioBuffer<f32> for WavBuffer<i16> {
    //stores samples stored as i16 internalls but reads out as f32
    fn read(&mut self, out: &mut [AudioSample<f32>]) -> usize {
        if self.sample_index >= self.samples.len() {
            return 0;
        }

        let buff = &self.samples[self.sample_index..];
        let out_len = out.len();
        let buff_len = buff.len();
        let mut samples_read = 0;

        buff.iter()
            .enumerate()
            .take_while(|&(i, _)| i < out_len.min(buff_len))
            .for_each(|(i, &s)| {
                out[i] = AudioSample::from([
                    s.channel[0] as f32 / 65535.0f32,
                    s.channel[1] as f32 / 65535.0f32,
                ]);
                samples_read += 1;
            });

        self.sample_index += samples_read;
        samples_read
    }

    fn seek_to_start(&mut self) {
        self.sample_index = 0;
    }
}
