
#[derive(Copy, Clone)]
pub struct PointInterceptLine {
    pub m: f32,
    pub b: f32,
}

#[allow(dead_code)]
pub fn translate(dx: f32, dy: f32) -> [f32; 16] {
    [
        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., dx, dy, 0., 1.,
    ]
}

///computes a matrix that resizes a region of `src` space  to `dst` space
pub fn resize_region(src:AABB,dst:AABB)->[f32;16]{
    let scale_x = dst.w/src.w;
    let scale_y = dst.h/src.h;
    [ 
        scale_x,     0., 0.,-src.x*scale_x + dst.x,
        0.     ,scale_y, 0.,-src.y*scale_y + dst.y,
        0.     ,     0., 1.,          0.          ,
        0.     ,     0., 0.,          1.
    ]
}

pub fn map_coefs(l: f32, u: f32, a: f32, b: f32) -> PointInterceptLine {
    let m = (b - a) / (u - l);
    let b = a - m * l;
    PointInterceptLine { m, b }
}

#[allow(dead_code)]
pub fn eval_line(coefs: PointInterceptLine, x: f32) -> f32 {
    let PointInterceptLine { m, b } = coefs;
    m * x + b
}
#[derive(Copy, Clone)]
pub struct AABB {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl AABB {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
}