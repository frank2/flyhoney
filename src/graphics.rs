use crate::Error;
use std::convert::{TryFrom, TryInto};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Rgb888(pub u32);
impl Rgb888 {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        let mut result = Self(0);
        result.set_red(r);
        result.set_green(g);
        result.set_blue(b);
        result
    }
    pub fn get_red(&self) -> u8 {
        ((self.0 >> 16) & 0xFF) as u8
    }
    pub fn set_red(&mut self, r: u8) {
        self.0 = (self.0 & 0xFFFF) | ((r as u32) << 16)
    }
    pub fn get_green(&self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }
    pub fn set_green(&mut self, g: u8) {
        self.0 = (self.0 & 0xFF00FF) | ((g as u32) << 8)
    }
    
    pub fn get_blue(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }
    pub fn set_blue(&mut self, b: u8) {
        self.0 = (self.0 & 0xFFFF00) | (b as u32)
    }
    pub fn as_bgr555(&self) -> Bgr555 {
        (*self).into()
    }
}
impl From<u32> for Rgb888 {
    fn from(data: u32) -> Self {
        Self(data)
    }
}
impl From<Bgr555> for Rgb888 {
    fn from(data: Bgr555) -> Self {
        let mut result = Self(0);
        result.set_red(data.get_red() << 3);
        result.set_green(data.get_green() << 3);
        result.set_blue(data.get_blue() << 3);
        result
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Bgr555(pub u16);
impl Bgr555 {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        let mut result = Self(0);
        result.set_red(r);
        result.set_green(g);
        result.set_blue(b);
        result
    }
    pub fn get_blue(&self) -> u8 {
        ((self.0 >> 10) & 0x1F) as u8
    }
    pub fn set_blue(&mut self, value: u8) {
        self.0 = (self.0 & 0x3FF) | ((value as u16 & 0x1F) << 10);
    }
    pub fn get_green(&self) -> u8 {
        ((self.0 >> 5) & 0x1F) as u8
    }
    pub fn set_green(&mut self, value: u8) {
        self.0 = (self.0 & 0x7C1F) | ((value as u16 & 0x1F) << 5);
    }
    pub fn get_red(&self) -> u8 {
        (self.0 & 0x1F) as u8
    }
    pub fn set_red(&mut self, value: u8) {
        self.0 = (self.0 & 0x7FE0) | (value as u16 & 0x1F);
    }
    pub fn as_rgb888(&self) -> Rgb888 {
        (*self).into()
    }
}
impl From<u16> for Bgr555 {
    fn from(data: u16) -> Self {
        Self(data)
    }
}
impl From<Rgb888> for Bgr555 {
    fn from(data: Rgb888) -> Self {
        let mut result = Self(0);
        result.set_red(data.get_red() >> 3);
        result.set_green(data.get_green() >> 3);
        result.set_blue(data.get_blue() >> 3);
        result
    }
}

pub trait SNESPalette: Sized {
    fn from_data<B: AsRef<[u8]>>(data: B) -> Result<Self, Error>;
    fn set_index(&mut self, index: u8, color: Bgr555) -> Result<(), Error>;
    fn get_index(&self, index: u8) -> Result<Bgr555, Error>;
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct SNESPalette16([Bgr555; 16]);
impl SNESPalette for SNESPalette16 {
    fn from_data<B: AsRef<[u8]>>(data: B) -> Result<Self, Error> {
        let buf = data.as_ref();
        if buf.len() != 16*2 { return Err(Error::DataLengthMismatch(buf.len(),16*2)); }

        let mut array = [Bgr555(0); 16];

        for i in (0..buf.len()).step_by(2) {
            let mut value = 0u16;
            value |= buf[i] as u16;
            value |= (buf[i+1] as u16) << 8;

            array[i/2] = Bgr555(value);
        }

        Ok(Self(array))
    }
    fn set_index(&mut self, index: u8, color: Bgr555) -> Result<(), Error> {
        if index >= 16 { return Err(Error::InvalidColorIndex(index)); }

        self.0[index as usize] = color;
        Ok(())
    }
    fn get_index(&self, index: u8) -> Result<Bgr555, Error> {
        if index >= 16 { return Err(Error::InvalidColorIndex(index)); }

        Ok(self.0[index as usize])
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct SNESPalette256([Bgr555; 256]);
impl SNESPalette for SNESPalette256 {
    fn from_data<B: AsRef<[u8]>>(data: B) -> Result<Self, Error> {
        let buf = data.as_ref();
        if buf.len() != 256*2 { return Err(Error::DataLengthMismatch(buf.len(),256*2)); }

        let mut array = [Bgr555(0); 256];

        for i in (0..buf.len()).step_by(2) {
            let mut value = 0u16;
            value |= buf[i] as u16;
            value |= (buf[i+1] as u16) << 8;

            array[i/2] = Bgr555(value);
        }

        Ok(Self(array))
    }
    fn set_index(&mut self, index: u8, color: Bgr555) -> Result<(), Error> {
        self.0[index as usize] = color;
        Ok(())
    }
    fn get_index(&self, index: u8) -> Result<Bgr555, Error> {
        Ok(self.0[index as usize])
    }
}

pub trait SNESTile: Sized {
    fn new() -> Self;
    fn from_data<B: AsRef<[u8]>>(data: B) -> Result<Self, Error>;
    fn set_value(&mut self, x: usize, y: usize, value: u8) -> Result<(), Error>;
    fn get_value(&self, x: usize, y: usize) -> Result<u8, Error>;
    fn to_colormap(&self) -> Result<Vec<u8>, Error> {
        let mut result = Vec::<u8>::new();

        for y in 0..8 {
            for x in 0..8 {
                match self.get_value(x,y) {
                    Ok(v) => result.push(v),
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(result)
    }
    fn from_colormap<B: AsRef<[u8]>>(colormap: B) -> Result<Self, Error> {
        let mut result = Self::new();
        let data = colormap.as_ref();

        for i in 0..data.len() {
            let x = i % 8;
            let y = i / 8;
            let value = data[i];

            match result.set_value(x,y,value) {
                Ok(()) => (),
                Err(e) => return Err(e),
            }
        }

        Ok(result)
    }
    fn to_bgr555<T: SNESPalette>(&self, palette: &T) -> Result<Vec<Bgr555>, Error> {
        let colormap = match self.to_colormap() {
            Ok(c) => c,
            Err(e) => return Err(e),
        };
        let mut result = Vec::<Bgr555>::new();

        for c in &colormap {
            let color = match palette.get_index(*c) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            result.push(color);
        }

        Ok(result)
    }
    fn to_rgb888<T: SNESPalette>(&self, palette: &T) -> Result<Vec<Rgb888>, Error> {
        let results = match self.to_bgr555(palette) {
            Ok(r) => r,
            Err(e) => return Err(e),
        };

        Ok(results.iter().map(|&x| x.into()).collect())
    }
    fn direct_color_mode(&self, palette_arg: u8) -> Result<Vec<Bgr555>, Error> {
        let colormap = match self.to_colormap() {
            Ok(c) => c,
            Err(e) => return Err(e),
        };

        let red_bit =   (palette_arg & 1) >> 0;
        let green_bit = (palette_arg & 2) >> 1;
        let blue_bit =  (palette_arg & 4) >> 1;
        let mut result = Vec::<Bgr555>::new();

        for c in &colormap {
            let blue =  ((*c & 0xC0) >> 4) | blue_bit;
            let green = ((*c & 0x38) >> 2) | green_bit;
            let red =   ((*c & 0x7) << 1) | red_bit;
            let mut color = Bgr555(0);
            
            color.set_blue(blue);
            color.set_green(green);
            color.set_red(red);

            result.push(color);
        }

        Ok(result)
    }
}

pub trait SNESGraphic<T: SNESTile>: Sized {
    fn to_vec(&self) -> Vec<T>;
    fn to_colormap(&self) -> Vec<u8>;
    fn from_colormap<B: AsRef<[u8]>>(colormap: B) -> Result<Self, Error>;
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SNESTile1BPP(pub [u8; 8]);
impl TryFrom<&[u8]> for SNESTile1BPP {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl TryFrom<&Vec<u8>> for SNESTile1BPP {
    type Error = Error;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl SNESTile for SNESTile1BPP {
    fn new() -> Self {
        Self([0u8; 8])
    }
    fn from_data<B: AsRef<[u8]>>(data: B) -> Result<Self, Error> {
        let buf = data.as_ref();
        let array: [u8; 8] = match buf.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Error::DataLengthMismatch(buf.len(), 8)),
        };

        Ok(Self(array))
    }
    fn set_value(&mut self, x: usize, y: usize, value: u8) -> Result<(), Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }
        if value >= 2 { return Err(Error::InvalidColorIndex(value)); }

        let index = 7 - x;
        let mask = 1 << index;

        self.0[y] &= mask ^ 0xFF;
        self.0[y] |= value << index;
        
        Ok(())
    }
    fn get_value(&self, x: usize, y: usize) -> Result<u8, Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }

        let index = 7 - x;
        let mask = 1 << index;

        Ok((self.0[y] & mask) >> index)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SNESTile2BPPPlanar(pub [u8; 8*2]);
impl TryFrom<&[u8]> for SNESTile2BPPPlanar {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl TryFrom<&Vec<u8>> for SNESTile2BPPPlanar {
    type Error = Error;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl SNESTile for SNESTile2BPPPlanar {
    fn new() -> Self {
        Self([0u8; 8*2])
    }
    fn from_data<B: AsRef<[u8]>>(data: B) -> Result<Self, Error> {
        let buf = data.as_ref();
        let array: [u8; 8*2] = match buf.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Error::DataLengthMismatch(buf.len(), 8*2)),
        };

        Ok(Self(array))
    }
    fn set_value(&mut self, x: usize, y: usize, value: u8) -> Result<(), Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }
        if value >= 4 { return Err(Error::InvalidColorIndex(value)); }

        let index = 7 - x;
        let mask = 1 << index;

        self.0[y]   &= mask ^ 0xFF;
        self.0[y+8] &= mask ^ 0xFF;

        self.0[y]   |= ((value & 1) >> 0) << index;
        self.0[y+8] |= ((value & 2) >> 1) << index;

        Ok(())
    }
    fn get_value(&self, x: usize, y: usize) -> Result<u8, Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }

        let index = 7 - x;
        let mask = 1 << index;
        let mut value = 0u8;

        value |= ((self.0[y]   & mask) >> index) << 0;
        value |= ((self.0[y+8] & mask) >> index) << 1;

        Ok(value)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SNESTile2BPPIntertwined(pub [u8; 8*2]);
impl TryFrom<&[u8]> for SNESTile2BPPIntertwined {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl TryFrom<&Vec<u8>> for SNESTile2BPPIntertwined {
    type Error = Error;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl SNESTile for SNESTile2BPPIntertwined {
    fn new() -> Self {
        Self([0u8; 8*2])
    }
    fn from_data<B: AsRef<[u8]>>(data: B) -> Result<Self, Error> {
        let buf = data.as_ref();
        let array: [u8; 8*2] = match buf.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Error::DataLengthMismatch(buf.len(), 8*2)),
        };

        Ok(Self(array))
    }
    fn set_value(&mut self, x: usize, y: usize, value: u8) -> Result<(), Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }
        if value >= 4 { return Err(Error::InvalidColorIndex(value)); }

        let index = 7 - x;
        let mask = 1 << index;

        self.0[y*2]   &= mask ^ 0xFF;
        self.0[y*2+1] &= mask ^ 0xFF;

        self.0[y*2]   |= ((value & 1) >> 0) << index;
        self.0[y*2+1] |= ((value & 2) >> 1) << index;

        Ok(())
    }
    fn get_value(&self, x: usize, y: usize) -> Result<u8, Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }

