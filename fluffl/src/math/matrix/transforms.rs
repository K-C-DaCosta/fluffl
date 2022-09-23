use super::*;

#[rustfmt::skip]
pub fn translate4<T>(translate:Vec4<T>)->Mat4<T>
where T:HasConstants+Default+Copy,
{
    Mat4::new().with_data([
        [T::one() , T::zero(),T::zero(),translate[0]],
        [T::zero(), T::one() ,T::zero(),translate[1]],  
        [T::zero(), T::zero(),T::one() ,translate[2]],
        [T::zero(), T::zero(),T::zero(),T::one()    ]
    ])
}

#[rustfmt::skip]
pub fn scale4<T>(scale:Vec4<T>)->Mat4<T>
where T:HasConstants+Default+Copy,
{
    
    Mat4::new().with_data([
        [scale[0] , T::zero(),T::zero(),T::zero()],
        [T::zero(), scale[1] ,T::zero(),T::zero()],
        [T::zero(), T::zero(),scale[2] ,T::zero()],
        [T::zero(), T::zero(),T::zero(),T::one()],
    ])
}

#[rustfmt::skip]
pub fn rotate_z<T>(rad:T)->Mat4<T>
where T:HasConstants+Default+Copy + HasTrig + Neg<Output=T>,
{
    let cos = rad.cos(); 
    let sin = rad.sin();
    Mat4::new().with_data([
        [cos      , -sin     ,T::zero(),T::zero()],
        [sin      ,  cos     ,T::zero(),T::zero()],
        [T::zero(), T::zero(),T::one() ,T::zero()],
        [T::zero(), T::zero(),T::zero(),T::one() ],
    ])
}

#[rustfmt::skip]
pub fn rotate_x<T>(rad:T)->Mat4<T>
where T:HasConstants+Default+Copy + HasTrig + Neg<Output=T>,
{
    let cos = rad.cos(); 
    let sin = rad.sin();
    Mat4::new().with_data([
        [T::one() ,  T::zero(),T::zero(),T::zero()],
        [T::zero(),  cos      , -sin    ,T::zero()],
        [T::zero(),  sin      , cos     ,T::zero()],
        [T::zero(), T::zero() ,T::zero(),T::one() ],
    ])
}



