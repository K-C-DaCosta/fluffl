#[allow(unused_imports)]
use super::*; 


#[test]
fn multiplication_mat_by_mat_and_mat_by_vec() {
    #[rustfmt::skip]
    let test_mat = ||{
        Mat4::<f32>::new().with_data([
            [0. , 1. , 2. , 3. ],
            [4. , 5. , 6. , 7. ],
            [8. , 9. , 10., 11.],
            [12., 13., 14., 15.],
        ])
    };

    #[rustfmt::skip]
    let test_mat_squared = ||{
        Mat4::<f32>::new().with_data([
            [ 56. , 62. , 68. ,74. ],
            [ 152., 174., 196.,218.],
            [ 248., 286., 324.,362.],
            [ 344., 398., 452.,506.],
        ])
    };

    const TOLERANCE: f32 = 0.0001;
    let a = test_mat();
    let product = a * a;

    assert_eq!(
        false,
        product.is_similar(&a, TOLERANCE),
        "test_mat and product should be very different"
    );

    assert_eq!(
        true,
        product.is_similar(&test_mat_squared(), TOLERANCE),
        "product should be exactly the same as test_mat_squared"
    );

    let column_1 = product * Vec4::from([1.0, 0.0, 0.0, 0.]);
    assert_eq!(
        true,
        (column_1 - Vec4::from([56., 152., 248., 344.])).length_squared() < TOLERANCE,
        "product*<1,0,0,0> should be equal to the first column of product"
    );

    let column_3 = product * Vec4::from([0.0, 0.0, 0.0, 1.]);
    assert_eq!(
        true,
        (column_3 - Vec4::from([74., 218., 362., 506.])).length_squared() < TOLERANCE,
        "product*<0,0,0,1> should be equal to the last column of product"
    );
}

#[test]
#[allow(non_snake_case)]
fn transpose_test() {
    #[rustfmt::skip]
    let test_mat = ||{
        Mat4::<f32>::new().with_data([
            [0. , 1. , 2. , 3. ],
            [4. , 5. , 6. , 7. ],
            [8. , 9. , 10., 11.],
            [12., 13., 14., 15.],
        ])
    };
    let A = test_mat();
    let A_T = A.transpose();
    assert_eq!(
        true,
        A.is_similar(&A_T.transpose(), 0.0001),
        "  (A^T)^T should be  A "
    );

    assert_eq!(
        false,
        A.is_similar(&A_T, 0.0001),
        "  A != A^T in general "
    );
}