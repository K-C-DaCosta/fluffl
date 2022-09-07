use crate::{
    audio::{Interval, PCMSlice},
    collections::{
        linked_list::{LLNodeOps, LLOps, LinkedList},
        segment_tree::{index_types::GlobalIndex, CircularSegmentTree, TreeIterState},
        Ptr,
    },
    math::Vec4,
    math::FP64,
    mem,
};

use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
};

pub mod protocol;

/// Provides an implementation for a mixer
pub mod standard;

pub mod streams;
pub mod time;

use self::protocol::{
    AddTrackErr, LocalRequestQueue, MixerEventKind, MixerRequest, MixerResponse, OffsetKind,
    RemoveTrackErr, RequestQueuePtr, ResponseQueuePtr, TrackID, TrackMutatedErr,
};
pub use self::time::SampleTime;

pub type MutatedResult<T> = Result<T, TrackMutatedErr>;

#[derive(Clone)]
pub struct MixerProtocol {
    requests: RequestQueuePtr,
    responses: ResponseQueuePtr,
}
impl MixerProtocol {
    pub fn submit_requests(&self, queue: &mut LocalRequestQueue) {
        self.requests.submit_requests(queue);
    }
    pub fn recieve_responses(&self) -> impl Iterator<Item = MixerResponse> + '_ {
        self.responses.recieve_responses()
    }
}

#[derive(Copy, Clone)]
struct MixerCursor {
    t0: SampleTime,
    delta: SampleTime,
}
impl MixerCursor {
    pub fn new(t0: SampleTime, delta: SampleTime) -> Self {
        Self {
            t0: t0,
            delta: delta,
        }
    }

    pub fn to_interval_ms(&self) -> Interval {
        let lo = self.t0.elapsed_in_ms_fp();
        let hi = self.t0.sum(&self.delta).elapsed_in_ms_fp();
        Interval { lo, hi }
    }

