use crate::cpu::Size;
use bytemuck::Pod;
use std::mem;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Mapping {
    None,
    RdramData,
    RdramRegister,
    Rsp,
    RdpCommand,
    RdpSpan,
    MipsInterface,
    VideoInterface,
    AudioInterface,
    PeripheralInterface,
    RdramInterface,
    SerialInterface,
    CartridgeRom,
    Pif,
}

pub struct Memory {
    vec: Vec<u32>,
}

impl Memory {
    pub fn new(len: usize) -> Self {
        assert!((len & 3) == 0);
        Self {
            vec: vec![0; len >> 2],
        }
    }

    pub fn read<T: Pod>(&self, address: u32) -> T {
        let mem_size = mem::size_of::<u32>();
        let data_size = mem::size_of::<T>();
        debug_assert!((mem_size % data_size) == 0);
        let index = address as usize >> data_size.ilog2();
        let slice: &[T] = bytemuck::must_cast_slice(&self.vec);
        slice[index ^ ((mem_size / data_size) - 1)]
    }

    pub fn write<T: Pod>(&mut self, address: u32, value: T) {
        let mem_size = mem::size_of::<u32>();
        let data_size = mem::size_of::<T>();
        debug_assert!((mem_size % data_size) == 0);
        let index = address as usize >> data_size.ilog2();
        let slice: &mut [T] = bytemuck::must_cast_slice_mut(&mut self.vec);
        slice[index ^ ((mem_size / data_size) - 1)] = value;
    }
}

impl From<Vec<u8>> for Memory {
    fn from(value: Vec<u8>) -> Self {
        let vec = value
            .chunks_exact(4)
            .map(|chunks| u32::from_be_bytes([chunks[0], chunks[1], chunks[2], chunks[3]]))
            .collect();

        Self { vec }
    }
}

#[derive(Clone, Debug)]
pub struct WriteMask {
    value: u32,
    mask: u32,
}

impl WriteMask {
    pub fn new<T: Size>(address: u32, value: T) -> Self {
        let data_size = mem::size_of::<T>() as u32;
        let base_mask = 0xffff_ffffu32.wrapping_shr(32 - (data_size << 3));
        let shift = ((address & 3) ^ (4 - data_size)) << 3;

        Self {
            value: value.to_u32().wrapping_shl(shift),
            mask: base_mask.wrapping_shl(shift),
        }
    }

    pub fn rotate(self, bits: u32) -> Self {
        Self {
            value: self.value.rotate_right(bits),
            mask: self.mask.rotate_right(bits),
        }
    }

    pub fn raw(&self) -> u32 {
        self.value
    }

    pub fn write<T: Copy + From<u32> + Into<u32>>(&self, dst: &mut T) {
        *dst = T::from(((*dst).into() & !self.mask) | (self.value & self.mask));
    }

    pub fn write_partial<T: Copy + From<u32> + Into<u32>>(&self, dst: &mut T, partial_mask: u32) {
        let mask = self.mask & partial_mask;
        *dst = T::from(((*dst).into() & !mask) | (self.value & mask));
    }

    pub fn set_or_clear<T, F>(&self, dst: &mut T, setter: F, set_bit: u32, clr_bit: u32)
    where
        F: Fn(&mut T, bool),
    {
        let set = (self.value & (1 << set_bit)) != 0;
        let clr = (self.value & (1 << clr_bit)) != 0;

        match (set, clr) {
            (false, false) => (),
            (false, true) => setter(dst, false),
            (true, false) => setter(dst, true),
            (true, true) => panic!(
                "Conflict between SET_* and CLR_* bits {} and {}",
                set_bit, clr_bit
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_mask_u8() {
        let mut dst = 0x00112233u32;
        let mask = WriteMask::new(0, 0x44u8);
        mask.write(&mut dst);
        assert_eq!(dst, 0x44112233);
        let mask = WriteMask::new(1, 0x55u8);
        mask.write(&mut dst);
        assert_eq!(dst, 0x44552233);
        let mask = WriteMask::new(2, 0x66u8);
        mask.write(&mut dst);
        assert_eq!(dst, 0x44556633);
        let mask = WriteMask::new(3, 0x77u8);
        mask.write(&mut dst);
        assert_eq!(dst, 0x44556677);
    }

    #[test]
    fn write_mask_u16() {
        let mut dst = 0x00112233u32;
        let mask = WriteMask::new(0, 0x4455u16);
        mask.write(&mut dst);
        assert_eq!(dst, 0x44552233u32);
        let mask = WriteMask::new(2, 0x6677u16);
        mask.write(&mut dst);
        assert_eq!(dst, 0x44556677u32);
    }

    #[test]
    fn write_mask_u32() {
        let mut dst = 0x00112233u32;
        let mask = WriteMask::new(0, 0x44556677u32);
        mask.write(&mut dst);
        assert_eq!(dst, 0x44556677u32);
    }
}
