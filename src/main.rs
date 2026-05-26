use chrono::NaiveDate;
use log::{info, warn};
use regex::Regex;
use std::{fs::File, io::Read};
use walkdir::{DirEntry, WalkDir};

#[derive(PartialEq, Debug)]
enum ScheduleType {
    Deadline,
    Schedule,
}

#[derive(PartialEq, Debug)]
struct Schedule {
    head: String,
    schedule: NaiveDate,
    scheduletype: ScheduleType,
}
fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let base_path = "../../keimoriyama/org-files/";
    let org_files = list_org_files(base_path);
    let mut schedules: Vec<Schedule> = vec![];
    for entry in org_files {
        if !is_org_file(&entry) {
            warn!("Skipping non-org file: {:?}", entry.path());
            continue;
        }
        if let Some(file) = open_org_file(&entry) {
            info!("Opened file: {:?}", entry.path());
            info!("File content: {:?}", file);
            // read file content
            let mut content = String::new();
            if let Err(e) = file.take(10_000).read_to_string(&mut content) {
                warn!("Failed to read file {:?}: {}", entry.path(), e);
                continue;
            }
            info!("File content: {:?}", content);
            let schedule = parse_schedules(content);
            info!(
                "Extracted schedules from file {:?}: {:?}",
                entry.path(),
                schedule
            );
            schedules.extend(schedule);
        }
    }
}

fn open_org_file(entry: &DirEntry) -> Option<File> {
    File::open(entry.path()).ok()
}

fn extract_org_head_and_schedules(content: String) -> Vec<Schedule> {
    let mut schedules = Vec::new();

    let mut current_head: Option<String> = None;

    for line in content.lines() {
        if let Some(head) = extract_head(line.to_string()) {
            current_head = Some(head)
        } else if let Some(caps) = extract_schedule_and_type(line.to_string()) {
            if let Some(head) = &current_head {
                for (schedule, scheduletype) in caps {
                    schedules.push(Schedule {
                        head: head.clone(),
                        schedule,
                        scheduletype,
                    });
                }
            }
        }
    }
    return schedules;
}

fn is_org_file(entry: &DirEntry) -> bool {
    info!("Checking if entry is an org file: {:?}", entry.path());
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".org") || s.contains(".git"))
        .unwrap_or(false)
}

fn list_org_files(path: &str) -> Vec<DirEntry> {
    return WalkDir::new(path).into_iter().map(|e| e.unwrap()).collect();
}

fn extract_head(content: String) -> Option<String> {
    let header_re = Regex::new(r"^(\*+)\s+(TODO|DOING|DONE)?\s*(?P<title>.+)$").unwrap();
    if let Some(caps) = header_re.captures(&content) {
        return Some(caps["title"].to_string());
    }
    None
}

fn extract_schedule_and_type(content: String) -> Option<Vec<(NaiveDate, ScheduleType)>> {
    let schedule_re = Regex::new(
        r"(?P<type>SCHEDULED|DEADLINE):\s+<(?P<date>\d{4}-\d{2}-\d{2}\s+\w{3}(?:\s+\d{2}:\d{2})?)(?:\s+[^>]*)?>",
    )
    .unwrap();
    let schedules: Vec<(NaiveDate, ScheduleType)> = schedule_re
        .find_iter(&content)
        .map(|matches| {
            let caps = schedule_re.captures(matches.as_str()).unwrap();
            let scheduletype = match &caps["type"] {
                "SCHEDULED" => ScheduleType::Schedule,
                "DEADLINE" => ScheduleType::Deadline,
                _ => panic!("Unexpected schedule type"),
            };
            info!("caps for schedule: {:?}", caps);
            let date_format = if caps["date"].contains(':') {
                "%Y-%m-%d %a %H:%M"
            } else {
                "%Y-%m-%d %a"
            };
            let date = NaiveDate::parse_from_str(&caps["date"], date_format);
            info!("date parse result: {:?}", date);
            match date {
                Ok(d) => (d, scheduletype),
                Err(e) => panic!("Failed to parse date: {}", e),
            }
            //            (date, scheduletype)
        })
        .collect();
    if schedules.is_empty() {
        return None;
    } else {
        return Some(schedules);
    }
}