    #[allow(dead_code)]
    pub fn to_interval_tuple_ms_f32(&self) -> (f32, f32) {
        let lo = self.t0.elapsed_in_ms_f32();
        let hi = self.t0.sum(&self.delta).elapsed_in_ms_f32();
        (lo, hi)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StreamState {
    /// intended to be expressed in milliseconds, but stored as a rational number so it can be represented exactly
    pub local_time: SampleTime,
    /// interval is in milliseconds
    pub global_interval: Interval,
    /// attack time is in milliseconds
    pub attack_time: u32,
    /// in milliseconds
    pub release_time: u32,
    pub gain: f32,
    pub pan: f32,

    /// in samples per seconds
    pub frequency: u32,
    pub channels: u32,
}

#[derive(Copy, Clone)]
pub struct PullInfo {
    /// number of samples written to the `PCMSlice`
    pub samples_read: usize,
    /// number of samples per channel written to the `PCMSlice`
    pub samples_read_per_channel: usize,
    /// amount of time ,in ms, that it would take for the written audio to play
    pub elapsed_audio_in_ms: FP64,
}

pub trait HasAudioStream: Send + Debug {
    fn stream_state(&self) -> &StreamState;
    fn stream_state_mut(&mut self) -> &mut StreamState;

    fn interval(&self) -> &Interval {
        &self.stream_state().global_interval
    }

    fn gain(&self) -> f32 {
        self.stream_state().gain
    }

    fn set_gain(&mut self, new_gain: f32) {
        self.stream_state_mut().gain = new_gain.abs().max(0.0);
    }

    /// in samples per second
    fn frequency(&self) -> u32 {
        self.stream_state().frequency
    }

    fn interval_mut(&mut self) -> &mut Interval {
        &mut self.stream_state_mut().global_interval
    }
    ///given a time interval `dt`(in *milliseconds*) returns number of samples needed to represent the interval
    fn calculate_samples_needed(&self, dt: u32) -> u32 {
        const NUM_MILLISECONDS_IN_ONE_SECOND: u32 = 1000;
        (self.frequency() * dt) / NUM_MILLISECONDS_IN_ONE_SECOND
    }

    fn calculate_samples_needed_per_channel_fp(&self, dt: FP64) -> FP64 {
        let f = self.frequency();
        super::calculate_samples_needed_per_channel_fp(f, dt)
    }

    fn calculate_samples_needed_per_channel_f32(&self, dt: f32) -> f32 {
        const NUM_MILLISECONDS_IN_ONE_SECOND: f32 = 1000.0;
        (self.frequency() as f32 * dt) / NUM_MILLISECONDS_IN_ONE_SECOND
    }

    fn time_remaining_in_ms(&self) -> FP64 {
        let state = self.stream_state();
        let local_t = state.local_time.elapsed_in_ms_fp();
        let interval = state.global_interval;
        interval.distance() - local_t
    }

    fn is_dead(&self) -> bool {
        let state = self.stream_state();
        let local_t = state.local_time.elapsed_in_ms_fp();
        let interval = state.global_interval;
        local_t > interval.distance()
    }

    /// ## Description
    /// Pulls samples and write to `audio_pcm` \
    /// ## Returns
    /// info about how much data was written, the elapsed time that the written audio would take to pay it, etc
    /// ## Comments
    /// - will advance the `local_time` cursor that **ALL** streams must have
    /// - the samples that the stream pulls internally and writes to `audio_pcm` **MUST** be in the same format as specified by PCMSlice.\
    /// For exmaple, if  the stream is 4 channels interleaved internally but `audio_pcm` is 2 channels interleaved then this function **must** convert
    /// the internal stream to 2 channels before writing to `audio_pcm`.
    ///     - Use `scratch_space` to do the conversion
    ///     - note: the mixer *WILL* break if the channel conversion doesn't happen
    fn pull_samples<'a>(
        &mut self,
        scratch_space: &mut [f32],
        audio_pcm: PCMSlice<'a, f32>,
    ) -> PullInfo;

    fn seek(&mut self, global_time: SampleTime);
}

/// mixes sounds together (assumed 2 channels for now)
pub struct Mixer {
    request_queue: RequestQueuePtr,
    response_queue: ResponseQueuePtr,

    /// all responses in here eventually get forwarded to `response_queue` , however, `response_queue`\
    /// requires a lock in order to write to it which can be inconvinent in certain areas, \
    /// hence this queues exists to solve that
    local_response_queue: VecDeque<MixerResponse>,

    /// ## Description
    /// controls how 'fast' the mixer will play the track ( will effect the pitch )
    /// by default this is set to '1', but if you want to play the chart to times as fast
    /// you set this to '2'
    speed_factor: FP64,

    /// essentially a rational number, perfectly represents global time
    global_t: SampleTime,

    /// a temporary buffer used as scratch space for pulling in audio from streams/waves
    stream_scratch_space: Vec<f32>,

    /// a temporary buffer that is used to mix audio
    sample_scratch_space: Vec<f32>,

    /// datastructure stores tracks, can be used to quickly search through
    /// a massive number of tracks
    track_chart: CircularSegmentTree<Box<dyn HasAudioStream>>,

    /// maps a track id to global index
    track_id_table: HashMap<TrackID, GlobalIndex>,

    /// streams currently running
    running_streams_table: HashMap<GlobalIndex, Ptr>,

    /// streams where the cursor first intersects the track
    running_streams_on_intersection: Vec<GlobalIndex>,

    /// streams where the cursor has already intersected the track
    running_streams: LinkedList<GlobalIndex>,

