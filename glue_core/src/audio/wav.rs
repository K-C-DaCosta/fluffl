use super::{AudioBuffer, AudioSample, PcmConverter};
use crate::GlueError;
use std::io;
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

    pub fn parse(mut self) -> Result<Self, GlueError> {
        let res = self.data.as_ref().map(|data| {
            let mut data_slice = &data[..];
            let mut header_bytes = [0u8; 44];
            data_slice.read(&mut header_bytes).map(|bytes_read| {
                let header = *unsafe { mem::transmute::<_, &WavHeader>(header_bytes.as_ptr()) };
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

        match &self.header {
            &Some(WavHeader {
                bits_per_sample,
                audio_format,
                ..
            }) => {
                if bits_per_sample != 16 && bits_per_sample != 8 {
                    return Err(wav_parse_error(
                        "Only 8 and 16 bit precision pcm is supported",
                    ));
                }
                if audio_format != 1 {
                    return Err(wav_parse_error("Only uncompressed wav files is supported"));
                }
            }
            _ => (),
        }

        Ok(self)
    }
}

impl PcmConverter<f32> for WavFile {
    fn samples(&self) -> Vec<AudioSample<f32>> {
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

impl<T> Into<AudioBuffer<T>> for Vec<AudioSample<T>> {
    fn into(self) -> AudioBuffer<T>{
        AudioBuffer {
            samples: self,
            sample_index:0,
        }
    }
}

fn wav_parse_error(msg: &str) -> GlueError {
    GlueError::WavParseError(String::from(msg))
}
