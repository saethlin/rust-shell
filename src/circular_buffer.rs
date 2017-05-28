extern crate std;

pub struct CircularBuffer<T> {
    buffer: Vec<T>,
    head: usize,
    tail: usize,
}

impl <T: Default> CircularBuffer<T>
    where T: std::clone::Clone {
    pub fn new(size: usize) -> Self {
        CircularBuffer {
            buffer: vec![Default::default(); size],
            head: 0,
            tail: 0,
        }
    }

    pub fn push(&mut self, entry: T) {
        self.buffer[self.tail] = entry;
        self.tail += 1;
        if self.tail > self.buffer.len() {
            self.tail = 0;
        }
        if self.head == self.tail {
            self.head += 1;
            if self.head > self.buffer.len() {
                self.head = 0;
            }
        }
    }

    pub fn iter(&self) -> CircularBufferIter<T> {
        CircularBufferIter {
            buffer: self,
            position: self.head,
        }
    }

    pub fn iter_rev(&self) -> CircularBufferIterRev<T> {
        CircularBufferIterRev {
            buffer: self,
            position: self.tail,
            exhausted: false,
        }
    }

    pub fn tail(&self) -> Option<&T> {
        if self.head == self.tail {return None}
        Some(&self.buffer[self.tail-1])
    }

    pub fn head(&self) -> Option<&T> {
        if self.head == self.tail {return None}
        Some(&self.buffer[self.head])
    }
}

pub struct CircularBufferIter<'a, T: 'a> {
    position: usize,
    buffer: &'a CircularBuffer<T>,
}

impl <'a, T: Default + 'a>Iterator for CircularBufferIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        let oldpos = self.position;
        if self.position == self.buffer.tail {
            None
        }
        else {
            self.position += 1;
            if self.position > self.buffer.buffer.len() {
                self.position = 0;
            }
            Some(&self.buffer.buffer[oldpos])
        }
    }
}

pub struct CircularBufferIterRev<'a, T: 'a> {
    position: usize,
    buffer: &'a CircularBuffer<T>,
    exhausted: bool,
}

impl <'a, T: Default + 'a>Iterator for CircularBufferIterRev<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if self.exhausted {return None}
        if self.position == self.buffer.head {
            self.exhausted = true;
        }

        // Decrement the location
        let oldpos = self.position;
        if self.position > 0 {
            self.position -= 1;
        }
        else {
            self.position = self.buffer.buffer.len()-1
        }
        Some(&self.buffer.buffer[oldpos])
    }
}