    /// a stack used to remove tracks in the removal routine
    track_removal_stack: Vec<Ptr>,
}
impl Mixer {
    pub fn new(sample_rate: u32, _channels: u32) -> Self {
        Self {
            request_queue: RequestQueuePtr::new(),
            response_queue: ResponseQueuePtr::new(),
            global_t: SampleTime::new().with_sample_rate(sample_rate),
            running_streams_on_intersection: Vec::new(),
            running_streams: LinkedList::new(),
            running_streams_table: HashMap::new(),
            track_chart: CircularSegmentTree::new(30, 1 << 30),
            track_removal_stack: vec![],
            track_id_table: HashMap::new(),
            // this buffer should be sufficiently larger than any PCMSlice
            // the data is allocated once and will
            stream_scratch_space: vec![0.0f32; 8192 * 2],

            // another scrach slice used for mixing audio
            sample_scratch_space: vec![0.0f32; 8192 * 2],
            local_response_queue: VecDeque::new(),
            speed_factor: FP64::from(1),
        }
    }

    fn mix_audio<'a>(&mut self, mut output_buffer: PCMSlice<'a, f32>) {
        //the 'length' of the cursor determines the global speed of the mixer
        //this controls the speed of the mixer
        let new_samples_per_channel =
            FP64::from(output_buffer.samples_per_channel()) * self.speed_factor;
        let cursor = MixerCursor::new(
            self.global_t,
            self.global_t
                .with_sample_count(new_samples_per_channel.as_i64().max(0) as u64),
        );

        output_buffer.set_zero();

        self.search_for_active_tracks(cursor);

        self.mix_active_tracks(cursor, output_buffer);

        self.remove_irrelevent_tracks(cursor);

        self.handle_user_requests(cursor);

        self.forward_local_responses_back_to_client();

        //update t
        self.global_t.increment(cursor.delta.samps());
    }
    fn mix_active_tracks(&mut self, cursor: MixerCursor, output_buffer: PCMSlice<f32>) {
        // its easier to pull audio from tracks KNOWING that the cursor for the output buffer starts at ZERO
        self.handle_intersecting_tracks_not_first_time(cursor, output_buffer);

        // tracks that intersect the cursor for the first time are handled differently than
        self.handle_intersecting_tracks(cursor, output_buffer);
    }
    fn handle_intersecting_tracks(
        &mut self,
        cursor: MixerCursor,
        mut output_buffer: PCMSlice<f32>,
    ) {
        let track_chart = &mut self.track_chart;
        let running_streams_table = &mut self.running_streams_table;
        let running_streams_on_intersection = &mut self.running_streams_on_intersection;
        let running_streams_on_intersection_ptr =
            running_streams_on_intersection as *const Vec<GlobalIndex>;
        let running_streams = &mut self.running_streams;
        let stream_scratch_space = self.stream_scratch_space.as_mut_slice();
        let sample_scratch_space = &mut self.sample_scratch_space;

        // println!("list len = {}",running_streams_on_intersection.len());

        let iter = running_streams_on_intersection.iter().enumerate().rev();

        let delta = cursor.delta.elapsed_in_ms_fp();

        for (stream_vec_idx, &gi) in iter {
            let current_track = &mut track_chart[gi];

            let samples_needed_per_channel = current_track
                .calculate_samples_needed_per_channel_fp(delta)
                .ceil()
                .as_i64() as usize;

            let samples_needed = samples_needed_per_channel * 2;

            //actually pull required pulses from track
            let PullInfo { samples_read, .. } = current_track.pull_samples(
                stream_scratch_space,
                output_buffer.with_slice(&mut sample_scratch_space[0..samples_needed]),
            );

            //sound gets added to
            resample_and_mix_assumed_2_channels(
                &sample_scratch_space[0..samples_read],
                &mut output_buffer[..],
            );

            let stream_idx = unsafe {
                let on_intersection_ref =
                    mem::force_ptr_to_ref_mut(running_streams_on_intersection_ptr);
                //it should be safe to swap and pop while iterating in reverse
                on_intersection_ref.swap_remove(stream_vec_idx)
            };

            // add the stream index to the linkedlist of non-attack streams
            running_streams.push_rear(stream_idx);

            //assign pointer in the table to a valid linkedlist address/offset because
            //previously it was set to NULL
            let stream_ptr = running_streams.get_rear();
            if let Some(ptr) = running_streams_table.get_mut(&stream_idx) {
                *ptr = stream_ptr;
            }
        }
    }
    fn handle_intersecting_tracks_not_first_time(
        &mut self,
        cursor: MixerCursor,
        mut output_buffer: PCMSlice<f32>,
    ) {
        let track_chart = &mut self.track_chart;
        let running_streams = &mut self.running_streams;
        let stream_scratch_space = self.stream_scratch_space.as_mut_slice();
        let sample_scratch_space = &mut self.sample_scratch_space;

        let elapsed_time = cursor.delta.elapsed_in_ms_fp();

        running_streams
            .iter()
            .filter_map(|e| e.get_data())
            .for_each(|&gi| {
                // println!("elapsed = {}",cursor_elapsed);

                let current_track = &mut track_chart[gi];
                let elapsed_time_in_ms = elapsed_time;

                //samples needed to represent `cursor_elaped` time
                let samples_required_to_pull_from_track =
                    current_track.calculate_samples_needed_per_channel_fp(elapsed_time_in_ms) * 2;

                let samples_required_to_pull_from_track_truncated =
                    samples_required_to_pull_from_track.ceil().as_i64() as usize;
                // .min((output_buffer.len()) as i64) as usize;

                let PullInfo { samples_read, .. } = current_track.pull_samples(
                    stream_scratch_space,
                    output_buffer.with_slice(
                        &sample_scratch_space[0..samples_required_to_pull_from_track_truncated],
                    ),
                );

                //sound gets added to
                resample_and_mix_assumed_2_channels(
                    &sample_scratch_space[0..samples_read],
                    &mut output_buffer[..],
                );
            });
    }

