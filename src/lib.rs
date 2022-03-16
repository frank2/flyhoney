#[cfg(test)]
mod tests;

use pkbuffer::{self, Buffer, VecBuffer};
use std::path::Path;

pub mod graphics;
pub use graphics::*;

#[derive(Debug)]
pub enum Error {
    PKBufferError(pkbuffer::Error),
    NoHeader,
    TitleNotASCII,
    ChecksumComplimentMismatch,
    ROMSizeMismatch(usize,usize),
    DataLengthMismatch(usize,usize),
    InvalidColorIndex(u8),
    InvalidROMAddress(Addr24),
    InvalidDiskAddress(Addr24),
    OutOfBounds(usize,usize),
}

#[repr(packed)]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Addr24 {
    pub address: u16,
    pub bank: u8,
}
impl Addr24 {
    pub fn new(bank: u8, address: u16) -> Self {
        Self { address, bank }
    }
    pub fn from_u32(u: u32) -> Self {
        Self { address: (u & 0xFFFF) as u16, bank: ((u >> 16) & 0xFF) as u8 }
    }
    pub fn from_i32(i: i32) -> Self {
        Self { address: (i & 0xFFFF) as u16, bank: ((i >> 16) & 0xFF) as u8 }
    }
    pub fn from_offset(rom: &Rom, offset: usize) -> Self {
        Self::from_u32((offset - rom.header_size()) as u32)
    }
    pub fn as_u32(&self) -> u32 {
        let mut result = 0u32;
        result |= self.bank as u32;
        result <<= 16;
        result |= self.address as u32;
        result
    }
    pub fn as_i32(&self) -> i32 {
        self.as_u32() as i32
    }
    pub fn to_rom_address(&self) -> Result<Self, Error> {
        let mut result = self.clone();

        if let Some(new_bank) = result.bank.checked_add(0xC0) { result.bank = new_bank; Ok(result) }
        else { Err(Error::InvalidDiskAddress(*self)) }
    }
    pub fn to_disk_address(&self) -> Result<Self, Error> {
        let mut result = self.clone();

        if let Some(new_bank) = result.bank.checked_sub(0xC0) { result.bank = new_bank; Ok(result) }
        else { Err(Error::InvalidROMAddress(*self)) }
    }
    pub fn to_offset(&self, rom: &Rom) -> usize {
        if let Ok(fixed_addr) = self.to_disk_address() {
            fixed_addr.as_u32() as usize + rom.header_size()
        }
        else {
            self.as_u32() as usize + rom.header_size()
        }
    }
    pub fn is_rom_address(&self) -> bool {
        self.bank >= 0xC0
    }
    pub fn is_disk_address(&self) -> bool {
        !self.is_rom_address()
    }
}
impl std::fmt::Debug for Addr24 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unsafe { write!(f, "Addr24({:02X}:{:04X})", self.bank, self.address) }
    }
}
impl std::ops::Add<u16> for Addr24 {
    type Output = Self;

    fn add(self, rhs: u16) -> Self {
        Self::new(self.bank, self.address+rhs)
    }
}
impl std::ops::Sub<u16> for Addr24 {
    type Output = Self;

    fn sub(self, rhs: u16) -> Self {
        Self::new(self.bank, self.address-rhs)
    }
}
impl std::ops::Mul<u16> for Addr24 {
    type Output = Self;

    fn mul(self, rhs: u16) -> Self {
        Self::new(self.bank, self.address*rhs)
    }
}

#[repr(packed)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct NativeModeVectors {
    /* +4 */ cop: u16,
    /* +6 */ brk: u16,
    /* +8 */ abort: u16,
    /* +a */ nmi: u16,
    /* +c */ _padding: u16,
    /* +e */ irq: u16,
}

#[repr(packed)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct EmulationModeVectors {
    /* +4 */ cop: u16,
    /* +6 */ _padding: u16,
    /* +8 */ abort: u16,
    /* +a */ nmi: u16,
    /* +c */ res: u16,
    /* +e */ irq_or_brk: u16,
}

