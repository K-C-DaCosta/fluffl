pub trait HasTrig: Copy {
    fn sin(self) -> Self;
    fn cos(self) -> Self;
}

#[rustfmt::skip]
impl HasTrig for f32{
    fn cos(self) -> Self { f32::cos(self) }
    fn sin(self) -> Self { f32::sin(self) }
}

#[rustfmt::skip]
impl HasTrig for f64{
    fn cos(self) -> Self { f64::cos(self) }
    fn sin(self) -> Self { f64::sin(self) }
}