fn parse_schedules(content: String) -> Vec<Schedule> {
    for content in content.lines() {
        let li = content.to_string();
        info!("Parsing line: {:?}", li);
        let head = extract_head(li.clone());
        let schedule_and_type = extract_schedule_and_type(li.clone());
        if head.is_none() || schedule_and_type.is_none() {
            continue;
        }
        info!(
            "Extracted head: {:?}, schedule and type: {:?}",
            head, schedule_and_type
        );
    }
    return vec![];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_schedule_and_type_schedule() {
        let sample_input = "SCHEDULED: <2024-06-01 Sat 10:00>".to_string();
        let expected = Some(vec![(
            NaiveDate::parse_from_str(r"2024-06-01 Sat 10:00", "%Y-%m-%d %a %H:%M").unwrap(),
            ScheduleType::Schedule,
        )]);
        assert_eq!(extract_schedule_and_type(sample_input), expected);
    }
    #[test]
    fn test_extract_schedule_and_type_schedule_without_time() {
        let sample_input = "SCHEDULED: <2024-06-01 Sat>".to_string();
        let expected = Some(vec![(
            NaiveDate::parse_from_str(r"2024-06-01 Sat", "%Y-%m-%d %a").unwrap(),
            ScheduleType::Schedule,
        )]);
        assert_eq!(extract_schedule_and_type(sample_input), expected);
    }

    #[test]
    fn test_extract_schedule_and_type_deadline() {
        let sample_input = "DEADLINE: <2024-06-01 Sat 10:00>".to_string();
        let expected = Some(vec![(
            NaiveDate::parse_from_str("2024-06-01 Sat 10:00", "%Y-%m-%d %a %H:%M").unwrap(),
            ScheduleType::Deadline,
        )]);
        assert_eq!(extract_schedule_and_type(sample_input), expected);
    }

    #[test]
    fn test_extract_schedule_and_type_deadline_plus_one() {
        let sample_input = "DEADLINE: <2026-05-13 Wed 10:00 +1w>".to_string();
        let expected = Some(vec![(
            NaiveDate::parse_from_str("2026-05-13 Wed 10:00", "%Y-%m-%d %a %H:%M").unwrap(),
            ScheduleType::Deadline,
        )]);
        assert_eq!(extract_schedule_and_type(sample_input), expected)
    }

    #[test]
    fn test_extract_schedule_and_type_both() {
        let sample_input =
            "SCHEDULED: <2026-06-05 Fri> DEADLINE: <2026-06-05 Fri 21:00>".to_string();
        let expected = Some(vec![
            (
                NaiveDate::parse_from_str("2026-06-05 Fri", "%Y-%m-%d %a").unwrap(),
                ScheduleType::Schedule,
            ),
            (
                NaiveDate::parse_from_str("2026-06-05 Fri 21:00", "%Y-%m-%d %a %H:%M").unwrap(),
                ScheduleType::Deadline,
            ),
        ]);
        assert_eq!(extract_schedule_and_type(sample_input), expected);
    }
    #[test]
    fn test_parse_scuedules_scuedule() {
        let sample_input = "* Test Schedule\nSCHEDULED: <2024-06-01 Sat 10:00>\n * Test1\nDEADLINE: <2024-06-01 Sat>".to_string();
        let expected = vec![
            Schedule {
                head: "Test Schedule".to_string(),
                schedule: NaiveDate::parse_from_str("2024-06-01 Sat 10:00", "%Y-%m-%d %a %H:%M")
                    .unwrap(),
                scheduletype: ScheduleType::Schedule,
            },
            Schedule {
                head: "Test Schedule".to_string(),
                schedule: NaiveDate::parse_from_str("2024-06-01 Sat", "%Y-%m-%d %a").unwrap(),
                scheduletype: ScheduleType::Deadline,
            },
        ];
        assert_eq!(extract_org_head_and_schedules(sample_input), expected);
    }

    #[test]
    fn test_extract_head_and_schedules() {
        let sample_input = "* Test Schedule\nSCHEDULED: <2024-06-01 Sat 10:00>".to_string();
        let expected = vec![Schedule {
            head: "Test Schedule".to_string(),
            schedule: NaiveDate::parse_from_str("2024-06-01 Sat 10:00", "%Y-%m-%d %a %H:%M")
                .unwrap(),
            scheduletype: ScheduleType::Schedule,
        }];
        assert_eq!(extract_org_head_and_schedules(sample_input), expected);
    }
}
