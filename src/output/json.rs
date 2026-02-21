use crate::report::Report;

pub fn render_json(report: &Report) -> String {
    serde_json::to_string_pretty(report)
        .expect("failed to serialize report as JSON")
}
