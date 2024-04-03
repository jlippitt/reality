use bytemuck::Pod;
use std::fmt::Debug;
use std::mem;
use tracing::trace;

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

pub trait Size: Pod {
    fn from_u32(value: u32) -> Self;
    fn to_u32(self) -> u32;
}

impl Size for u8 {
    fn from_u32(value: u32) -> Self {
        value as Self
    }

    fn to_u32(self) -> u32 {
        self as u32
    }
}

impl Size for u16 {
    fn from_u32(value: u32) -> Self {
        value as Self
    }

    fn to_u32(self) -> u32 {
        self as u32
    }
}

impl Size for u32 {
    fn from_u32(value: u32) -> Self {
        value as Self
    }

    fn to_u32(self) -> u32 {
        self
    }
}

#[derive(Clone, Debug)]
pub struct Memory<T: AsRef<[u32]> + AsMut<[u32]> = Vec<u32>> {
    data: T,
}

impl<T: AsRef<[u32]> + AsMut<[u32]>> Memory<T> {
    pub fn as_slice(&self) -> &[u32] {
        self.data.as_ref()
    }

    pub fn as_mut_slice(&mut self) -> &mut [u32] {
        self.data.as_mut()
    }

    pub fn read<U: Size>(&self, address: u32) -> U {
        let mem_size = mem::size_of::<u32>();
        let data_size = mem::size_of::<U>();
        debug_assert!((mem_size % data_size) == 0);
        let index = address as usize >> data_size.ilog2();
        let slice: &[U] = bytemuck::must_cast_slice(self.data.as_ref());
        slice[index ^ ((mem_size / data_size) - 1)]
    }

    pub fn write<U: Size>(&mut self, address: u32, value: U) {
        let mem_size = mem::size_of::<u32>();
        let data_size = mem::size_of::<U>();
        debug_assert!((mem_size % data_size) == 0);
        let index = address as usize >> data_size.ilog2();
        let slice: &mut [U] = bytemuck::must_cast_slice_mut(self.data.as_mut());
        slice[index ^ ((mem_size / data_size) - 1)] = value;
    }

    pub fn read_block(&self, address: u32, data: &mut [u32]) {
        assert!((address & 3) == 0);
        let index = (address >> 2) as usize;
        data.copy_from_slice(&self.data.as_ref()[index..(index + data.len())]);
    }

    pub fn write_block(&mut self, address: u32, data: &[u32]) {
        assert!((address & 3) == 0);
        let index = (address >> 2) as usize;
        self.data.as_mut()[index..(index + data.len())].copy_from_slice(data);
    }
}

impl<T: AsRef<[u32]> + AsMut<[u32]> + Default> Default for Memory<T> {
    fn default() -> Self {
        Self { data: T::default() }
    }
}

impl<T: AsRef<[u32]> + AsMut<[u32]>> From<T> for Memory<T> {
    fn from(value: T) -> Self {
        Self { data: value }
    }
}

impl Memory<Vec<u32>> {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!((bytes.len() & 3) == 0);
        let data = bytes
            .chunks_exact(4)
            .map(|chunks| u32::from_be_bytes([chunks[0], chunks[1], chunks[2], chunks[3]]))
            .collect();

        Self { data }
    }

    pub fn with_byte_len(len: usize) -> Self {
        assert!((len & 3) == 0);
        Self {
            data: vec![0; len >> 2],
        }
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

    // Convenience method for outputting debug information
    pub fn write_reg<T: Copy + From<u32> + Into<u32> + Debug>(
        &self,
        name: &'static str,
        dst: &mut T,
    ) {
        self.write(dst);
        trace!("{}: {:?}", name, *dst);
    }

    // Convenience method for outputting debug information
    pub fn write_reg_hex<T: Copy + From<u32> + Into<u32> + Debug>(
        &self,
        name: &'static str,
        dst: &mut T,
    ) {
        self.write(dst);
        trace!("{}: {:08X?}", name, *dst);
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
    fn memory_read() {
        let memory = Memory::from_bytes(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]);
        assert_eq!(0x00112233, memory.read::<u32>(0));
        assert_eq!(0x44556677, memory.read::<u32>(4));
        assert_eq!(0x0011, memory.read::<u16>(0));
        assert_eq!(0x2233, memory.read::<u16>(2));
        assert_eq!(0x4455, memory.read::<u16>(4));
        assert_eq!(0x6677, memory.read::<u16>(6));
        assert_eq!(0x00, memory.read::<u8>(0));
        assert_eq!(0x11, memory.read::<u8>(1));
        assert_eq!(0x22, memory.read::<u8>(2));
        assert_eq!(0x33, memory.read::<u8>(3));
        assert_eq!(0x44, memory.read::<u8>(4));
        assert_eq!(0x55, memory.read::<u8>(5));
        assert_eq!(0x66, memory.read::<u8>(6));
        assert_eq!(0x77, memory.read::<u8>(7));
    }

    #[test]
    fn memory_read_block() {
        let memory = Memory::from_bytes(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]);
        let mut block = [0u32; 2];
        memory.read_block(0, &mut block);
        assert_eq!([0x00112233, 0x44556677], block);
    }

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
