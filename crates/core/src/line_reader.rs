use heapless::Vec;

pub struct LineReader<const N: usize> {
    pub buffer: Vec<u8, N>,
    pub is_overflowed: bool
}

impl<const N: usize> LineReader<N>  {
    pub fn new() -> Self {
        LineReader { 
            buffer: Vec::<u8, N>::new(), 
            is_overflowed: false }
    }
    
    pub fn feed(&mut self, bytes: &[u8], mut on_line: impl FnMut(&[u8])) {
        for &b in bytes {
            match b {
                b'\n' => {
                    if !self.is_overflowed && !self.buffer.is_empty() {
                        on_line(&self.buffer)
                    }
                    self.buffer.clear();
                    self.is_overflowed = false;
                }
                b'\r' => { }
                _ if self.is_overflowed => { }
                _ => {
                    if self.buffer.push(b).is_err() {
                        self.is_overflowed = true;
                    }
                }
            }
        }
    }
}