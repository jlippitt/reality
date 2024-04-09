use crate::memory::Size;
use std::fmt::{self, Display};
use std::mem;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Vector(u128);

impl Vector {
    pub fn from_le_array(value: [u16; 8]) -> Self {
        Self(bytemuck::must_cast(value))
    }

    pub fn to_le_array(self) -> [u16; 8] {
        bytemuck::must_cast(self.0)
    }

    pub fn read<T: Size>(&self, el: usize) -> T {
        let size = mem::size_of::<T>();
        let shift = 128 - (((size + el) as i32) << 3);

        T::truncate_u128(if shift >= 0 {
            self.0 >> shift
        } else {
            self.0 << -shift
        })
    }

    pub fn write<T: Size>(&mut self, el: usize, value: T) {
        let size = mem::size_of::<T>();
        let shift = 128 - (((size + el) as i32) << 3);

        let (mask, value) = if shift >= 0 {
            (
                T::max_value().to_u128().unwrap() << shift,
                value.to_u128().unwrap() << shift,
            )
        } else {
            (
                T::max_value().to_u128().unwrap() >> -shift,
                value.to_u128().unwrap() >> -shift,
            )
        };

        self.0 = (self.0 & !mask) | value;
    }

    pub fn broadcast_le(&self, el: usize) -> [u16; 8] {
        let values = self.to_le_array();

        match el & 15 {
            0 | 1 => values,
            2 => [
                values[1], values[1], values[3], values[3], values[5], values[5], values[7],
                values[7],
            ],
            3 => [
                values[0], values[0], values[2], values[2], values[4], values[4], values[6],
                values[6],
            ],
            4 => [
                values[3], values[3], values[3], values[3], values[7], values[7], values[7],
                values[7],
            ],
            5 => [
                values[2], values[2], values[2], values[2], values[6], values[6], values[6],
                values[6],
            ],
            6 => [
                values[1], values[1], values[1], values[1], values[5], values[5], values[5],
                values[5],
            ],
            7 => [
                values[0], values[0], values[0], values[0], values[4], values[4], values[4],
                values[4],
            ],
            8 => [values[7]; 8],
            9 => [values[6]; 8],
            10 => [values[5]; 8],
            11 => [values[4]; 8],
            12 => [values[3]; 8],
            13 => [values[2]; 8],
            14 => [values[1]; 8],
            15 => [values[0]; 8],
            _ => unreachable!(),
        }
    }
}

impl From<u128> for Vector {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl From<Vector> for u128 {
    fn from(value: Vector) -> u128 {
        value.0
    }
}

impl Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = self.to_le_array();

        write!(
            f,
            "{:04X} {:04X} {:04X} {:04X} {:04X} {:04X} {:04X} {:04X}",
            values[7], values[6], values[5], values[4], values[3], values[2], values[1], values[0],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write() {
        let mut vec = Vector::from(0x0011_2233_4455_6677_8899_aabb_ccdd_eeff);

        assert_eq!(vec.read::<u64>(0), 0x0011_2233_4455_6677);
        assert_eq!(vec.read::<u32>(8), 0x8899_aabb);
        assert_eq!(vec.read::<u16>(12), 0xccdd);
        assert_eq!(vec.read::<u8>(14), 0xee);

        vec.write::<u64>(8, 0x0011_2233_4455_6677);
        vec.write::<u32>(4, 0x8899_aabb);
        vec.write::<u16>(2, 0xccdd);
        vec.write::<u8>(1, 0xee);

        assert_eq!(u128::from(vec), 0x00ee_ccdd_8899_aabb_0011_2233_4455_6677);
    }

    #[test]
    fn read_write_end() {
        let mut vec = Vector::from(0x0011_2233_4455_6677_8899_aabb_ccdd_eeff);

        assert_eq!(vec.read::<u64>(12), 0xccdd_eeff_0000_0000);

        vec.write::<u64>(12, 0x0011_2233_4455_6677);

        assert_eq!(u128::from(vec), 0x0011_2233_4455_6677_8899_aabb_0011_2233);
    }
}
