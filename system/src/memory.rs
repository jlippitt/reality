use bitflags::Flags;
use bytemuck::Pod;
use num_traits::PrimInt;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;
use tracing::debug;

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
    DDRegisters,
    CartridgeRom,
    Pif,
}

pub trait Size: Pod + PrimInt {
    fn truncate_u32(value: u32) -> Self;
    fn truncate_u128(value: u128) -> Self;
    fn as_u8(self) -> u8;
}

impl Size for u8 {
    fn truncate_u32(value: u32) -> Self {
        value as Self
    }

    fn truncate_u128(value: u128) -> Self {
        value as Self
    }

    fn as_u8(self) -> u8 {
        self
    }
}

impl Size for u16 {
    fn truncate_u32(value: u32) -> Self {
        value as Self
    }

    fn truncate_u128(value: u128) -> Self {
        value as Self
    }

    fn as_u8(self) -> u8 {
        self as u8
    }
}

impl Size for u32 {
    fn truncate_u32(value: u32) -> Self {
        value as Self
    }

    fn truncate_u128(value: u128) -> Self {
        value as Self
    }

    fn as_u8(self) -> u8 {
        self as u8
    }
}

impl Size for u64 {
    fn truncate_u32(_value: u32) -> Self {
        panic!("64-bit read from 32-bit register");
    }

    fn truncate_u128(value: u128) -> Self {
        value as Self
    }

    fn as_u8(self) -> u8 {
        self as u8
    }
}

impl Size for u128 {
    fn truncate_u32(_value: u32) -> Self {
        panic!("128-bit read from 32-bit register");
    }

    fn truncate_u128(value: u128) -> Self {
        value as Self
    }

    fn as_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Clone, Debug)]
pub struct Memory<T: Size, U: AsRef<[T]> + AsMut<[T]> = Box<[T]>> {
    data: U,
    _phantom: PhantomData<T>,
}

impl<T: Size, U: AsRef<[T]> + AsMut<[T]>> Memory<T, U> {
    pub fn read<V: Size>(&self, address: usize) -> V {
        let index = address >> mem::size_of::<V>().ilog2();
        let slice: &[V] = bytemuck::must_cast_slice(self.data.as_ref());
        slice[index].swap_bytes()
    }

    pub fn write<V: Size>(&mut self, address: usize, value: V) {
        let index = address >> mem::size_of::<V>().ilog2();
        let slice: &mut [V] = bytemuck::must_cast_slice_mut(self.data.as_mut());
        slice[index] = value.swap_bytes();
    }

    pub fn read_unaligned<V: Size>(&self, address: usize, mirror: usize) -> V {
        let size = std::mem::size_of::<V>();
        let align_mask = size - 1;

        if (address & align_mask) == 0 {
            return self.read(address & mirror);
        }

        let mut value = V::zeroed();

        for index in 0..size {
            let byte_address = address.wrapping_add(index) & mirror;
            let shift = (index ^ align_mask) * 8;
            let byte_value = V::from(self[byte_address]).unwrap();
            value = value | (byte_value << shift);
        }

        value
    }

    pub fn write_unaligned<V: Size>(&mut self, address: usize, mirror: usize, value: V) {
        let size = std::mem::size_of::<V>();
        let align_mask = size - 1;

        if (address & align_mask) == 0 {
            return self.write(address & mirror, value);
        }

        for index in 0..size {
            let byte_address = address.wrapping_add(index) & mirror;
            let shift = (index ^ align_mask) * 8;
            let byte_value = value >> shift;
            self[byte_address] = byte_value.as_u8();
        }
    }

    pub fn read_block<V: Size>(&self, address: usize, data: &mut [V]) {
        let start = address >> mem::size_of::<V>().ilog2();
        let len = data.len();
        let slice: &[V] = bytemuck::must_cast_slice(self.data.as_ref());
        data.as_mut().copy_from_slice(&slice[start..(start + len)]);
    }

    pub fn write_block<V: Size>(&mut self, address: usize, data: &[V]) {
        let start = address >> mem::size_of::<V>().ilog2();
        let len = data.len();
        let slice: &mut [V] = bytemuck::must_cast_slice_mut(self.data.as_mut());
        slice[start..(start + len)].copy_from_slice(data.as_ref());
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::must_cast_slice(self.data.as_ref())
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        bytemuck::must_cast_slice_mut(self.data.as_mut())
    }

    pub fn len(&self) -> usize {
        self.as_bytes().len()
    }
}

