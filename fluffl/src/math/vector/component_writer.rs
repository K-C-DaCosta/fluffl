use super::*;

/// ## Description
/// Used to write vector components into arrays
pub struct ComponentWriter<'a> {
    cursor: usize,
    data: &'a mut Vec<f32>,
}

impl<'a> ComponentWriter<'a> {
    pub fn new(data: &'a mut Vec<f32>) -> Self {
        Self { cursor: 0, data }
    }

    pub fn seek(&mut self, from_start: usize) {
        self.cursor = from_start.clamp(0, self.data.len() - 1);
    }

    pub fn write<const N: usize>(&mut self, vec: &Vector<N, f32>) {
        let data = &mut self.data;
        let cursor = &mut self.cursor;
        for k in 0..N {
            if *cursor < data.len() {
                data[*cursor] = vec[k];
            } else {
                data.push(vec[k]);
            }
            *cursor += 1;
        }
    }
}

impl<'a> From<&'a mut Vec<f32>> for ComponentWriter<'a>
{
    fn from(obj: &'a mut Vec<f32>) -> Self {
        Self::new(
            obj,
        )
    }
}

#[test]
pub fn sanity() {
    let mut list = Vec::<f32>::new();
    let mut writer = ComponentWriter::from(&mut list);
    writer.write(&Vec4::from_array([1.0f32, 0.2, 0.3, 0.4]));
    writer.write(&Vec4::from_array([2.0f32, 3.2, -0.3, 9.]));

    assert!(list
        .iter()
        .zip([1., 0.2, 0.3, 0.4, 2.0, 3.2, -0.3, 9.].iter())
        .all(|(a, b)| (b - a).abs() < 0.001));
}
