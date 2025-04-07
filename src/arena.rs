#[derive(Default)]
pub struct Arena<T> {
    data: Vec<T>,
}

impl<T> Arena<T> {
    pub fn allocate(&mut self, value: T) -> usize {
        self.data.push(value);
        self.data.len() - 1 // Return the index of the newly allocated object
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }
}
