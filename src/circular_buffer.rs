extern crate std;

struct CircularBuffer<T> {
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
        self.tail += 1;
        if self.tail > self.buffer.len() {
            self.tail = 0;
        }
        self.buffer[self.tail] = entry;
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

    pub fn tail(&self) -> Option<&T> {
        if self.head == self.tail {
            ()
        }
        Some(&self.buffer[self.tail])
    }

    pub fn head(&self) -> Option<&T> {
        if self.head == self.tail {
            ()
        }
        Some(&self.buffer[self.head])
    }
}

struct CircularBufferIter<'a, T: 'a> {
    position: usize,
    buffer: &'a CircularBuffer<T>,
}

impl <'a, T: Default + 'a>Iterator for CircularBufferIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if self.position == self.buffer.tail {
            None
        }
        else {
            self.position += 1;
            if self.position > self.buffer.buffer.len() {
                self.position = 0;
            }
            Some(&self.buffer.buffer[self.position])
        }
    }
}