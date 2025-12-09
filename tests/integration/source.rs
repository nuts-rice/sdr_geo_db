use sdr_db::source::{file::FileSpectrum, spectrum::SpectrumDataSource};

#[test]
fn test_file_spectrum_load() {
    let file_path = "./recordings/sweep.csv".to_string();
    let mut file_spectrum = FileSpectrum::from_csv(file_path).unwrap();
    let spectrum_data = file_spectrum
        .get_spectrum_data(855000000., 10000000.)
        .unwrap();
    assert!(!spectrum_data.is_empty());
}