    fn search_for_active_tracks(&mut self, cursor: MixerCursor) {
        let track_chart = &mut self.track_chart;
        let running_streams_on_intersection = &mut self.running_streams_on_intersection;
        let running_streams_table = &mut self.running_streams_table;
        let local_response_queue = &mut self.local_response_queue;
        let track_id_table = &mut self.track_id_table;

        track_chart
            .search_interval(&mut TreeIterState::new(), cursor.to_interval_ms())
            .for_each(|(gi, _)| {
                if running_streams_table.contains_key(&gi) == false {
                    running_streams_on_intersection.push(gi);
                    // At this stage it is good enough to know that the stream is being mixed
                    // Inserting (GlobalIndex, Ptr::NULL) into the table tells us the stream is being mixed but
                    // hasn't been added to the running_streams linkedlist yet (its still intersecting)
                    running_streams_table.insert(gi, Ptr::null());

                    let track_id = track_id_table
                        .iter()
                        .find(|&(_, &v)| v == gi)
                        .map(|(&k, _)| k)
                        .expect("track_id should exist");

                    local_response_queue.push_back(MixerResponse::MixerEvent(
                        MixerEventKind::TrackStarted(track_id),
                    ));

                    // println!("[{:?}] added",gi);
                }
            });
    }

    fn remove_irrelevent_tracks(&mut self, _cursor: MixerCursor) {
        self.remove_irrelevent_tracks_predicate(|track| {
            track.time_remaining_in_ms() < FP64::from(1)
        })
    }