        let index = 7 - x;
        let mask = 1 << index;
        let mut value = 0u8;

        value |= ((self.0[y*2]   & mask) >> index) << 0;
        value |= ((self.0[y*2+1] & mask) >> index) << 1;

        Ok(value)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SNESTile3BPPPlanar(pub [u8; 8*3]);
impl TryFrom<&[u8]> for SNESTile3BPPPlanar {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl TryFrom<&Vec<u8>> for SNESTile3BPPPlanar {
    type Error = Error;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl SNESTile for SNESTile3BPPPlanar {
    fn new() -> Self {
        Self([0u8; 8*3])
    }
    fn from_data<B: AsRef<[u8]>>(data: B) -> Result<Self, Error> {
        let buf = data.as_ref();
        let array: [u8; 8*3] = match buf.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Error::DataLengthMismatch(buf.len(), 8*3)),
        };

        Ok(Self(array))
    }
    fn set_value(&mut self, x: usize, y: usize, value: u8) -> Result<(), Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }
        if value >= 8 { return Err(Error::InvalidColorIndex(value)); }

        let index = 7 - x;
        let mask = 1 << index;

        self.0[y]      &= mask ^ 0xFF;
        self.0[y+0x8]  &= mask ^ 0xFF;
        self.0[y+0x10] &= mask ^ 0xFF;

