use crate::mcl_methods;
use std::ops::{Add, AddAssign};
use std::ops::{Div, DivAssign};
use std::ops::{Mul, MulAssign};
use std::ops::{Sub, SubAssign};
use std::os::raw::c_int;

#[link(name = "mcl", kind = "static")]
#[link(name = "mclbn384_256", kind = "static")]
#[link(name = "stdc++")]
#[allow(non_snake_case)]
extern "C" {
    fn mclBnFr_isEqual(x: *const Fr, y: *const Fr) -> i32;
    fn mclBnFr_isValid(x: *const Fr) -> i32;
    fn mclBnFr_isZero(x: *const Fr) -> i32;
    fn mclBnFr_isOne(x: *const Fr) -> i32;
    fn mclBnFr_isOdd(x: *const Fr) -> i32;
    fn mclBnFr_isNegative(x: *const Fr) -> i32;

    fn mclBnFr_setStr(x: *mut Fr, buf: *const u8, bufSize: usize, ioMode: i32) -> c_int;
    fn mclBnFr_getStr(buf: *mut u8, maxBufSize: usize, x: *const Fr, ioMode: i32) -> usize;
    fn mclBnFr_serialize(buf: *mut u8, maxBufSize: usize, x: *const Fr) -> usize;
    fn mclBnFr_deserialize(x: *mut Fr, buf: *const u8, bufSize: usize) -> usize;

    fn mclBnFr_setInt32(x: *mut Fr, v: i32);
    fn mclBnFr_setLittleEndian(x: *mut Fr, buf: *const u8, bufSize: usize) -> i32;
    fn mclBnFr_getLittleEndian(buf: *mut u8, bufSize: usize, x: *const Fr) -> i32;
    fn mclBnFr_setLittleEndianMod(x: *mut Fr, buf: *const u8, bufSize: usize) -> i32;
    fn mclBnFr_setHashOf(x: *mut Fr, buf: *const u8, bufSize: usize) -> i32;
    fn mclBnFr_setByCSPRNG(x: *mut Fr);

    fn mclBnFr_add(z: *mut Fr, x: *const Fr, y: *const Fr);
    fn mclBnFr_sub(z: *mut Fr, x: *const Fr, y: *const Fr);
    fn mclBnFr_neg(y: *mut Fr, x: *const Fr);

    fn mclBnFr_mul(z: *mut Fr, x: *const Fr, y: *const Fr);
    fn mclBnFr_div(z: *mut Fr, x: *const Fr, y: *const Fr);
    fn mclBnFr_inv(y: *mut Fr, x: *const Fr);
    fn mclBnFr_sqr(y: *mut Fr, x: *const Fr);
    fn mclBnFr_squareRoot(y: *mut Fr, x: *const Fr) -> i32;
}

#[derive(Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct Fr {
    pub d: [u64; crate::MCLBN_FR_UNIT_SIZE],
}

pub struct U2516([u64; 4]);

impl Fr {
    pub fn get_order() -> String {
        mcl_methods::get_curve_order()
    }

    pub fn pow(&self, n: usize) -> Self {
        if n == 0 {
            return Self::one();
        }
        let mut res = *self;
        for _ in 1..n {
            res = res * *self;
        }
        res
    }

    pub fn inverse(&self) -> Self {
        let mut res = Fr::zero();
        Fr::inv(&mut res, self);
        res
    }

    pub fn from_u64_arr(u: &[u64; 4]) -> Self {
        let mut arr = [0u8; 32];
        for i in 0..4 {
            arr[i * 8..i * 8 + 8].copy_from_slice(&u[i].to_le_bytes());
        }
        Fr::from_scalar(&arr).unwrap()
    }

    pub fn from_scalar(scalar: &[u8; 32]) -> Result<Self, u8> {
        let mut t = Fr::default();
        if !t.set_little_endian_mod(scalar) {
            return Err(1);
        }
        Ok(t)
    }

    pub fn to_scalar(fr: &Self) -> [u8; 32] {
        let mut buf = [0u8; 32];
        assert!(fr.get_little_endian(&mut buf));
        buf
    }

    pub fn to_u64_arr(&self) -> [u64; 4] {
        let v: Vec<u64> = Fr::to_scalar(self)
            .chunks(8)
            .map(|ch| u64::from_le_bytes(ch.try_into().unwrap()))
            .collect();

        v.try_into().unwrap()
    }
}
common_impl![Fr, mclBnFr_isEqual, mclBnFr_isZero];
is_valid_impl![Fr, mclBnFr_isValid];
serialize_impl![
    Fr,
    mcl_methods::mclBn_getFrByteSize(),
    mclBnFr_serialize,
    mclBnFr_deserialize
];
str_impl![Fr, 1024, mclBnFr_getStr, mclBnFr_setStr];
int_impl![Fr, mclBnFr_setInt32, mclBnFr_isOne];
base_field_impl![
    Fr,
    mclBnFr_setLittleEndian,
    mclBnFr_getLittleEndian,
    mclBnFr_setLittleEndianMod,
    mclBnFr_setHashOf,
    mclBnFr_setByCSPRNG,
    mclBnFr_isOdd,
    mclBnFr_isNegative,
    mclBnFr_squareRoot
];
add_op_impl![Fr, mclBnFr_add, mclBnFr_sub, mclBnFr_neg];
field_mul_op_impl![Fr, mclBnFr_mul, mclBnFr_div, mclBnFr_inv, mclBnFr_sqr];