    fn remove_irrelevent_tracks_predicate<Predicate>(&mut self, can_be_removed: Predicate)
    where
        Predicate: Fn(&Box<dyn HasAudioStream>) -> bool,
    {
        let track_chart = &mut self.track_chart;
        let running_streams = &mut self.running_streams;
        let running_streams_table = &mut self.running_streams_table;
        let track_removal_stack = &mut self.track_removal_stack;
        let track_id_table = &mut self.track_id_table;
        let local_response_queue = &mut self.local_response_queue;

        for node_ptr in running_streams.node_index_iter() {
            let &gi = running_streams[node_ptr]
                .get_data()
                .expect("should be available");

            if can_be_removed(&track_chart[gi]) {
                //queue track to be removed
                track_removal_stack.push(node_ptr);
                //remove it from the table as well
                running_streams_table.remove(&gi);

                let track_id = track_id_table
                    .iter()
                    .find(|&(_, &v)| v == gi)
                    .map(|(&k, _)| k)
                    .expect("track_id should exist");

                local_response_queue.push_back(MixerResponse::MixerEvent(
                    MixerEventKind::TrackStopped(track_id),
                ));

                // println!("track [{:?}] bumped off running_streams list...", gi);
            }
        }

        while let Some(node_ptr) = track_removal_stack.pop() {
            running_streams.remove(node_ptr);
        }
    }

    fn forward_local_responses_back_to_client(&mut self) -> Option<()> {
        let local_response_queue = &mut self.local_response_queue;
        let mut response_queue = self.response_queue.lock()?;

        while let Some(local_response) = local_response_queue.pop_front() {
            response_queue.push_back(local_response);
        }

        Some(())
    }

    fn handle_user_requests(&mut self, cursor: MixerCursor) -> Option<()> {
        let mixer_ref = unsafe { mem::force_static_mut(self) };
        let track_chart = &mut self.track_chart;
        let track_id_table = &mut self.track_id_table;
        let running_streams_table = &mut self.running_streams_table;
        let global_t = &mut self.global_t;
        let mut request_queue = self.request_queue.lock()?;
        let mut response_queue = self.response_queue.lock()?;

        let current_time = cursor.t0.elapsed_in_ms_fp();

        while let Some(req) = request_queue.pop_front() {
            match req {
                MixerRequest::FetchMixerTime => {
                    response_queue.push_back(MixerResponse::MixerTime(*global_t))
                }
                MixerRequest::AddTrack(tid, off, track) => {
                    response_queue.push_back(MixerResponse::AddTrackStatus(
                        tid,
                        Self::request_operation_add_track(
                            track_chart,
                            track_id_table,
                            current_time,
                            tid,
                            off,
                            track,
                        ),
                    ));
                }
                MixerRequest::MutateMixer(tid, callback) => response_queue.push_back(
                    MixerResponse::MixerMutatedStatus(tid, callback(tid, mixer_ref)),
                ),
                MixerRequest::RemoveTrack(tid) => {
                    response_queue.push_back(MixerResponse::RemoveTrackStatus(
                        tid,
                        Self::request_operation_remove_track(
                            tid,
                            track_id_table,
                            running_streams_table,
                            track_chart,
                        ),
                    ));
                }
                MixerRequest::Seek(offset_kind) => {
                    Self::request_operation_seek(track_chart, global_t, offset_kind);
                    mixer_ref.remove_irrelevent_tracks_predicate(|track| {
                        track.interval().is_within(global_t.elapsed_in_ms_fp()) == false
                    });
                }
            }
        }

        Some(())
    }

    fn request_operation_seek(
        track_chart: &mut CircularSegmentTree<Box<dyn HasAudioStream>>,
        global_t: &mut SampleTime,
        offset_kind: OffsetKind,
    ) {
        let global_t_in_ms = global_t.elapsed_in_ms_fp();

        let new_global_t = match offset_kind {
            OffsetKind::Current { offset } => {
                let offset = FP64::from(offset);
                global_t.from_time_in_ms_fp((global_t_in_ms + offset).max(FP64::zero()))
            }
            OffsetKind::Start { offset } => {
                let offset = FP64::from(offset);
                global_t.from_time_in_ms_fp((offset).max(FP64::zero()))
            }
        };

        //update global_t
        *global_t = new_global_t;

        //update tracks
        for track in track_chart.values_mut() {
            track.seek(new_global_t);
        }
    }

