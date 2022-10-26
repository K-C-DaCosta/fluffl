use super::*;

#[rustfmt::skip]
/// standard projection matrix in **row-major**
pub fn perspective(t:f32,b:f32,l:f32,r:f32,n:f32,f:f32)->Mat4<f32>{
    Mat4::new().with_data([
        [2.*n/(r-l), 0.0         ,(r+l)/(r-l) , 0.0            ],
        [0.0       , (2.*n)/(t-b),(t+b)/(t-b) , 0.0            ],
        [0.0       , 0.0         ,(f+n)/(n-f) , (2.0*f*n)/(n-f)],
        [0.0       , 0.          ,-1.         , 0.             ]
    ])
}

/// standard projection matrix in **row-major**
pub fn perspective_by_angle(deg: f32, n: f32, f: f32) -> Mat4<f32> {
    let tan = (deg * (std::f32::consts::PI / 180.0)).tan();
    let r = n / tan;
    let l = -r;
    perspective(r, l, l, r, n, f)
}

#[rustfmt::skip]
/// standard projection matrix in **row-major**
pub fn ortho(t:f32,b:f32,l:f32,r:f32,n:f32,f:f32)->Mat4<f32>{
    Mat4::new().with_data([
        [2./(r-l), 0.0       , 0.0       , (r+l)/(l-r)  ],
        [0.0       , 2./(t-b), 0.0       , (t+b)/(b-t)  ],
        [0.0       , 0.0     , 2.0/(n-f) , (f+n)/(n-f)  ],
        [0.0       , 0.      , -1.       , 1.0          ],
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