#[repr(packed)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct SNESHeader {
    /* +fc0 */ game_title: [u8; 21],
    /* +fd5 */ mapping_mode: u8,
    /* +fd6 */ rom_type: u8,
    /* +fd7 */ rom_size: u8,
    /* +fd8 */ sram_size: u8,
    /* +fd9 */ developer_id: u16,
    /* +fdb */ version: u8,
    /* +fdc */ checksum_compliment: u16,
    /* +fde */ checksum: u16,
    /* +fe0 */ _padding: u32,
    /* +fe4 */ native: NativeModeVectors,
    /* +ff0 */ _padding2: u32,
    /* +ff4 */ emulation: EmulationModeVectors,
}
impl SNESHeader {
    pub fn validate(&self, rom: &Rom) -> Result<(), Error> {
        for c in &self.game_title {
            if *c < 32 || *c >= 127 { return Err(Error::TitleNotASCII); }
        }

        if self.checksum_compliment.wrapping_add(self.checksum) != 0xFFFF {
            return Err(Error::ChecksumComplimentMismatch);
        }

        let rom_size = 0x400 << self.rom_size as usize;

        if rom.rom_size() > rom_size {
            return Err(Error::ROMSizeMismatch(rom_size, rom.rom_size()));
        }

        Ok(())
    }
}
    
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Rom {
    buffer: VecBuffer,
}
impl Rom {
    pub fn new<B: AsRef<[u8]>>(data: B) -> Self {
        Self { buffer: VecBuffer::from_data(data) }
    }
    pub fn from_file<P: AsRef<Path>>(filename: P) -> Result<Self, Error> {
        let buffer = match VecBuffer::from_file(filename) {
            Ok(b) => b,
            Err(e) => return Err(Error::PKBufferError(e)),
        };

        Ok(Self { buffer })
    }
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    pub fn as_ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buffer.as_mut_ptr()
    }
    pub fn as_slice(&self) -> &[u8] {
        self.buffer.as_slice()
    }
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.buffer.as_mut_slice()
    }
    pub fn offset_to_ptr(&self, offset: usize) -> Result<*const u8, Error> {
        match self.buffer.offset_to_ptr(offset) {
            Ok(p) => Ok(p),
            Err(e) => Err(Error::PKBufferError(e)),
        }
    }
    pub fn offset_to_mut_ptr(&mut self, offset: usize) -> Result<*mut u8, Error> {
        match self.buffer.offset_to_mut_ptr(offset) {
            Ok(p) => Ok(p),
            Err(e) => Err(Error::PKBufferError(e)),
        }
    }
    pub fn get_ref<T>(&self, offset: usize) -> Result<&T, Error> {
        match self.buffer.get_ref::<T>(offset) {
            Ok(r) => Ok(r),
            Err(e) => Err(Error::PKBufferError(e)),
        }
    }
    pub fn get_mut_ref<T>(&mut self, offset: usize) -> Result<&mut T, Error> {
        match self.buffer.get_mut_ref::<T>(offset) {
            Ok(r) => Ok(r),
            Err(e) => Err(Error::PKBufferError(e)),
        }
    }
    pub fn get_slice_ref<T>(&self, offset: usize, size: usize) -> Result<&[T], Error> {
        match self.buffer.get_slice_ref::<T>(offset, size) {
            Ok(r) => Ok(r),
            Err(e) => Err(Error::PKBufferError(e)),
        }
    }
    pub fn get_mut_slice_ref<T>(&mut self, offset: usize, size: usize) -> Result<&mut [T], Error> {
        match self.buffer.get_mut_slice_ref::<T>(offset, size) {
            Ok(r) => Ok(r),
            Err(e) => Err(Error::PKBufferError(e)),
        }
    }
    pub fn read(&self, offset: usize, size: usize) -> Result<&[u8], Error> {
        match self.buffer.read(offset, size) {
            Ok(d) => Ok(d),
            Err(e) => Err(Error::PKBufferError(e)),
        }
    }
    pub fn read_mut(&mut self, offset: usize, size: usize) -> Result<&mut [u8], Error> {
        match self.buffer.read_mut(offset, size) {
            Ok(d) => Ok(d),
            Err(e) => Err(Error::PKBufferError(e)),
        }
    }
    pub fn write<B: AsRef<[u8]>>(&mut self, offset: usize, data: B) -> Result<(), Error> {
        match self.buffer.write(offset, data) {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::PKBufferError(e)),
        }
    }
    pub fn write_ref<T>(&mut self, offset: usize, data: &T) -> Result<(), Error> {
        match self.buffer.write_ref::<T>(offset, data) {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::PKBufferError(e)),
        }
    }
    pub fn write_slice_ref<T>(&mut self, offset: usize, data: &[T]) -> Result<(), Error> {
        match self.buffer.write_slice_ref::<T>(offset, data) {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::PKBufferError(e)),
        }
    }
    pub fn resize(&mut self, size: usize) {
        self.buffer.resize(size, 0);
    }
    pub fn resize_blocks(&mut self, blocks: usize) {
        self.resize(blocks * 0x10000);
    }
 
    pub fn header_size(&self) -> usize {
        self.buffer.len() % 1024
    }
    pub fn rom_size(&self) -> usize {
        self.buffer.len() - self.header_size()
    }
    
    pub fn header(&self) -> Result<Buffer, Error> {
        if self.header_size() == 0 {
            return Err(Error::NoHeader);
        }

        match self.buffer.sub_buffer(0, self.header_size()) {
            Ok(b) => Ok(b),
            Err(e) => Err(Error::PKBufferError(e)),
        }
    }
    pub fn banks(&self) -> usize {
        self.rom_size() / 0x10000
    }
    pub fn get_bank(&self, bank: u8) -> Result<Buffer, Error> {
        let offset = Addr24::new(bank, 0).to_offset(self);

        match self.buffer.sub_buffer(offset, 0x10000) {
            Ok(b) => Ok(b),
            Err(e) => Err(Error::PKBufferError(e)),
        }
    }
    pub fn checksum(&self) -> u16 {
        /* this is technically incomplete, I just don't know how to handle some cases yet */
        /* TODO look up how bsnes does it, snes9x is weird */
        
        let mut checksum = 0u16;

        for byte in &self.buffer {
            checksum = checksum.wrapping_add(*byte as u16);
        }

        if self.rom_size() == 0x300000 { checksum = checksum.wrapping_add(checksum); }

        checksum
    }
    pub fn get_snes_header(&self, address: Addr24) -> Result<&SNESHeader, Error> {
        match self.buffer.get_ref::<SNESHeader>(address.to_offset(self)) {
            Ok(h) => Ok(h),
            Err(e) => Err(Error::PKBufferError(e)),
        }
    }
    pub fn get_valid_snes_header(&self, address: Addr24) -> Result<&SNESHeader, Error> {
        let header = match self.get_snes_header(address) {
            Ok(h) => h,
            Err(e) => return Err(e),
        };

        let result = header.validate(self);

        if result.is_ok() { Ok(header) }
        else { Err(result.unwrap_err()) }
    }
    pub fn get_lorom_snes_header(&self) -> Result<&SNESHeader, Error> {
        self.get_snes_header(Addr24::new(0, 0x7fc0))
    }
    pub fn get_valid_lorom_snes_header(&self) -> Result<&SNESHeader, Error> {
        self.get_valid_snes_header(Addr24::new(0, 0x7fc0))
    }
    pub fn get_hirom_snes_header(&self) -> Result<&SNESHeader, Error> {
        self.get_snes_header(Addr24::new(0, 0xffc0))
    }
    pub fn get_valid_hirom_snes_header(&self) -> Result<&SNESHeader, Error> {
        self.get_valid_snes_header(Addr24::new(0, 0xffc0))
    }
    pub fn find_valid_snes_header(&self) -> Result<&SNESHeader, Error> {
        let lo_result = self.get_valid_lorom_snes_header();

        if lo_result.is_ok() { return lo_result; }

        let hi_result = self.get_valid_hirom_snes_header();

        if hi_result.is_ok() { return hi_result; }

        lo_result
    }
}