    fn request_operation_add_track(
        track_chart: &mut CircularSegmentTree<Box<dyn HasAudioStream>>,
        track_id_table: &mut HashMap<TrackID, GlobalIndex>,
        current_time: FP64,
        tid: TrackID,
        off: OffsetKind,
        mut track: Box<dyn HasAudioStream>,
    ) -> Result<(), AddTrackErr> {
        if track_id_table.contains_key(&tid) {
            return Err(AddTrackErr::TrackIdAlreadyExists(track));
        }

        let offset_interval = match off {
            OffsetKind::Current { offset } => {
                *track.interval() + (current_time + FP64::from(offset))
            }
            OffsetKind::Start { offset } => {
                let interval_length = track.interval().distance();
                Interval::from_point_and_length(FP64::from(offset), interval_length)
            }
        };

        // update track interval
        *track.interval_mut() = offset_interval;

        // insert the track and store the tracks alias
        let global_idx = track_chart.insert(offset_interval, track);

        // make sure to bind the tid to the tracks alias
        track_id_table.insert(tid, global_idx);

        //send a response to user that it has been added
        Ok(())
    }

    fn request_operation_remove_track(
        tid: TrackID,
        track_id_table: &mut HashMap<TrackID, GlobalIndex>,
        running_streams_table: &mut HashMap<GlobalIndex, Ptr>,
        track_chart: &mut CircularSegmentTree<Box<dyn HasAudioStream>>,
    ) -> Result<Box<dyn HasAudioStream>, RemoveTrackErr> {
        let &global_idx = track_id_table
            .get(&tid)
            .ok_or(RemoveTrackErr::TrackNotFound)?;

        let track_is_currently_playing = running_streams_table.contains_key(&global_idx);
        if track_is_currently_playing {
            return Err(RemoveTrackErr::TrackCurrentlyPlaying);
        }

        // track is not playing and can be removed easily
        track_id_table
            .remove(&tid)
            .expect("tid should already exist");

        let item = track_chart
            .remove_by_global_idx(global_idx)
            .expect("item should exist");

        Ok(item)
    }

    fn protocol(&self) -> MixerProtocol {
        MixerProtocol {
            requests: self.request_queue.clone(),
            responses: self.response_queue.clone(),
        }
    }

    /// sets mixer `speed`.
    /// ## Comments
    /// - this should be obvious but it will affect the mixers internal timing
    /// - examples of values for `speed`:
    ///     - `1` is normal speed
    ///     - `2` twice as fast speed
    ///     - `0.5` is twice as slow
    ///     - `0` and the mixer comes to a complete stop
    pub fn set_mixer_speed(&mut self, speed: FP64) -> MutatedResult<()> {
        self.speed_factor = speed.max(FP64::zero());
        Ok(())
    }

    /// prints the `track_chart` tree
    /// ## Comments
    /// - the mixer uses this tree to find tracks quickly
    pub fn print_tree(&self) -> MutatedResult<()> {
        self.track_chart.print_tree(".");
        Ok(())
    }

    /// fetches the track_interval
    /// ## Complexity
    /// **O**(1)    
    pub fn track_get_interval(&self, tid: TrackID) -> MutatedResult<Interval> {
        let track_id_table = &self.track_id_table;
        let track_chart = &self.track_chart;
        let &gid = track_id_table
            .get(&tid)
            .ok_or(TrackMutatedErr::TrackNotFound)?;
        let current_track = &track_chart[gid];

        let &interval = current_track.interval();
        Ok(interval)
    }

