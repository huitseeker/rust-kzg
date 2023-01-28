use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use kzg::Poly;
use blst::{BLST_ERROR, blst_p1, blst_p1_affine, blst_p1_from_affine, blst_p1_uncompress, blst_p2, blst_p2_affine, blst_p2_from_affine, blst_p2_uncompress};
use kzg::{Fr, FFTSettings as FFTSettingsT, FFTG1, KZGSettings as LKZGSettings};
use kzg_bench::tests::fk20_proofs::reverse_bit_order;
use crate::kzg_proofs::{FFTSettings, KZGSettings};
use crate::kzg_types::{ArkG1, ArkG2, FsFr};
use crate::utils::{PolyData};

pub fn bytes_from_bls_field(fr: &FsFr) -> [u8; 32usize] {
    fr.to_scalar()
}

pub fn bytes_to_bls_field(bytes: &[u8; 32usize]) -> FsFr {
    FsFr::from_scalar(*bytes)
}

pub fn compute_powers(base: &FsFr, num_powers: usize) -> Vec<FsFr> {
    let mut powers: Vec<FsFr> = vec![FsFr::default(); num_powers];
    if num_powers == 0 {
        return powers;
    }
    powers[0] = FsFr::one();
    for i in 1..num_powers {
        powers[i] = powers[i - 1].mul(base);
    }
    powers
}

pub fn bytes_to_g1(bytes: &[u8; 48usize]) -> ArkG1 {
    let mut tmp = blst_p1_affine::default();
    let mut g1 = blst_p1::default();
    unsafe {
        if blst_p1_uncompress(&mut tmp, bytes.as_ptr()) != BLST_ERROR::BLST_SUCCESS {
            panic!("blst_p1_uncompress failed");
        }
        blst_p1_from_affine(&mut g1, &tmp);
    }
    ArkG1(g1)
}

pub fn bytes_to_g2(bytes: &[u8; 96usize]) -> ArkG2 {
    let mut tmp = blst_p2_affine::default();
    let mut g2 = blst_p2::default();
    unsafe {
        if blst_p2_uncompress(&mut tmp, bytes.as_ptr()) != BLST_ERROR::BLST_SUCCESS {
            panic!("blst_p2_uncompress failed");
        }
        blst_p2_from_affine(&mut g2, &tmp);
    }
    ArkG2(g2)
}

fn fr_batch_inv(out: &mut [FsFr], a: &[FsFr], len: usize) {
    let prod: &mut Vec<FsFr> = &mut vec![FsFr::default(); len];
    let mut i: usize = 1;

    prod[0] = a[0];

    while i < len {
        prod[i] = a[i].mul(&prod[i - 1]);
        i += 1;
    }

    let inv: &mut FsFr = &mut prod[len - 1].eucl_inverse();

    i = len - 1;
    while i > 0 {
        out[i] = prod[i - 1].mul(inv);
        *inv = a[i].mul(inv);
        i -= 1;
    }
    out[0] = *inv;
}

pub fn load_trusted_setup(filepath: &str) -> KZGSettings {
    let mut file = File::open(filepath).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    let mut lines = contents.lines();
    let length = lines.next().unwrap().parse::<usize>().unwrap();
    let n2 = lines.next().unwrap().parse::<usize>().unwrap();

    let mut g2_values: Vec<ArkG2> = Vec::new();
    let mut g1_projectives: Vec<ArkG1> = Vec::new();

    for _ in 0..length {
        let line = lines.next().unwrap();
        assert_eq!(line.len(), 96);
        let bytes_array: [u8; 48] = (0..line.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&line[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();
        g1_projectives.push(bytes_to_g1(&bytes_array));
    }

    for _ in 0..n2 {
        let line = lines.next().unwrap();
        assert_eq!(line.len(), 192);
        let bytes_array: [u8; 96] = (0..line.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&line[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();
        g2_values.push(bytes_to_g2(&bytes_array));
    }

    let mut max_scale: usize = 0;
    while (1 << max_scale) < length {
        max_scale += 1;
    }
    let fs = FFTSettings::new(max_scale).unwrap();
    let mut g1_values = fs.fft_g1(&g1_projectives, true).unwrap();
    reverse_bit_order(&mut g1_values);

    KZGSettings {
        secret_g1: g1_values,
        secret_g2: g2_values,
        fs,
        ..KZGSettings::default()
    }
}

pub fn evaluate_polynomial_in_evaluation_form(p: &PolyData, x: &FsFr, s: &KZGSettings) -> FsFr {
    let mut tmp: FsFr;
    let mut inverses_in: Vec<FsFr> = vec![FsFr::default(); p.len()];
    let mut inverses: Vec<FsFr> = vec![FsFr::default(); p.len()];
    let mut i: usize = 0;
    let mut roots_of_unity: Vec<FsFr> = s.fs.expanded_roots_of_unity.clone();

    reverse_bit_order(&mut roots_of_unity);

    while i < p.len() {
        if x.equals(&roots_of_unity[i]) {
            return p.get_coeff_at(i);
        }

        inverses_in[i] = x.sub(&roots_of_unity[i]);
        i += 1;
    }
    fr_batch_inv(&mut inverses, &inverses_in, p.len());

    i = 0;
    let mut out = FsFr::zero();
    while i < p.len() {
        tmp = inverses[i].mul(&roots_of_unity[i]);
        tmp = tmp.mul(&p.coeffs[i]);
        out = out.add(&tmp);
        i += 1;
    }

    tmp = FsFr::from_u64(p.len().try_into().unwrap());
    out = out.div(&tmp).unwrap();
    tmp = x.pow(p.len());
    tmp = tmp.sub(&FsFr::one());
    out = out.mul(&tmp);
    out
}