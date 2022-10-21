use super::*;

#[rustfmt::skip]
pub fn perspective(l:f32,r:f32,t:f32,b:f32,n:f32,f:f32)->Mat4<f32>{
    Mat4::new().with_data([
        [2.*n/(r-l), 0.0        , (r+l)/(r-l), 0.0       ],
        [0.0       , 2.0*n/(t-b), (t+b)/(t-b), 0.0       ],
        [0.0       , 0.0        , (f+n)/(n-f), 2.*n/(n-f)],
        [0.0       , 0.0        ,  -1.0      , 0.0       ],
    ])
}

/// a orthographic matrix with fixed variables
#[rustfmt::skip]
pub fn calc_ortho_window_f32(w: f32, h: f32) -> Mat4<f32> {
    let data = [
        [2.0 / w,0.      ,0.   ,-1.0 ], 
        [0.     ,-2.0 / h,0.   , 1.0 ],
        [0.     ,0.      ,-0.01,  0. ],
        [0.     ,0.      ,0.   , 1.0 ],
    ];
    Mat4::new().with_data(data)
}