        self.0[y]      |= ((value & 1) >> 0) << index;
        self.0[y+0x8]  |= ((value & 2) >> 1) << index;
        self.0[y+0x10] |= ((value & 4) >> 2) << index;

        Ok(())
    }
    fn get_value(&self, x: usize, y: usize) -> Result<u8, Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }

        let index = 7 - x;
        let mask = 1 << index;
        let mut value = 0u8;

        value |= ((self.0[y]      & mask) >> index) << 0;
        value |= ((self.0[y+0x8]  & mask) >> index) << 1;
        value |= ((self.0[y+0x10] & mask) >> index) << 2; 

        Ok(value)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SNESTile3BPPIntertwined(pub [u8; 8*3]);
impl TryFrom<&[u8]> for SNESTile3BPPIntertwined {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl TryFrom<&Vec<u8>> for SNESTile3BPPIntertwined {
    type Error = Error;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl SNESTile for SNESTile3BPPIntertwined {
    fn new() -> Self {
        Self([0u8; 8*3])
    }
    fn from_data<B: AsRef<[u8]>>(data: B) -> Result<Self, Error> {
        let buf = data.as_ref();
        let array: [u8; 8*3] = match buf.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Error::DataLengthMismatch(buf.len(), 8*3)),
        };

        Ok(Self(array))
    }
    fn set_value(&mut self, x: usize, y: usize, value: u8) -> Result<(), Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }
        if value >= 8 { return Err(Error::InvalidColorIndex(value)); }

        let index = 7 - x;
        let mask = 1 << index;

        self.0[y*2]    &= mask ^ 0xFF;
        self.0[y*2+1]  &= mask ^ 0xFF;
        self.0[y+0x10] &= mask ^ 0xFF;

        self.0[y*2]    |= ((value & 1) >> 0) << index;
        self.0[y*2+1]  |= ((value & 2) >> 1) << index;
        self.0[y+0x10] |= ((value & 4) >> 2) << index;

        Ok(())
    }
    fn get_value(&self, x: usize, y: usize) -> Result<u8, Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }

        let index = 7 - x;
        let mask = 1 << index;
        let mut value = 0u8;

        value |= ((self.0[y*2]    & mask) >> index) << 0;
        value |= ((self.0[y*2+1]  & mask) >> index) << 1;
        value |= ((self.0[y+0x10] & mask) >> index) << 2;

        Ok(value)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SNESTile4BPPPlanar(pub [u8; 8*4]);
