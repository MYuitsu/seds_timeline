use std::fs;

use serde_json::Value;
use timeline_core::TimelineConfig;
use timeline_fhir::summarize_bundle_str;

fn fixture_path(name: &str) -> String {
    format!("{}/tests/data/{name}", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn observation_bundle_matches_golden() {
    let bundle = fs::read_to_string(fixture_path("emergency_observation_bundle.json"))
        .expect("Không đọc được bundle mẫu");

    let snapshot =
        summarize_bundle_str(&bundle, &TimelineConfig::default()).expect("Không tạo được snapshot");

    let mut actual = serde_json::to_value(snapshot).expect("Không serialize snapshot");
    normalize_dynamic_fields(&mut actual);

    let expected = fs::read_to_string(fixture_path("emergency_observation_snapshot.json"))
        .expect("Không đọc được golden snapshot");

    let mut expected_value: Value = serde_json::from_str(&expected).expect("Golden không hợp lệ");
    normalize_dynamic_fields(&mut expected_value);

    assert_eq!(actual, expected_value);
}

fn normalize_dynamic_fields(value: &mut Value) {
    if let Some(obj) = value.as_object_mut() {
        if obj.contains_key("generated_at") {
            obj.insert(
                "generated_at".to_string(),
                Value::String("__DYNAMIC_TIMESTAMP__".to_string()),
            );
        }
    }
}