impl<T: Size, U: AsRef<[T]> + AsMut<[T]> + Default> Default for Memory<T, U> {
    fn default() -> Self {
        Self {
            data: U::default(),
            _phantom: PhantomData,
        }
    }
}

impl<T: Size, U: AsRef<[T]> + AsMut<[T]>> From<U> for Memory<T, U> {
    fn from(value: U) -> Self {
        Self {
            data: value,
            _phantom: PhantomData,
        }
    }
}

impl<T: Size, U: AsRef<[T]> + AsMut<[T]>, I: SliceIndex<[u8]>> Index<I> for Memory<T, U> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.as_bytes()[index]
    }
}
impl<T: Size, U: AsRef<[T]> + AsMut<[T]>, I: SliceIndex<[u8]>> IndexMut<I> for Memory<T, U> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.as_bytes_mut()[index]
    }
}

impl<T: Size> Memory<T, Box<[T]>> {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!((bytes.len() % mem::size_of::<T>()) == 0);
        let mut vec = vec![T::zeroed(); bytes.len() / mem::size_of::<T>()];
        bytemuck::cast_slice_mut(vec.as_mut_slice()).copy_from_slice(bytes);

        Self {
            data: vec.into_boxed_slice(),
            _phantom: PhantomData,
        }
    }

    pub fn with_byte_len(len: usize) -> Self {
        assert!((len % mem::size_of::<T>()) == 0);

        Self {
            data: vec![T::zeroed(); len >> mem::size_of::<T>().ilog2()].into_boxed_slice(),
            _phantom: PhantomData,
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
            value: value.to_u32().unwrap().wrapping_shl(shift),
            mask: base_mask.wrapping_shl(shift),
        }
    }

    pub fn unmasked(value: u32) -> Self {
        Self {
            value,
            mask: u32::MAX,
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
        debug!("{}: {:?}", name, *dst);
    }

    // Convenience method for outputting debug information
    pub fn write_reg_hex<T: Copy + From<u32> + Into<u32> + Debug>(
        &self,
        name: &'static str,
        dst: &mut T,
    ) {
        self.write(dst);
        debug!("{}: {:08X?}", name, *dst);
    }

    pub fn set_or_clear<T, F>(&self, dst: &mut T, setter: F, set_bit: u32, clr_bit: u32)
    where
        F: Fn(&mut T, bool),
    {
        let set = (self.value & (1 << set_bit)) != 0;
        let clr = (self.value & (1 << clr_bit)) != 0;

        match (set, clr) {
            (false, true) => setter(dst, false),
            (true, false) => setter(dst, true),
            _ => (),
        }
    }

    pub fn set_or_clear_flag<T: Flags>(&self, dst: &mut T, flag: T, set_bit: u32, clr_bit: u32) {
        let set = (self.value & (1 << set_bit)) != 0;
        let clr = (self.value & (1 << clr_bit)) != 0;

        match (set, clr) {
            (false, true) => dst.remove(flag),
            (true, false) => dst.insert(flag),
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_read() {
        let memory = Memory::<u32>::from_bytes(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]);
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
        let memory = Memory::<u32>::from_bytes(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]);
        let mut block = [0u8; 4];
        memory.read_block(3, &mut block);
        assert_eq!([0x33, 0x44, 0x55, 0x66], block);
    }

    #[test]
    fn memory_read_write_unaligned() {
        let mut memory =
            Memory::<u32>::from_bytes(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]);

        assert_eq!(memory.read_unaligned::<u32>(1, usize::MAX), 0x11223344);
        assert_eq!(memory.read_unaligned::<u16>(5, usize::MAX), 0x5566);
        assert_eq!(memory.read_unaligned::<u8>(7, usize::MAX), 0x77);

        memory.write_unaligned::<u32>(3, usize::MAX, 0x8899aabb);
        memory.write_unaligned::<u16>(1, usize::MAX, 0xccdd);
        memory.write_unaligned::<u8>(0, usize::MAX, 0xee);

        assert_eq!(
            memory.as_bytes(),
            &[0xee, 0xcc, 0xdd, 0x88, 0x99, 0xaa, 0xbb, 0x77]
        );
    }

    #[test]
    fn memory_read_write_unaligned_mirror() {
        let mut memory =
            Memory::<u32>::from_bytes(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]);

        assert_eq!(memory.read_unaligned::<u32>(1, 3), 0x11223300);

        memory.write_unaligned::<u32>(5, 7, 0x8899aabb);

        assert_eq!(
            memory.as_bytes(),
            &[0xbb, 0x11, 0x22, 0x33, 0x44, 0x88, 0x99, 0xaa]
        );
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
