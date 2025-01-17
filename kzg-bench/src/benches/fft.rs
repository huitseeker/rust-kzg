use criterion::Criterion;
use kzg::{FFTFr, FFTSettings, Fr, FFTG1, G1};

const BENCH_SCALE: usize = 15;

pub fn bench_fft_fr<TFr: 'static + Fr, TFFTSettings: 'static + FFTSettings<TFr> + FFTFr<TFr>>(
    c: &mut Criterion,
) {
    let fft_settings = TFFTSettings::new(BENCH_SCALE).unwrap();
    let data: Vec<TFr> = vec![TFr::rand(); fft_settings.get_max_width()];
    let id = format!("bench_fft_fr scale: '{}'", BENCH_SCALE);
    c.bench_function(&id, move |b| b.iter(|| fft_settings.fft_fr(&data, false)));
}

pub fn bench_fft_g1<
    TFr: 'static + Fr,
    TG1: 'static + G1,
    TFFTSettings: 'static + FFTSettings<TFr> + FFTG1<TG1>,
>(
    c: &mut Criterion,
) {
    let fft_settings = TFFTSettings::new(BENCH_SCALE).unwrap();
    let data: Vec<TG1> = vec![TG1::rand(); fft_settings.get_max_width()];
    let id = format!("bench_fft_g1 scale: '{}'", BENCH_SCALE);
    c.bench_function(&id, move |b| b.iter(|| fft_settings.fft_g1(&data, false)));
}
