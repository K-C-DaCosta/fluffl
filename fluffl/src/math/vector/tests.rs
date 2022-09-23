#[allow(unused_imports)]
use super::*; 

#[test]
#[allow(unused_imports)]
pub fn color_test() {
    let color = Vec4::<f32>::rgba_u32(0xffffffff);

    assert_eq!(
        true,
        color.iter().all(|&c| (1.0 - c).abs() < 0.001),
        "all the elements should equal 1.0"
    );
}

#[test]
pub fn position_test() {
    let a = Vec2::<f32>::from_array([0.1, 0.2]);
    let b = Vec4::to_pos(a);

    assert_eq!(
        true,
        b.iter()
            .zip([0.1, 0.2, 0.0, 1.0])
            .all(|(a, b)| (b - a).abs() < 0.001),
        "Vector::to_pos(..) broken, should be [0.1,0.2,0.0,1.0]"
    );

    let a = Vec3::<f32>::from_array([3., 5., 9.5]);
    let b = Vec4::to_pos(a);

    assert_eq!(
        true,
        b.iter()
            .zip([3., 5., 9.5, 1.0])
            .all(|(a, b)| (b - a).abs() < 0.001),
        "Vector::to_pos(..) broken, should be [3.,5.,9.5,1.0]"
    );
}