    /// resizes the track interval
    /// ## Comments
    /// - there's more to it, but the basic idea is to re-insert the track into the segment tree with a `new_interval`
    /// ## Complexity
    /// **O**(log(`n`)), where `n` is number of buckets in the tree
    pub fn track_set_interval(
        &mut self,
        tid: TrackID,
        new_interval: Interval,
    ) -> MutatedResult<()> {
        let &current_track_gid = self
            .track_id_table
            .get(&tid)
            .ok_or(TrackMutatedErr::TrackNotFound)?;

        // remove all references of the track in the mixer pipeline
        let was_in_intersection = self
            .remove_track_references_from_the_track_intersection_phase(current_track_gid)
            .is_some();
        let was_in_running = self
            .remove_track_references_from_the_running_track_phase(current_track_gid)
            .is_some();

        // remove the track from the tree
        let track_chart = &mut self.track_chart;
        let mut track = track_chart
            .remove_by_global_idx(current_track_gid)
            .expect("track should exist, since it also exists in the track_id_table");

        // update track with a new interval
        *track.interval_mut() = new_interval;

        //re-insert track into tree
        let new_gid = track_chart.insert(new_interval, track);

        // println!("{:?} -->{:?}", tid, new_gid);

        //update track_id_table with new GID
        let track_id_table = &mut self.track_id_table;
        *track_id_table
            .get_mut(&tid)
            .expect("tid should already exist in table") = new_gid;

        //if the track was being played, put it back into where it was in the pipeline
        if was_in_intersection != was_in_running {
            self.running_streams_table.insert(new_gid, Ptr::null());
            if was_in_intersection {
                self.running_streams_on_intersection.push(new_gid);
            } else {
                self.running_streams.push_rear(new_gid);
                let ptr = self.running_streams.get_rear();
                *self
                    .running_streams_table
                    .get_mut(&new_gid)
                    .expect("key should exist") = ptr;
            }
        }

        Ok(())
    }

    fn remove_track_references_from_the_track_intersection_phase(
        &mut self,
        current_track_gid: GlobalIndex,
    ) -> Option<()> {
        //track being re-inserted could be here
        let running_streams_on_intersection = &mut self.running_streams_on_intersection;
        let running_streams_table = &mut self.running_streams_table;

        let item_that_match_gid = running_streams_on_intersection
            .iter_mut()
            .enumerate()
            .map(|(a, &mut b)| (a, b))
            .rev()
            .find(|&(_, gi)| gi == current_track_gid)
            .map(|(k, _)| k)?;

        running_streams_on_intersection.swap_remove(item_that_match_gid);
        running_streams_table.remove(&current_track_gid);

        Some(())
    }
    fn remove_track_references_from_the_running_track_phase(
        &mut self,
        current_track_gid: GlobalIndex,
    ) -> Option<()> {
        //track being re-inserted could also be here
        let running_streams = &mut self.running_streams;
        let running_streams_table = &mut self.running_streams_table;

        let ptr = running_streams_table.get(&current_track_gid).map(|&a| a)?;
        if ptr.is_null() {
            return None;
        }
        running_streams
            .remove(ptr)
            .expect("pointer registered in table, so should exist here as well");
        running_streams_table.remove(&current_track_gid);

        Some(())
    }
    pub fn get_time(&self) -> SampleTime {
        self.global_t
    }
}

/// `src` and `dst` are both assumed to be 2 channels interleaved
fn resample_and_mix_assumed_2_channels(src: &[f32], dst: &mut [f32]) {
    mix_resample_audio_both_2_channels_iterator_version_vectorized(src, dst)
}

#[allow(dead_code)]
fn mix_resample_audio_both_2_channels_slow_reference(src: &[f32], dst: &mut [f32]) {
    const NUM_CHANNELS: usize = 2;
    let src_sample_count = src.len() / NUM_CHANNELS;
    let dst_sample_count = dst.len() / NUM_CHANNELS;
    if src_sample_count == 0 || dst_sample_count == 0 {
        return;
    }
    let scale_ratio = src_sample_count as f32 / dst_sample_count as f32;
    for dst_i in 0..dst_sample_count {
        let src_i_estimate = dst_i as f32 * scale_ratio;
        let src_i = src_i_estimate as usize;
        let lerp_t = src_i_estimate.fract();
        //interpolate both channels
        for k in 0..NUM_CHANNELS {
            // accumulate destination here
            let dst_index_sub_sample = NUM_CHANNELS * dst_i + k;

            let cur_block = (src_i + 0).max(0);
            let nxt_block = (src_i + 1).min(src_sample_count - 1);
            let cur = src[NUM_CHANNELS * cur_block + k];
            let nxt = src[NUM_CHANNELS * nxt_block + k];

            let old_value = dst[dst_index_sub_sample];
            //interpolated src
            let new_value = (nxt - cur) * lerp_t + cur;
            // let mixed_value = old_value + new_value;
            let mixed_value = old_value + new_value;
            dst[dst_index_sub_sample] = mixed_value;
        }
    }
}

