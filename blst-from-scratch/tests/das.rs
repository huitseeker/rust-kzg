#[cfg(test)]
mod tests {
    use blst_from_scratch::types::fft_settings::FsFFTSettings;
    use blst_from_scratch::types::fr::FsFr;
    use kzg_bench::tests::das::{das_extension_test_known, das_extension_test_random};

    #[test]
    fn das_extension_test_known_() {
        das_extension_test_known::<FsFr, FsFFTSettings>();
    }

    #[test]
    fn das_extension_test_random_() {
        das_extension_test_random::<FsFr, FsFFTSettings>();
    }
}
