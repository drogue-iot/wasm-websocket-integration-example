use num_traits::{Num, FromPrimitive};

pub struct MovingAverage<T> {
    values: Vec<T>,
    position: usize,
}


impl<T: FromPrimitive + Copy + Num> MovingAverage<T> {

    pub fn new (size: usize) -> Self {

        MovingAverage {
            values: Vec::with_capacity(size),
            position: 0,
        }
    }

    pub fn add(&mut self, new_val: T) -> T {

        self.values.insert(self.position, new_val);
        self.position = ( self.position + 1) % self.values.len();

        self.average()
    }

    pub fn last(&self) -> T {
        if !self.values.is_empty() {
            self.values[self.position]
        } else {
            T::zero()
        }
    }

    fn average(&self) -> T {

        self.values.iter().fold(T::zero(), |sum,  &x| sum + x ) / T::from_usize(self.values.len()).unwrap()
    }
}