//! CSV export of raw blocks.

use chrono::{Local, TimeZone};

use crate::db::BlockRow;

const HEADER: &str = "start,end,duration_s,kind,app_name,app_id,title";

pub fn to_csv(blocks: &[BlockRow]) -> String {
    let mut out = String::from(HEADER);
    out.push('\n');
    for b in blocks {
        let fields = [
            timestamp(b.start),
            timestamp(b.end),
            ((b.end - b.start) / 1000).to_string(),
            b.kind.as_str().to_string(),
            b.app_name.clone().unwrap_or_default(),
            b.app_id.clone().unwrap_or_default(),
            b.title.clone().unwrap_or_default(),
        ];
        out.push_str(&fields.map(|f| escape(&f)).join(","));
        out.push('\n');
    }
    out
}

/// Local time with its UTC offset, e.g. `2026-10-28T09:14:02+08:00`.
fn timestamp(ms: i64) -> String {
    Local
        .timestamp_millis_opt(ms)
        .single()
        .map(|d| d.format("%Y-%m-%dT%H:%M:%S%:z").to_string())
        .unwrap_or_default()
}

fn escape(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::Kind;

    #[test]
    fn writes_rows_and_escapes_fields() {
        let blocks = [
            BlockRow {
                start: 0,
                end: 90_500,
                kind: Kind::Active,
                app_id: Some("com.example".into()),
                app_name: Some("Example".into()),
                title: Some("Say \"hi\", world".into()),
            },
            BlockRow { start: 90_500, end: 100_000, kind: Kind::Away, app_id: None, app_name: None, title: None },
        ];
        let csv = to_csv(&blocks);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], HEADER);
        assert!(lines[1].ends_with(",90,active,Example,com.example,\"Say \"\"hi\"\", world\""));
        assert!(lines[2].ends_with(",9,away,,,"));
        assert_eq!(lines.len(), 3);
    }
}
