use regex::Regex;
use std::fs::File;
use walkdir::{DirEntry, WalkDir};

#[derive(PartialEq, Debug)]
enum ScheduleType {
    Deadline,
    Schedule,
}

#[derive(PartialEq, Debug)]
struct Schedule {
    head: String,
    schedule: String,
    scheduletype: ScheduleType,
}

fn main() {
    let org_files = list_org_files();
}

fn open_org_file(entry: &DirEntry) -> Option<File> {
    File::open(entry.path()).ok()
}

fn extract_org_head_and_schedules() {}

fn is_org_file(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".org"))
        .unwrap_or(false)
}

fn list_org_files() -> Vec<DirEntry> {
    return WalkDir::new(".")
        .into_iter()
        .filter_entry(|e| is_org_file(e))
        .map(|e| e.unwrap())
        .collect();
}

fn extract_head(content: String) -> Option<String> {
    let header_re = Regex::new(r"^(\*+)\s+(TODO|DOING|DONE)?\s*(?P<title>.+)$").unwrap();
    if let Some(caps) = header_re.captures(&content) {
        return Some(caps["title"].to_string());
    }
    None
}

fn extract_scuedule_and_type(content: String) -> Option<Vec<(String, ScheduleType)>> {
    let schedule_re = Regex::new(r"(?P<type>SCHEDULED|DEADLINE):\s+<(?P<date>[^>]+)>").unwrap();
    // let mut schedules = Vec::new();
    let schedules: Vec<(String, ScheduleType)> = schedule_re
        .find_iter(&content)
        .map(|matches| {
            let caps = schedule_re.captures(matches.as_str()).unwrap();
            let scheduletype = match &caps["type"] {
                "SCHEDULED" => ScheduleType::Schedule,
                "DEADLINE" => ScheduleType::Deadline,
                _ => panic!("Unexpected schedule type"),
            };
            (caps["date"].to_string(), scheduletype)
        })
        .collect();
    // if let Some(caps) = schedule_re.captures(&content) {
    //     let scheduletype = match &caps["type"] {
    //         "SCHEDULED" => ScheduleType::Schedule,
    //         "DEADLINE" => ScheduleType::Deadline,
    //         _ => return None,
    //     };
    //     schedules.push((caps["date"].to_string(), scheduletype));
    // }
    if schedules.is_empty() {
        return None;
    } else {
        return Some(schedules);
    }
}

fn parse_schedules(content: String) -> Vec<Schedule> {
    return vec![];
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extract_head_with_status_todo() {
        let sample_input = "* TODO Sample Task".to_string();
        let expected = Some("Sample Task".to_string());
        assert_eq!(extract_head(sample_input), expected);
    }
    #[test]
    fn test_extract_head_with_status_doing() {
        let sample_input = "* DOING Sample Task".to_string();
        let expected = Some("Sample Task".to_string());
        assert_eq!(extract_head(sample_input), expected);
    }
    #[test]
    fn test_extract_head_with_status_done() {
        let sample_input = "* DONE Sample Task".to_string();
        let expected = Some("Sample Task".to_string());
        assert_eq!(extract_head(sample_input), expected);
    }
    #[test]
    fn test_extract_head_with_status_mulitple_bullet() {
        let sample_input = "*** TODO Sample Task".to_string();
        let expected = Some("Sample Task".to_string());
        assert_eq!(extract_head(sample_input), expected);
    }
    #[test]
    fn test_extract_head_with_status_none() {
        let sample_input = " TODO Sample Task".to_string();
        assert_eq!(extract_head(sample_input), None);
    }

    #[test]
    fn test_extract_schedule_and_type_schedule() {
        let sample_input = "SCHEDULED: <2024-06-01 Sat 10:00>".to_string();
        let expected = Some(vec![(
            "2024-06-01 Sat 10:00".to_string(),
            ScheduleType::Schedule,
        )]);
        assert_eq!(extract_scuedule_and_type(sample_input), expected);
    }
    #[test]
    fn test_extract_schedule_and_type_deadline() {
        let sample_input = "DEADLINE: <2024-06-01 Sat 10:00>".to_string();
        let expected = Some(vec![(
            "2024-06-01 Sat 10:00".to_string(),
            ScheduleType::Deadline,
        )]);
        assert_eq!(extract_scuedule_and_type(sample_input), expected);
    }

    #[test]
    fn test_extract_schedule_and_type_both() {
        let sample_input =
            "SCHEDULED: <2026-06-05 Fri> DEADLINE: <2026-06-05 Fri 21:00>".to_string();
        let expected = Some(vec![
            ("2026-06-05 Fri".to_string(), ScheduleType::Schedule),
            ("2026-06-05 Fri 21:00".to_string(), ScheduleType::Deadline),
        ]);
        assert_eq!(extract_scuedule_and_type(sample_input), expected);
    }
    #[test]
    fn test_parse_scuedules_scuedule() {
        let sample_input = "* Test Schedule\nSCHEDULED: <2024-06-01 Sat 10:00>".to_string();
        let expected = vec![Schedule {
            head: "Test Schedule".to_string(),
            schedule: "2024-06-01 Sat 10:00".to_string(),
            scheduletype: ScheduleType::Schedule,
        }];
        assert_eq!(parse_schedules(sample_input), expected);
    }
}