impl TryFrom<&[u8]> for SNESTile4BPPPlanar {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl TryFrom<&Vec<u8>> for SNESTile4BPPPlanar {
    type Error = Error;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl SNESTile for SNESTile4BPPPlanar {
    fn new() -> Self {
        Self([0u8; 8*4])
    }
    fn from_data<B: AsRef<[u8]>>(data: B) -> Result<Self, Error> {
        let buf = data.as_ref();
        let array: [u8; 8*4] = match buf.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Error::DataLengthMismatch(buf.len(), 8*4)),
        };

        Ok(Self(array))
    }
    fn set_value(&mut self, x: usize, y: usize, value: u8) -> Result<(), Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }
        if value >= 16 { return Err(Error::InvalidColorIndex(value)); }

        let index = 7 - x;
        let mask = 1 << index;

        self.0[y]      &= mask ^ 0xFF;
        self.0[y+0x8]  &= mask ^ 0xFF;
        self.0[y+0x10] &= mask ^ 0xFF;
        self.0[y+0x18] &= mask ^ 0xFF;

        self.0[y]      |= ((value & 1) >> 0) << index;
        self.0[y+0x8]  |= ((value & 2) >> 1) << index;
        self.0[y+0x10] |= ((value & 4) >> 2) << index;
        self.0[y+0x18] |= ((value & 8) >> 3) << index;

        Ok(())
    }
    fn get_value(&self, x: usize, y: usize) -> Result<u8, Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }

        let index = 7 - x;
        let mask = 1 << index;
        let mut value = 0u8;

        value |= ((self.0[y]      & mask) >> index) << 0;
        value |= ((self.0[y+0x8]  & mask) >> index) << 1;
        value |= ((self.0[y+0x10] & mask) >> index) << 2;
        value |= ((self.0[y+0x18] & mask) >> index) << 3; 

        Ok(value)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SNESTile4BPPIntertwined(pub [u8; 8*4]);
