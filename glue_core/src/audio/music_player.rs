use super::{AudioBuffer,AudioSample};
#[derive(Clone, Copy)]
pub enum PlayState {
    RampUp(usize),
    RampDown(usize),
    Playing,
    Paused,
}

pub struct MusicPlayer {
    pub ticks: usize,
    pub state: PlayState,
    pub volume: f32,
    pub music_src: AudioBuffer<f32>,
    pub channels: usize,
}

/// This callback assumes that samples are interleaved and works for two channels ONLY
pub fn music_player_callback(mp: &mut MusicPlayer, out: &mut [f32]) {
    if let PlayState::Paused = mp.state {
        out.iter_mut().for_each(|e| *e = 0.);
        return;
    }
    let num_channels = mp.channels;
    let samples = out.len();
    let mut input_samples = Vec::new();
    input_samples.resize(samples / num_channels, AudioSample::from([0f32; 2]));
   

    let vol = mp.volume;

    mp.state = match mp.state {
        PlayState::RampUp(max_ticks) => {
            if mp.ticks > max_ticks {
                PlayState::Playing
            } else {
                mp.state
            }
        }
        PlayState::RampDown(max_ticks) => {
            if mp.ticks > max_ticks {
                PlayState::Paused
            } else {
                mp.state
            }
        }
        _ => mp.state,
    };

    let samples_read = mp.music_src.read(&mut input_samples[..]);
    let inv_out_len = 1.0 / (out.len() as f32);
    let play_state = mp.state; 

    if samples_read == 0 {
        return;
    }

    for k in (0..out.len()).step_by(num_channels) {
        let j = (k * samples_read) as f32 * inv_out_len;
        let t = j.fract();

        let j0 = j as usize;
        let j1 = (j0 + 1).min((samples_read - 1).max(0));

        let samp0 = input_samples[j0];
        let samp1 = input_samples[j1];
        

        let exec = |samp0: AudioSample<_>, samp1: AudioSample<_>, channel_index| {
            let f0 = samp0.channel[channel_index];
            let f1 = samp1.channel[channel_index];
            //do some linear interpolation here
            let lerp = (f1 - f0) * t + f0;

            match play_state {
                PlayState::RampUp(max_ticks) => {
                    let t = (mp.ticks as f32 / max_ticks as f32).min(1.0).max(0.0);
                    lerp * vol * (t * t)
                }
                PlayState::RampDown(max_ticks) => {
                    let t = (mp.ticks as f32 / max_ticks as f32).min(1.0).max(0.0);
                    let linear_down = 1. - t;
                    lerp * vol * linear_down * linear_down
                }
                PlayState::Paused => 0.0,
                _ => lerp * vol,
            }
        };
        //write 'samples' into the output buffer
        //In this callback samples are assumed to be INTERLEAVED , not planar. 
        for j in 0..num_channels{
            out[k+j] = exec(samp0, samp1,1-j);
        }

        mp.ticks += 1;
    }
}