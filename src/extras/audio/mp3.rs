use crate::audio::{GenericAudioSpecs};
use super::{AudioSample, AudioBuffer};
use super::super::{Result,ErrorKind}; 

//mp3 decoding relies on puremp3 crate, its appears to have performance problems 
//but its the only crate I've found that targets wasm
pub use puremp3;
use puremp3::Mp3Decoder;


pub struct Mp3File {
    data: Option<Vec<u8>>,
    header: Option<puremp3::FrameHeader>,
}

impl Mp3File {
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
        if let Some(data) = &self.data {
            let mut decoder = puremp3::Mp3Decoder::new(&data[..]);
            let frame = decoder.next_frame()?;
            self.header = Some(frame.header);
        }
        Ok(self)
    }
}
impl GenericAudioSpecs for Mp3File {
    fn sample_rate(&self) -> Option<u32> {
        self.header.as_ref().map(|hdr| hdr.sample_rate.hz())
    }
    fn bits_per_sample(&self) -> Option<u32> {
        self.header.as_ref().map(|_hdr| 16)
    }
    fn channels(&self) -> Option<u32> {
        self.header
            .as_ref()
            .map(|hdr| hdr.channels.num_channels() as u32)
    }
}

impl From<puremp3::Error> for ErrorKind {
    fn from(_err: puremp3::Error) -> Self {
        Self::Mp3ParseError(String::from("failed to extract header from mp3"))
    }
}

impl Into<Mp3Buffer> for Mp3File {
    fn into(self) -> Mp3Buffer {
        let mut buffer = Mp3Buffer {
            mp3_data: self.data.unwrap(),
            decoder: None,
            current_frame: None,
            frame_index: 0,
            current_samples: 0,
            header: None,
        };
        buffer.header = Some(self.header.unwrap().clone());
        buffer.init_decoder();
        buffer
    }
}

pub struct Mp3Buffer {
    // literally just the mp3 file loaded entirely into memory
    mp3_data: Vec<u8>,

    // sound specs 
    header: Option<puremp3::FrameHeader>,

    // i'm basically lying about the lifetimes to the borrow checker. Its not really desireable,
    // but it will do for now. I think its worth mentioning that I wouldn't need to tell this lie
    // about the lifetimes if the library allowed the user to own the decoder state. Unfortunately,
    // the 'state' field of the Mp3Decoder struct is private. Oh well i guess. Maybe I just need to take a closer
    // look at the puremp3 docs and code for a better solution.
    decoder: Option<Mp3Decoder<&'static [u8]>>,

    // I guess mp3 format has things called frames that contain segments of PCM 
    current_frame: Option<puremp3::Frame>,

    // internal state of the buffer needed to resume playback
    frame_index: usize,
    current_samples: usize,
}
impl Mp3Buffer {
    fn init_decoder(&mut self) {
        //In this line im saying to the BC:"just trust me bro this slice is static." (its obviously not static)
        let slice = unsafe { std::mem::transmute(&self.mp3_data[..]) };
        self.decoder = Some(Mp3Decoder::new(slice));
    }
}

impl Drop for Mp3Buffer {
    fn drop(&mut self) {
        self.current_frame = None;
        self.decoder = None;
        self.mp3_data.clear();
    }
}

impl GenericAudioSpecs for Mp3Buffer{
    fn sample_rate(&self) -> Option<u32> {
        self.header.as_ref().map(|hdr| {
            hdr.sample_rate.hz()
        })
    }
    // the mp3 lib Im using makes this irrelevant
    fn bits_per_sample(&self) -> Option<u32> {        
        None
    }
    
    fn channels(&self)->Option<u32>{
        self.header.as_ref().map(|hdr| {
            hdr.channels.num_channels() as u32
        })
    }
}

impl AudioBuffer<f32> for Mp3Buffer {
    fn read(&mut self, out: &mut [AudioSample<f32>]) -> usize {
        let mut sample_read = 0;

        if let Some(decoder) = self.decoder.as_mut() {
            //get next frame if possible
            if self.current_frame.is_none() {
                self.frame_index = 0;

                self.current_frame = match decoder.next_frame() {
                    Ok(frame) => Some(frame),
                    _ => None,
                };

                self.current_samples = self
                    .current_frame
                    .as_ref()
                    .map(|cf| cf.num_samples)
                    .unwrap_or_default();
            }

            let mut out_index = 0;
            while self.current_frame.is_some() && out_index < out.len() {
                let frame = self.current_frame.as_ref().unwrap();
                out[out_index].channel[0] = frame.samples[1][self.frame_index];
                out[out_index].channel[1] = frame.samples[0][self.frame_index];
                self.frame_index += 1;
                sample_read += 1;
                out_index += 1;

                //if true then i've hit the end of the current frame, so fetch new frame if possible
                if self.frame_index >= self.current_samples {
                    self.frame_index = 0;

                    self.current_frame = match decoder.next_frame() {
                        Ok(frame) => Some(frame),
                        Err(_err) => {
                            println!("decoder error: {}", _err.to_string());
                            None
                        }
                    };

                    self.current_samples = self
                        .current_frame
                        .as_ref()
                        .map(|cf| cf.num_samples)
                        .unwrap_or_default();
                }
            }
        }
        sample_read
    }

    fn seek_to_start(&mut self) {
        self.current_frame = None;
        self.decoder = None;
        self.frame_index = 0;
        self.current_samples = 0;
        self.decoder = Some(Mp3Decoder::new(unsafe {
            std::mem::transmute(&self.mp3_data[..])
        }));
    }
}