#[allow(dead_code)]
fn mix_resample_audio_both_2_channels_iterator_version_vectorized(src: &[f32], dst: &mut [f32]) {
    const NUM_CHANNELS: usize = 2;

    let src_sample_count = src.len() / NUM_CHANNELS;
    let dst_sample_count = dst.len() / NUM_CHANNELS;

    if src_sample_count == 0 || dst_sample_count == 0 {
        return;
    }

    let scale_ratio = src_sample_count as f32 / dst_sample_count as f32;

    dst.chunks_mut(2)
        .enumerate()
        .flat_map(|(dst_i, dst_chunk)| {
            let src_i_estimate = dst_i as f32 * scale_ratio;
            let src_i = src_i_estimate as usize;
            let lerp_t = src_i_estimate.fract();
            let cur_block = (src_i + 0).max(0);
            let nxt_block = (src_i + 1).min(src_sample_count - 1);
            //interpolate both channels
            dst_chunk.iter_mut().enumerate().map(move |(k, dst)| {
                let (cur, nxt) = unsafe {
                    (
                        src.get_unchecked(NUM_CHANNELS * cur_block + k),
                        src.get_unchecked(NUM_CHANNELS * nxt_block + k),
                    )
                };
                let new_value = (nxt - cur) * lerp_t + cur;
                (dst, new_value)
            })
        })
        .for_each(|(dst, new_value)| *dst += new_value);
}

#[allow(dead_code)]
pub fn mix_resample_audio_test(src: &[f32], dst: &mut [f32]) {
    const NUM_CHANNELS: usize = 2;
    let src_sample_count = src.len() / NUM_CHANNELS;
    let dst_sample_count = dst.len() / NUM_CHANNELS;
    if src_sample_count == 0 || dst_sample_count == 0 {
        return;
    }

    let delta = src_sample_count as f32 / dst_sample_count as f32;
    let mut src_full = Vec4::from([0., 1., 2., 3.]) * delta;
    let step = Vec4::from([2.0; 4]) * delta;
    dst.chunks_mut(4).for_each(|chunks| {
        chunks.iter_mut().enumerate().for_each(|(k, dst)| unsafe {
            let src_floor = *src_full.get_unchecked(k / 2) as usize;
            *dst += *src.get_unchecked(NUM_CHANNELS * src_floor);
        });
        src_full += step;
    });
}
pub fn integrate(cur_lst: &mut [Vec4<f32>], pre_lst: &mut [Vec4<f32>]) {
    for k in 0..cur_lst.len() {
        let pre = pre_lst[k];
        let cur = cur_lst[k];
        let new = (cur - pre) + cur;
        pre_lst[k] = cur;
        cur_lst[k] = new;
    }
}

pub fn integrate2(cur_lst: &mut [Vec4<f32>], pre_lst: &mut [Vec4<f32>]) {
    cur_lst
        .iter_mut()
        .zip(pre_lst.iter_mut())
        .for_each(|(cur, pre)| {
            let new = (*cur - *pre) + *cur;
            *pre = *cur;
            *cur = new;
        });
}


pub fn roots(a: &[f32], b: &[f32], c: &[f32], root: &mut [f32]) {
    root.iter_mut()
        .zip(a.iter().zip(b.iter().zip(c.iter())))
        .for_each(|(root, (&a, (&b, &c)))| {
            *root = -(b + (b * b - 4.0 * a * c).sqrt()) / (2.0 * a);
        })
}