impl TryFrom<&[u8]> for SNESTile4BPPIntertwined {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl TryFrom<&Vec<u8>> for SNESTile4BPPIntertwined {
    type Error = Error;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl SNESTile for SNESTile4BPPIntertwined {
    fn new() -> Self {
        Self([0u8; 8*4])
    }
    fn from_data<B: AsRef<[u8]>>(data: B) -> Result<Self, Error> {
        let buf = data.as_ref();
        let array: [u8; 8*4] = match buf.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Error::DataLengthMismatch(buf.len(), 8*4)),
        };

        Ok(Self(array))
    }
    fn set_value(&mut self, x: usize, y: usize, value: u8) -> Result<(), Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }
        if value >= 16 { return Err(Error::InvalidColorIndex(value)); }

        let index = 7 - x;
        let mask = 1 << index;

        self.0[y*2+0x00] &= mask ^ 0xFF;
        self.0[y*2+0x01] &= mask ^ 0xFF;
        self.0[y*2+0x10] &= mask ^ 0xFF;
        self.0[y*2+0x11] &= mask ^ 0xFF;

        self.0[y*2+0x00] |= ((value & 1) >> 0) << index;
        self.0[y*2+0x01] |= ((value & 2) >> 1) << index;
        self.0[y*2+0x10] |= ((value & 4) >> 2) << index;
        self.0[y*2+0x11] |= ((value & 8) >> 3) << index;

        Ok(())
    }
    fn get_value(&self, x: usize, y: usize) -> Result<u8, Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }

        let index = 7 - x;
        let mask = 1 << index;
        let mut value = 0u8;

        value |= ((self.0[y*2+0x00] & mask) >> index) << 0;
        value |= ((self.0[y*2+0x01] & mask) >> index) << 1;
        value |= ((self.0[y*2+0x10] & mask) >> index) << 2;
        value |= ((self.0[y*2+0x11] & mask) >> index) << 3;

        Ok(value)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SNESTile8BPPPlanar(pub [u8; 8*8]);
impl TryFrom<&[u8]> for SNESTile8BPPPlanar {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl TryFrom<&Vec<u8>> for SNESTile8BPPPlanar {
    type Error = Error;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl SNESTile for SNESTile8BPPPlanar {
    fn new() -> Self {
        Self([0u8; 8*8])
    }
    fn from_data<B: AsRef<[u8]>>(data: B) -> Result<Self, Error> {
        let buf = data.as_ref();
        let array: [u8; 8*8] = match buf.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Error::DataLengthMismatch(buf.len(), 8*8)),
        };

        Ok(Self(array))
    }
    fn set_value(&mut self, x: usize, y: usize, value: u8) -> Result<(), Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }
        if value >= 16 { return Err(Error::InvalidColorIndex(value)); }

        let index = 7 - x;
        let mask = 1 << index;

        self.0[y]      &= mask ^ 0xFF;
        self.0[y+0x8]  &= mask ^ 0xFF;
        self.0[y+0x10] &= mask ^ 0xFF;
        self.0[y+0x18] &= mask ^ 0xFF;
        self.0[y+0x20] &= mask ^ 0xFF;
        self.0[y+0x28] &= mask ^ 0xFF;
        self.0[y+0x30] &= mask ^ 0xFF;
        self.0[y+0x38] &= mask ^ 0xFF;

        self.0[y]      |= ((value & 0x01) >> 0) << index;
        self.0[y+0x8]  |= ((value & 0x02) >> 1) << index;
        self.0[y+0x10] |= ((value & 0x04) >> 2) << index;
        self.0[y+0x18] |= ((value & 0x08) >> 3) << index;
        self.0[y+0x20] |= ((value & 0x10) >> 4) << index;
        self.0[y+0x28] |= ((value & 0x20) >> 5) << index;
        self.0[y+0x30] |= ((value & 0x40) >> 6) << index;
        self.0[y+0x38] |= ((value & 0x80) >> 7) << index;

        Ok(())
    }
    fn get_value(&self, x: usize, y: usize) -> Result<u8, Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }

        let index = 7 - x;
        let mask = 1 << index;
        let mut value = 0u8;

        value |= ((self.0[y]      & mask) >> index) << 0;
        value |= ((self.0[y+0x8]  & mask) >> index) << 1;
        value |= ((self.0[y+0x10] & mask) >> index) << 2;
        value |= ((self.0[y+0x18] & mask) >> index) << 3; 
        value |= ((self.0[y+0x20] & mask) >> index) << 4; 
        value |= ((self.0[y+0x28] & mask) >> index) << 5; 
        value |= ((self.0[y+0x30] & mask) >> index) << 6; 
        value |= ((self.0[y+0x38] & mask) >> index) << 7; 

        Ok(value)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SNESTile8BPPIntertwined(pub [u8; 8*8]);
