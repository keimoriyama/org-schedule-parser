use std::fs::File;
use walkdir::{DirEntry, WalkDir};

enum ScheduleType {
    Deadline,
    Schedule,
}

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

fn parse_schedules(content: String) -> Vec<Schedule> {
    let mut schedules = Vec::new();
    let mut current_head = String::new();

    for line in content.lines() {
        if line.starts_with("* ") {
            current_head = line[2..].to_string();
        } else if line.contains("SCHEDULED: <") {
            if let Some(start) = line.find("SCHEDULED: <") {
                if let Some(end) = line[start..].find('>') {
                    let schedule_str = &line[start + 12..start + end];
                    schedules.push(Schedule {
                        head: current_head.clone(),
                        schedule: schedule_str.to_string(),
                        scheduletype: ScheduleType::Schedule,
                    });
                }
            }
        } else if line.contains("DEADLINE: <") {
            if let Some(start) = line.find("DEADLINE: <") {
                if let Some(end) = line[start..].find('>') {
                    let schedule_str = &line[start + 11..start + end];
                    schedules.push(Schedule {
                        head: current_head.clone(),
                        schedule: schedule_str.to_string(),
                        scheduletype: ScheduleType::Deadline,
                    });
                }
            }
        }
    }

    schedules
}

#[cfg(test)]
mod tests {
    use super::*;
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
