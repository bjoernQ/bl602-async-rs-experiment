// A clumsy container.
// Replace this with something better.
// Maybe use heapless Vec under the hood but we cannot use heapless
// currently since we don't have atomics and atomic-polyfill doesn't support RISCV without atomics.
pub struct Container<T> {
    data: [Option<T>; 4],
}

impl<T> Container<T> {
    pub const fn new() -> Container<T> {
        Container {
            data: [None, None, None, None],
        }
    }

    pub fn push(&mut self, v: T) -> usize {
        let i = {
            let mut r = 0;
            for i in 0..self.data.len() {
                if self.data[i].is_none() {
                    r = i;
                    break;
                }
            }

            r
        };

        self.data[i] = Some(v);
        i
    }

    /// Count of the non-None elements
    pub fn size(&self) -> usize {
        let mut r = 0;
        for i in 0..self.data.len() {
            if self.data[i].is_some() {
                r += 1;
            }
        }
        r
    }

    /// Size of the underlying container
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get(&mut self, i: usize) -> Option<&mut T> {
        self.data[i].as_mut()
    }

    pub fn remove(&mut self, i: usize) {
        self.data[i] = None;
    }

    /// not the *real* iterator since it doesn't support the needed lifetime annotations
    pub fn iter<'a>(&'a mut self) -> ContainerIterator<'a, T> {
        ContainerIterator::<'a> {
            index: 0,
            container: self,
        }
    }
}

pub struct ContainerIterator<'a, T> {
    index: usize,
    container: &'a mut Container<T>,
}

impl<'a, T> ContainerIterator<'a, T> {
    pub fn next(&mut self) -> (usize, Option<&mut T>) {
        if self.index < self.container.size() {
            while self.container.get(self.index).is_none() && self.index < self.container.len() {
                self.index += 1;
            }

            match self.container.get(self.index) {
                Some(item) => {
                    let i = self.index;
                    let r = Some(item);
                    self.index += 1;
                    (i, r)
                }
                None => (0, None),
            }
        } else {
            (0, None)
        }
    }
}