impl TryFrom<&[u8]> for SNESTile8BPPIntertwined {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl TryFrom<&Vec<u8>> for SNESTile8BPPIntertwined {
    type Error = Error;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl SNESTile for SNESTile8BPPIntertwined {
    fn new() -> Self {
        Self([0u8; 8*8])
    }
    fn from_data<B: AsRef<[u8]>>(data: B) -> Result<Self, Error> {
        let buf = data.as_ref();
        let array: [u8; 8*8] = match buf.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Error::DataLengthMismatch(buf.len(), 8*8)),
        };

        Ok(Self(array))
    }
    fn set_value(&mut self, x: usize, y: usize, value: u8) -> Result<(), Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }

        let index = 7 - x;
        let mask = 1 << index;

        self.0[y*2+0x00] &= mask ^ 0xFF;
        self.0[y*2+0x01] &= mask ^ 0xFF;
        self.0[y*2+0x10] &= mask ^ 0xFF;
        self.0[y*2+0x11] &= mask ^ 0xFF;
        self.0[y*2+0x20] &= mask ^ 0xFF;
        self.0[y*2+0x21] &= mask ^ 0xFF;
        self.0[y*2+0x30] &= mask ^ 0xFF;
        self.0[y*2+0x31] &= mask ^ 0xFF;

        self.0[y*2+0x00] |= ((value & 0x01) >> 0) << index;
        self.0[y*2+0x01] |= ((value & 0x02) >> 1) << index;
        self.0[y*2+0x10] |= ((value & 0x04) >> 2) << index;
        self.0[y*2+0x11] |= ((value & 0x08) >> 3) << index;
        self.0[y*2+0x20] |= ((value & 0x10) >> 4) << index;
        self.0[y*2+0x21] |= ((value & 0x20) >> 5) << index;
        self.0[y*2+0x30] |= ((value & 0x40) >> 6) << index;
        self.0[y*2+0x31] |= ((value & 0x80) >> 7) << index;

        Ok(())
    }
    fn get_value(&self, x: usize, y: usize) -> Result<u8, Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }

        let index = 7 - x;
        let mask = 1 << index;
        let mut value = 0u8;

        value |= ((self.0[y*2+0x00] & mask) >> index) << 0;
        value |= ((self.0[y*2+0x01] & mask) >> index) << 1;
        value |= ((self.0[y*2+0x10] & mask) >> index) << 2;
        value |= ((self.0[y*2+0x11] & mask) >> index) << 3;
        value |= ((self.0[y*2+0x20] & mask) >> index) << 4;
        value |= ((self.0[y*2+0x21] & mask) >> index) << 5;
        value |= ((self.0[y*2+0x30] & mask) >> index) << 6;
        value |= ((self.0[y*2+0x31] & mask) >> index) << 7;

        Ok(value)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SNESTileMode7(pub [u8; 8*8]);
impl TryFrom<&[u8]> for SNESTileMode7 {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl TryFrom<&Vec<u8>> for SNESTileMode7 {
    type Error = Error;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_data(value)
    }
}
impl SNESTile for SNESTileMode7 {
    fn new() -> Self {
        Self([0u8; 8*8])
    }
    fn from_data<B: AsRef<[u8]>>(data: B) -> Result<Self, Error> {
        let buf = data.as_ref();
        let array: [u8; 8*8] = match buf.try_into() {
            Ok(a) => a,
            Err(_) => return Err(Error::DataLengthMismatch(buf.len(), 8*8)),
        };

        Ok(Self(array))
    }
    fn set_value(&mut self, x: usize, y: usize, value: u8) -> Result<(), Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }

        self.0[y*8+x] = value;
        Ok(())
    }
    fn get_value(&self, x: usize, y: usize) -> Result<u8, Error> {
        if x >= 8 { return Err(Error::OutOfBounds(x,8)); }
        if y >= 8 { return Err(Error::OutOfBounds(y,8)); }

        Ok(self.0[y*8+x])
    }
}
