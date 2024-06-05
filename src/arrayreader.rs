use std::io::{self, Read};

pub struct ArrayReader<'a> {
    array: &'a [u8],
    current_index: usize,
}
impl<'a> ArrayReader<'a> {
    pub fn new(array: &'a [u8]) -> Self {
        Self {
            array,
            current_index: 0,
        }
    }
}
impl<'a> Read for ArrayReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let max_read = buf
            .len()
            .min(self.array.len().saturating_sub(self.current_index));
        if max_read > 0 {}
        let dst = &mut buf[..max_read];
        let src = &self.array[self.current_index..self.current_index + max_read];
        dst.copy_from_slice(src);
        Ok(max_read)
    }
}
