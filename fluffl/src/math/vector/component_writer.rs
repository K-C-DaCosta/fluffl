use super::*;

/// ## Description
/// Used to write vector components into arrays
pub struct ComponentWriter<'a, T, Len, Push> {
    cursor: usize,
    data: &'a mut T,
    len_cb: Len,
    push_cb: Option<Push>,
}

impl<'a, T, M, Len, Push> ComponentWriter<'a, T, Len, Push>
where
    Push: FnMut(&mut T, M) + Copy,
    Len: Fn(&T) -> usize + Copy,
    T: Index<usize, Output = M> + IndexMut<usize, Output = M>,
    M: Copy + Default,
{
    pub fn new(data: &'a mut T, len_cb: Len, push_cb: Option<Push>) -> Self {
        Self {
            cursor: 0,
            data,
            len_cb,
            push_cb,
        }
    }

    pub fn seek(&mut self, from_start: usize) {
        let len = self.len_cb;
        self.cursor = from_start.clamp(0, len(self.data));
    }

    pub fn write<const N: usize>(&mut self, vec: &Vector<N, M>) -> usize {
        match self.push_cb {
            Some(push) => self.write_pushable(push, vec),
            None => self.write_cant_push(vec),
        }
    }

    fn write_cant_push<const N: usize>(&mut self, vec: &Vector<N, M>) -> usize {
        let len = self.len_cb;
        let mut k = 0;
        while (k < N) && self.cursor < len(self.data) {
            self.data[self.cursor] = vec[k];
            self.cursor += 1;
            k += 1;
        }
        k
    }

    fn write_pushable<const N: usize>(&mut self, mut push: Push, vec: &Vector<N, M>) -> usize {
        let len = self.len_cb;
        for k in 0..N {
            if self.cursor >= len(self.data) {
                push(self.data, M::default())
            }
            self.data[self.cursor] = vec[k];
            self.cursor += 1;
        }
        N
    }
    pub fn done(self) {}
}

impl<'a, T> From<&'a mut Vec<T>>
    for ComponentWriter<'a, Vec<T>, fn(&Vec<T>) -> usize, fn(&mut Vec<T>, T)>
where
    T: Copy + Default,
{
    fn from(obj: &'a mut Vec<T>) -> Self {
        Self::new(
            obj,
            |vec: &Vec<_>| vec.len(),
            Some(|vec, item| vec.push(item)),
        )
    }
}

impl<'a, T> From<&'a mut [T]> for ComponentWriter<'a, [T], fn(&[T]) -> usize, fn(&mut [T], T)>
where
    [T]: Sized,
    T: Copy + Default,
{
    fn from(obj: &'a mut [T]) -> Self {
        Self::new(obj, |vec: &[_]| vec.len(), None)
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
