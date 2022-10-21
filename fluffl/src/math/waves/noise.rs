const NOISE_TABLE_LEN: usize = 4099;
const NOISE_TABLE_CAPACITY: usize = 4100;

static mut UNIFORM_NOISE_TABLE: [f32; NOISE_TABLE_CAPACITY] = [0.0; NOISE_TABLE_CAPACITY];

const PRIMES_TABLE: &[usize] = &[
    73000003, 73000013, 73000019, 73000043, 73000061, 73000063, 73000093, 73000099, 73000153,
    73000157, 73000171, 73000177, 73000189, 73000199, 73000201, 73000211, 73000217, 73000237,
    73000241, 73000247, 73000267, 73000273, 73000289, 73000339, 73000387, 73000423, 73000427,
    73000439, 73000457, 73000471, 73000481, 73000547, 73000549, 73000553, 73000561, 73000579,
    73000583, 73000607, 73000619, 73000621, 73000633, 73000639, 73000649, 73000687, 73000783,
    73000841, 73000843, 73000867, 73000883, 73000891, 73000903, 73000909, 73000913, 73000927,
    73000931, 73000969, 73000999, 73001003, 73001023, 73001119, 73001141, 73001171, 73001177,
    73001221, 73001237, 73001263, 73001267, 73001273, 73001309, 73001321, 73001393, 73001399,
    73001413, 73001431, 73001437, 73001441, 73001473, 73001479, 73001531, 73001543, 73001581,
    73001603, 73001623, 73001647, 73001651, 73001653, 73001681, 73001693, 73001707, 73001711,
    73001717, 73001783, 73001801, 73001809, 73001813, 73001849, 73001881, 73001921, 73001923,
    73001947, 73001989, 73002031, 73002053, 73002067, 73002071, 73002079, 73002101, 73002109,
    73002113, 73002119, 73002121, 73002133, 73002151, 73002157, 73002169, 73002173, 73002191,
    73002211, 73002217, 73002257, 73002277, 73002287, 73002289, 73002341, 73002343, 73002353,
    73002467, 73002469, 73002473, 73002493, 73002511, 73002533, 73002581, 73002583, 73002593,
    73002619, 73002623, 73002707, 73002719, 73002749, 73002757, 73002817, 73002827, 73002833,
    73002857, 73002877, 73002967, 73002971, 73002977, 73002983, 73002991, 73003019, 73003027,
    73003039, 73003061, 73003067, 73003087, 73003093, 73003097, 73003129, 73003141, 73003157,
    73003169, 73003193, 73003199, 73003213, 73003223, 73003261, 73003289, 73003297, 73003303,
    73003309, 73003319, 73003363, 73003373, 73003387, 73003393, 73003417, 73003421, 73003433,
    73003451, 73003487, 73003501, 73003531, 73003547, 73003573, 73003583, 73003589, 73003613,
    73003621, 73003649, 73003673, 73003687, 73003699, 73003703, 73003717, 73003741, 73003781,
    73003789, 73003823,
];

pub fn hash<const N: usize>(position: [f32; N]) -> usize {
    let mut hash = position[0] as usize * PRIMES_TABLE[0];
    for k in 1..N {
        hash ^= position[k] as usize * PRIMES_TABLE[k]
    }
    hash
}

pub fn value_noise_1d_octaves(x: f32, num_octaves: usize) -> f32 {
    let mut output = 0.0;
    let max_divisor = ((1 << num_octaves) - 1) as f32;
    for octave in 0..num_octaves {
        let divisor = (1 << octave) as f32;
        output += value_noise_1d(x / divisor) * divisor;
    }
    output / max_divisor
}

pub fn value_noise_1d(x: f32) -> f32 {
    unsafe {
        let block_idx = x;
        let t = block_idx.fract();
        let fade_t = fade(t);
        let f0 = UNIFORM_NOISE_TABLE[hash([block_idx + 0.0]) % NOISE_TABLE_LEN];
        let f1 = UNIFORM_NOISE_TABLE[hash([block_idx + 1.0]) % NOISE_TABLE_LEN];
        (f1 - f0) * fade_t + f0
    }
}
pub fn uniform(x: f32) -> f32 {
    unsafe {
        let block_idx = x;
        UNIFORM_NOISE_TABLE[hash([block_idx + 0.0, 10.0]) % NOISE_TABLE_LEN]
    }
}

pub fn init() {
    let noise_table = unsafe { &mut UNIFORM_NOISE_TABLE };
    let mut state = 12312;
    for table_element in noise_table.iter_mut() {
        *table_element = {
            let num = lehmer64(&mut state) % 4099;
            num as f32 / 4099.0
        };
        // println!("{:?}",table_element);
    }
}

pub fn perlin_noise_1d(p: f32) -> f32 {
    let p0 = p.floor();
    let p1 = p0 + 1.0;

    let t = p - p0;
    let fade_t = fade(t);

    let g0 = grad(p0);
    let g1 = grad(p1);

    (1.0 - fade_t) * g0 * (p - p0) + fade_t * g1 * (p - p1)
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn grad(p: f32) -> f32 {
    let v = unsafe { UNIFORM_NOISE_TABLE[hash([p, p]) % NOISE_TABLE_LEN] };
    if v > 0.5 {
        1.0
    } else {
        -1.0
    }
}

fn lehmer64(g_lehmer64_state: &mut u128) -> u64 {
    *g_lehmer64_state = g_lehmer64_state.wrapping_mul(0xda942042e4dd58b5);
    (*g_lehmer64_state >> 64) as u64
}
