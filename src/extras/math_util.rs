use std::ops;

pub type Vec2 = [f32; 2];

/// # Description
/// computes dot product
/// # Comments
/// -loop should get optimized away in --release
pub fn dot(a: Vec2, b: Vec2) -> f32 {
    let mut sum = 0.;
    for i in 0..2 {
        sum += a[i] * b[i]
    }
    sum
}

pub fn compute_bounding_box_from_points_2d(points: &[Vec2]) -> AABB {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for point in points.iter() {
        min_x = min_x.min(point[0]);
        max_x = max_x.max(point[0]);

        min_y = min_y.min(point[1]);
        max_y = max_y.max(point[1]);
    }

    AABB {
        x: min_x,
        y: min_y,
        w: max_x - min_x,
        h: max_y - min_y,
    }
}

/// # Description
/// Computes orthographic projection matrix that maps clip/ndc/(whatever you call it) to screen space.
/// # Comments
/// - Its been a while since I wrote this function but I think this returns a column-major
///  style matrix, so no transpose is needed here
pub fn calc_proj(w: f32, h: f32) -> [f32; 16] {
    let PointInterceptLine { m: m1, b: b1 } = map_coefs(0., w, -1., 1.);
    let PointInterceptLine { m: m2, b: b2 } = map_coefs(0., h, 1., -1.);
    [
        m1, 0., 0., 0., 0., m2, 0., 0., 0., 0., 1., 0., b1, b2, 0., 1.,
    ]
}
/// # Description
/// A POD struct that represents the point intercept form of a line (y=mx+b)
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
/// # Description 
/// computes local coordinate frame from line-segment
/// # Returns 
/// - returns (`world_to_local_matrix`,`rectangle_extents`,`half_width`) 
pub fn compute_world_to_local_from_segment(
    a: Vec2,
    b: Vec2,
    height: f32,
) -> ([f32; 9], [Vec2; 4], f32) {
    //compute local coordinates frame from line se
    let mut p = [b[0] - a[0], b[1] - a[1]];
    let mut q = [p[1], -p[0]];
    let o = [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];

    let mag = (p[0] * p[0] + p[1] * p[1]).sqrt();
    let inv_mag = 1.0 / mag;
    p[0] *= inv_mag;
    p[1] *= inv_mag;
    q[0] *= inv_mag;
    q[1] *= inv_mag;

    let rectangle_points = [
        [p[0] * mag + o[0], p[1] * mag + o[1]],
        [p[0] * (-mag) + o[0], p[1] * (-mag) + o[1]],
        [q[0] * height + o[0], q[1] * height + o[1]],
        [q[0] * (-height) + o[0], q[1] * (-height) + o[1]],
    ];

    (compute_world_to_local(p, q, o), rectangle_points, mag*0.5)
}

/// # Description
/// computes a 3x3 homo matrix that goes from world-space to localspace
/// # Comments
/// - p and q **MUST** be orthoganal, otherwise this method wont work.
/// - The matrix is columns wise so no transpose needed
pub fn compute_world_to_local(p: Vec2, q: Vec2, o: Vec2) -> [f32; 9] {
    [p[0], q[0], 0., p[1], q[1], 0., -dot(p, o), -dot(q, o), 1.0]
}

///computes a matrix that resizes a region of `src` space  to `dst` space
pub fn resize_region(src: AABB, dst: AABB) -> [f32; 16] {
    let scale_x = dst.w / src.w;
    let scale_y = dst.h / src.h;
    [
        scale_x,
        0.,
        0.,
        -src.x * scale_x + dst.x,
        0.,
        scale_y,
        0.,
        -src.y * scale_y + dst.y,
        0.,
        0.,
        1.,
        0.,
        0.,
        0.,
        0.,
        1.,
    ]
}

/// # Description
/// Generates a point intercept line that maps values in the range: \[l,u\] -> \[a,b\]
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
    pub fn get_top_left(&self)->[f32;2]{
        [self.x,self.y]
    }
    pub fn get_dims(&self)->[f32;2]{
        [self.w,self.h]
    }
}
