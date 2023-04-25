use std::ops::Deref;

use super::*;
use crate::audio::{AudioDeviceCore, DesiredSpecs, FlufflAudioContext, FlufflAudioDeviceContext};
use adhoc_audio::{StreamInfo, WavCodec};

pub type StandardMixerCB = fn(&mut StandardMixerState, &mut [f32]);

pub struct MixerAudioDeviceContext {
    protocol: MixerProtocol,
    device: FlufflAudioDeviceContext<StandardMixerCB, StandardMixerState>,
    local_requests: LocalRequestQueue,
    id_counter: u64,
}
impl MixerAudioDeviceContext {
    pub fn new(ctx: FlufflAudioContext) -> Self {
        let state = StandardMixerState::new(|state| {
            state.channels = 2;
            state.frequency = 44_100;
        });
        let protocol = state.protocol();
        Self {
            device: FlufflAudioDeviceContext::new(
                AudioDeviceCore::new()
                    .with_specs(DesiredSpecs {
                        sample_rate: Some(44_100),
                        channels: Some(2),
                        buffer_size: Some(2048),
                    })
                    .with_state(state)
                    .with_callback(standard_mixer_state_cb),
                ctx,
            ),
            protocol,
            local_requests: LocalRequestQueue::new(),
            id_counter: 0,
        }
    }

    /// ## Description
    /// generates a unique id for a track.
    /// ## Comments
    /// - ids are used to reference a track in the mixer
    pub fn gen_id(&mut self) -> TrackID {
        let id = self.id_counter;
        self.id_counter += 1;
        TrackID::from_u64(id)
    }

    pub fn dump_recording(&self) {
        // self.modify_state(|state|{
        //     let state = state?;
        //     let file = fs::File::create("mixer_dump.wav").ok()?;
        //     state.recording.save_to(file).ok()?;
        //     Some(())
        // });
    }

    /// ## Description
    /// Queues up a request
    /// ## Comments
    /// - You probably will not get a response back immediately after sending (may take multiple frames)
    /// - Once you send something **You must call ** `Self::recieve_responses(..)` to dequeue responses recieved from the mixer.
    ///     - if you dont call that function, response messages from the mixer will build up and eat your memory.
    pub fn send_request(&mut self, req: MixerRequest) {
        self.local_requests.enqueue(req);
        self.protocol.submit_requests(&mut self.local_requests);
    }

    /// ## Description
    /// Dequeues responses recived by the mixer
    pub fn recieve_responses(&mut self) -> impl Iterator<Item = MixerResponse> + '_ {
        self.protocol.recieve_responses()
    }
}

impl Deref for MixerAudioDeviceContext {
    type Target = FlufflAudioDeviceContext<StandardMixerCB, StandardMixerState>;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
/// The standard mixer has a final output of 2 channels and a sampling rate of 44_100 hz
pub struct StandardMixerState {
    pub mixer: Mixer,
    pub channels: u32,
    pub frequency: u32,
    pub recording: WavCodec,
}

impl StandardMixerState {
    pub fn new<CB>(mut init: CB) -> Self
    where
        CB: FnMut(&mut Self),
    {
        let mixer = Mixer::new(44_100, 2);
        let mut state = Self {
            mixer,
            channels: 2,
            frequency: 44_100,
            recording: WavCodec::new(StreamInfo::new(44_100, 2)),
        };
        init(&mut state);
        state
    }
}

impl std::ops::Deref for StandardMixerState {
    type Target = Mixer;
    fn deref(&self) -> &Self::Target {
        &self.mixer
    }
}

impl std::ops::DerefMut for StandardMixerState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mixer
    }
}

/// the standard callback for a mixer
fn standard_mixer_state_cb(state: &mut StandardMixerState, output: &mut [f32]) {
    state.mixer.mix_audio(PCMSlice::new(
        output,
        state.frequency,
        state.channels,
    ));
    //for debug purposes
    // state.recording.encode(output);
}
