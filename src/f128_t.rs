use std::ops::*;
use ffi;
use ffi::*;
use std::convert::{ From, Into };
use std::iter::*;
use std::hash::{ Hash, Hasher };
use std::mem;
use std::slice;
use std::ffi::CString;
use std::ffi::NulError;
use num::*;
use f128_derive::*;
use std::num::FpCategory;
use libc::c_int;

#[derive(Clone, Copy)]
pub struct f128(pub [u8; 16]);

pub trait To16Bytes {
    fn to_arr(&self) -> [u8; 16];
    fn to_u128(&self) -> u128;
    fn to_i128(&self) -> i128;
}

impl f128 {
    pub const RADIX: u32 = 128;
    pub const MANTISSA_DIGITS: u32 = 112;
    pub const INFINITY: f128 = f128::infinity();
    pub const NEG_INFINITY: f128 = f128::neg_infinity();

    pub const MAX_10_EXP: u32 = 4932;
    pub const MAX_EXP: u32 = 16383;
    pub const MIN_10_EXP: i32 = -4931;
    pub const MIN_EXP: i32 = -16382;
    pub const NAN: f128 = f128::nan();


    #[cfg(target_endian = "big")]
    pub const SIGN_BIT: f128 = f128([0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    #[cfg(target_endian = "little")]
    pub const SIGN_BIT: f128 = f128([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]);

    #[cfg(target_endian = "big")]
    pub const EXPONENT_BITS : f128 = f128([0x7f, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    #[cfg(target_endian = "little")]
    pub const EXPONENT_BITS : f128 = f128([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x7f]);

    #[cfg(target_endian = "big")]
    pub const FRACTION_BITS : f128 = f128([0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    #[cfg(target_endian = "little")]
    pub const FRACTION_BITS : f128 = f128([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00]);

    #[cfg(target_endian = "big")]
    pub const MIN           : f128 = f128([0xff, 0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    #[cfg(target_endian = "little")]
    pub const MIN           : f128 = f128([0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0xff]);

    #[cfg(target_endian = "big")]
    pub const MIN_POSITIVE  : f128 = f128([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]);
    #[cfg(target_endian = "little")]
    pub const MIN_POSITIVE  : f128 = f128([0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    #[cfg(target_endian = "big")]
    pub const ONE           : f128 = f128([0x3f, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    #[cfg(target_endian = "little")]
    pub const ONE           : f128 = f128([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x3f]);

    #[cfg(target_endian = "big")]
    pub const TWO           : f128 = f128([0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    #[cfg(target_endian = "little")]
    pub const TWO           : f128 = f128([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40]);

    #[cfg(target_endian = "big")]
    pub const E             : f128 = f128([0x40, 0x00, 0x5b, 0xf0, 0xa8, 0xb1, 0x45, 0x76, 0x95, 0x35, 0x5f, 0xb8, 0xac, 0x40, 0x4e, 0x7a]);
    #[cfg(target_endian = "little")]
    pub const E             : f128 = f128([0x7a, 0x4e, 0x40, 0xac, 0xb8, 0x5f, 0x35, 0x95, 0x76, 0x45, 0xb1, 0xa8, 0xf0, 0x5b, 0x00, 0x40]);

    #[cfg(target_endian = "big")]
    pub const PI            : f128 = f128([0x40, 0x00, 0x92, 0x1f, 0xb5, 0x44, 0x42, 0xd1, 0x84, 0x69, 0x89, 0x8c, 0xc5, 0x17, 0x01, 0xb8]);
    #[cfg(target_endian = "little")]
    pub const PI            : f128 = f128([0xb8, 0x01, 0x17, 0xc5, 0x8c, 0x89, 0x69, 0x84, 0xd1, 0x42, 0x44, 0xb5, 0x1f, 0x92, 0x00, 0x40]);
#[inline(always)]
    #[cfg(target_endian = "little")]
    pub const fn nan() -> Self {
        f128([ 0xFF, 0xFF, 0xFF,  0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFFu8, 0xFFu8, 0x7F ])
    }

    #[inline(always)]
    pub const fn infinity() -> Self {
        f128([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0x7f])
    }

    #[inline(always)]
    pub const fn neg_infinity() -> Self {
        f128([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff, 0xff])
    }

    #[inline(always)]
    pub const fn zero() -> Self { f128([0u8; 16]) }
    #[inline(always)]
    pub const fn neg_zero() -> Self { f128([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 80u8]) }
    #[cfg(target_endian = "little")]
    #[inline(always)]
    pub const fn min_value() -> f128 { f128([ 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0xff]) }
    #[cfg(target_endian = "little")]
    #[inline(always)]
    pub const fn max_value() -> f128 { f128([0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0x7f]) }

    #[cfg(target_endian = "big")]
    pub fn from_arr(d: [u8; 16]) -> Self { f128(d) }
    #[cfg(target_endian = "little")]
    pub fn from_arr(mut d: [u8; 16]) -> Self { d.reverse(); f128(d) }

    #[inline(always)]
    pub fn from_raw_u128(d: u128) -> Self { f128::from_arr(unsafe { mem::transmute::<u128, [u8; 16]>(d) }) }
    #[inline(always)]
    pub fn from_raw_i128(d: i128) -> Self { f128::from_arr(unsafe { mem::transmute::<i128, [u8; 16]>(d) }) }

    #[inline(always)]
    pub fn inner_as_i128(self) -> i128 { unsafe { mem::transmute::<[u8; 16], i128>(self.0) } }

    #[inline(always)]
    pub fn inner_as_u128(&self) -> u128 { unsafe { mem::transmute::<[u8; 16], u128>(self.0) } }

    #[inline(always)]
    pub fn from_bits<T: To16Bytes>(x: &To16Bytes) -> Self {
        f128( x.to_arr() )
    }


    pub fn to_i64(self)    -> i64 { unsafe { f128_to_i64(self.inner()) } }
    pub fn to_u64(self)    -> u64 { unsafe { f128_to_u64(self.inner()) } }
    pub fn to_isize(self)  -> isize { unsafe { f128_to_i64(self.inner()) as isize } }
    pub fn to_i8(self)     -> i8 { unsafe { f128_to_i8(self.inner()) } }
    pub fn to_i16(self)    -> i16 { unsafe { f128_to_i16(self.inner()) } }
    pub fn to_i32(self)    -> i32 { unsafe { f128_to_i32(self.inner()) } }
    pub fn to_usize(self)  -> usize { unsafe { f128_to_u64(self.inner()) as usize } }
    pub fn to_u8(self)     -> u8 { unsafe { f128_to_u8(self.inner()) } }
    pub fn to_u16(self)    -> u16 { unsafe { f128_to_u16(self.inner()) } }
    pub fn to_u32(self)    -> u32 { unsafe { f128_to_u32(self.inner()) } }
    pub fn to_f32(self)    -> f32 { unsafe { f128_to_f32(self.inner()) } }
    pub fn to_f64(self)    -> f64 { unsafe { f128_to_f64(self.inner()) } }
    pub fn to_i128(self)   -> i128 { unsafe { mem::transmute::<[u8; 16], i128>(f128_to_i128(self.inner())) } }
    pub fn to_u128(self)   -> u128 { unsafe { mem::transmute::<[u8; 16], u128>(f128_to_u128(self.inner())) } }

    pub fn from_i64(n: i64) -> Self { (unsafe { f128::from_arr(i64_to_f128(n)) }) }
    pub fn from_u64(n: u64) -> Self { (unsafe { f128::from_arr(u64_to_f128(n)) }) }
    pub fn from_isize(n: isize) -> Self { (unsafe { f128::from_arr(i64_to_f128(n as i64)) })}
    pub fn from_i8(n: i8) -> Self { (unsafe { f128::from_arr(i8_to_f128(n)) }) }
    pub fn from_i16(n: i16) -> Self { (unsafe { f128::from_arr(i16_to_f128(n)) }) }
    pub fn from_i32(n: i32) -> Self { (unsafe { f128::from_arr(i32_to_f128(n)) }) }
    pub fn from_usize(n: usize) -> Self { (unsafe { f128::from_arr(u64_to_f128(n as u64)) }) }
    pub fn from_u8(n: u8) -> Self { (unsafe { f128::from_arr(u8_to_f128(n)) }) }
    pub fn from_u16(n: u16) -> Self { (unsafe { f128::from_arr(u16_to_f128(n)) }) }
    pub fn from_u32(n: u32) -> Self { (unsafe { f128::from_arr(u32_to_f128(n)) }) }
    pub fn from_f32(n: f32) -> Self { (unsafe { f128::from_arr(f32_to_f128(n)) }) }
    pub fn from_f64(n: f64) -> Self { (unsafe { f128::from_arr(f64_to_f128(n)) }) }
    pub fn from_u128(n: u128) -> Self { unsafe { f128::from_arr(u128_to_f128(mem::transmute::<u128, [u8; 16]>(n))) } }
    pub fn from_i128(n: i128) -> Self { unsafe { f128::from_arr(i128_to_f128(mem::transmute::<i128, [u8; 16]>(n))) } }


    #[inline(always)]
    pub fn new<T: Into<f128>>(a: T) -> Self { a.into() }

    pub fn to_string(&self) -> String {
        self.to_string_fmt("%.36Qg").unwrap()
    }

    pub fn to_string_fmt<T: AsRef<str>>(&self, fmt: T) -> Option<String> {
        let mut buf: [u8; 128] = [0; 128];
        let cstr;
        match CString::new(fmt.as_ref()) {
            Ok(e) => cstr = e,
            Err(_) => return None
        };
        let n = unsafe { qtostr((&mut buf).as_mut_ptr(), 128, cstr.as_ptr(), self.inner()) };
        let mut v = Vec::with_capacity(n as usize);
        for i in 0..n {
            v.push(buf[i as usize]);
        }
        Some(String::from_utf8(v).unwrap())
    }

    #[inline(always)]
    pub fn inner(&self) -> [u8; 16] { self.0 }

    #[inline(always)]
    pub fn into_inner(self) -> [u8; 16] { self.0 }

    pub fn parse<T: AsRef<str>>(s: T) -> Result<Self, NulError> {
        let cstr = CString::new(s.as_ref())?;
        let result = unsafe { strtoflt128_f(cstr.as_ptr()) };

        Ok(unsafe { f128(strtoflt128_f(cstr.as_ptr()))})
    }


    pub fn is_finite(&self) -> bool {
        !self.is_infinite() && !self.is_nan()
    }

    pub fn is_infinite(&self) -> bool {
        // It's fine to compare the bits here since there is only 1 bit pattern that is inf, and one
        // that is -inf.
        let res = (self.inner_as_u128() & 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128);
        res == f128::EXPONENT_BITS.inner_as_u128()
    }

    pub fn is_nan(&self) -> bool {
        (self.inner_as_u128() & 0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu128) > f128::EXPONENT_BITS.inner_as_u128()
    }

    pub fn is_normal(&self) -> bool {
        let exp = self.exp_bits();
        exp >= 0x0001u32 && exp <= 0x7FFEu32
    }

    pub fn exp_bits(&self) -> u32 {
        ((self.inner_as_u128() & f128::EXPONENT_BITS.inner_as_u128()) >> 112) as u32
    }

    pub fn fract_bits(&self) -> u128 {
        self.inner_as_u128() & f128::FRACTION_BITS.inner_as_u128()
    }

    pub fn classify(&self) -> FpCategory {
        let x = (self.is_normal(), self.is_finite(), self.is_nan());
        println!("{:?}", x);
        match x {
            (true, true, false) => FpCategory::Normal,
            (false, true, false) => FpCategory::Subnormal,
            (_, _, true) => FpCategory::Nan,
            (_, false, _) => FpCategory::Infinite,
            _ => unreachable!()
        }
    }

    pub fn floor(self) -> Self {
        f128::from_arr(unsafe { floorq_f(self.0) })
    }

    pub fn ceil(self) -> Self {
        f128::from_arr(unsafe { ceilq_f(self.0) })
    }

    pub fn round(self) -> Self {
        f128::from_arr(unsafe { roundq_f(self.0) })
    }

    pub fn trunc(self) -> Self {
        f128::from_arr(unsafe { truncq_f(self.0) })
    }

    pub fn fract(self) -> Self {
        let mut x: c_int = 0;
        let _ = unsafe { frexpq_f(self.0, &mut x) };
        f128::from_u32(x as u32)
    }

    #[cfg(target_endian = "big")]
    pub fn abs(mut self) -> Self {
        self.0[0] &= 0x7F;
        self
    }
    #[cfg(target_endian = "little")]
    pub fn abs(mut self) -> Self {
        self.0[15] &= 0x7F;
        self
    }

    pub fn signum(self) -> Self {
        match self.0[0] & 0x80 {
            0 => f128::INFINITY,
            1 => f128::NEG_INFINITY,
            _ => unreachable!()
        }
    }

    pub fn is_sign_negative(self) -> bool {
        match self.0[0] & 0x80 {
            0 => false,
            1 => true,
            _ => unreachable!()
        }
    }

    pub fn is_sign_positive(self) -> bool {
        match self.0[0] & 0x80 {
            1 => false,
            0 => true,
            _ => unreachable!()
        }
    }

    pub fn mul_add(self, a: f128, b: f128) -> f128 {
        f128::from_arr(unsafe { fmaq_f(self.0, a.0, b.0) })
    }

    pub fn recip(self) -> f128 {
        f128::ONE / self
    }

    pub fn powi(self, n: i32) -> f128 {
        let mut i = self.clone();
        for _ in 0..n {
            i *= self;
        }
        i
    }

    pub fn powf(self, n: f128) -> f128 {
        f128::from_arr(unsafe { powq_f(self.0, n.0) })
    }

    pub fn sqrt(self) -> f128 {
        f128::from_arr(unsafe { sqrtq_f(self.0) })
    }

    pub fn exp(self) -> f128 {
        f128::from_arr(unsafe { expq_f(self.0) })
    }

    pub fn exp2(self) -> f128 {
        (f128::ONE * f128::from_u8(2)).powf(self)
    }


}

