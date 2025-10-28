// Integration test module - makes tests in integration/ directory visible to cargo test

mod integration {
    mod coordinate_tests;
    mod measurement_tests;
    mod layer_tests;
    mod query_tests;
    mod aggregate_tests;
    mod quickstart_tests;
}
