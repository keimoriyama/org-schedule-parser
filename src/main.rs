use clap::Parser;
use log::{info, warn};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::{BufWriter, Write};
use std::{fs::File, io::Read};
use walkdir::{DirEntry, WalkDir};

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
enum ScheduleType {
    Deadline,
    Schedule,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Schedule {
    head: String,
    schedule: String,
    time: Option<String>,
    scheduletype: ScheduleType,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct ArgParse {
    #[arg(short, long)]
    org_dir: String,
}

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let args = ArgParse::parse();
    let base_path = args.org_dir;
    let org_files = list_org_files(&base_path);
    let mut schedules: Vec<Schedule> = vec![];
    for entry in org_files {
        if !is_org_file(&entry) {
            warn!("Skipping non-org file: {:?}", entry.path());
            continue;
        }
        if let Some(mut file) = open_org_file(&entry) {
            let mut content = String::new();
            if let Err(e) = file.read_to_string(&mut content) {
                warn!("Failed to read file {:?}: {}", entry.path(), e);
                continue;
            }

            let schedule = parse_schedules(content);
            info!(
                "Extracted schedules from file {:?}: {:?}",
                entry.path(),
                schedule
            );
            schedules.extend(schedule);
        }
    }
    // 今日より前の日付を持つスケジュールをフィルタリング
    let today = chrono::Local::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();
    info!("extracted schedule before filtering: {:?}", schedules);
    let upcoming_schedules: Vec<Schedule> = schedules
        .into_iter()
        .filter(|s| s.schedule.as_str() >= today.as_str())
        .collect();

    let file = File::create("output.jsonl");
    match file {
        Ok(f) => f,
        Err(e) => {
            panic!("Failed to create output file: {}", e);
        }
    };
    // write result in jsonl
    let mut writer = BufWriter::new(File::create("output.jsonl").unwrap());
    for schedule in upcoming_schedules {
        let json = serde_json::to_string(&schedule).unwrap();
        if let Err(e) = writeln!(writer, "{}", json) {
            warn!("Failed to write schedule to output file: {}", e);
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
                for (schedule, time, scheduletype) in caps {
                    schedules.push(Schedule {
                        head: head.clone(),
                        schedule,
                        time,
                        scheduletype,
                    });
                }
            }
        }
    }
    return schedules;
}

fn is_org_file(entry: &DirEntry) -> bool {
    //info!("Checking if entry is an org file: {:?}", entry.path());
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".org") || s.contains(".git"))
        .unwrap_or(false)
}

fn list_org_files(path: &str) -> Vec<DirEntry> {
    info!("Listing org files in directory: {}", path);
    return WalkDir::new(path).into_iter().map(|e| e.unwrap()).collect();
}

fn extract_head(content: String) -> Option<String> {
    let header_re = Regex::new(r"^(\*+)\s+(TODO|DOING|DONE)?\s*(?P<title>.+)$").unwrap();
    if let Some(caps) = header_re.captures(&content) {
        return Some(caps["title"].to_string());
    }
    None
}

fn extract_schedule_and_type(
    content: String,
) -> Option<Vec<(String, Option<String>, ScheduleType)>> {
    let schedule_re = Regex::new(
        r"(?P<type>SCHEDULED|DEADLINE):\s+<(?P<date>\d{4}-\d{2}-\d{2})(?:\s+\w{3})?(?:\s+(?P<time>\d{2}:\d{2}))?(?:\s+[^>]*)?>",
    )
    .unwrap();
    let schedules: Vec<(String, Option<String>, ScheduleType)> = schedule_re
        .find_iter(&content)
        .map(|matches| {
            let caps = schedule_re.captures(matches.as_str()).unwrap();
            let scheduletype = match &caps["type"] {
                "SCHEDULED" => ScheduleType::Schedule,
                "DEADLINE" => ScheduleType::Deadline,
                _ => panic!("Unexpected schedule type"),
            };
            //info!("caps for schedule: {:?}", caps);
            let time = caps.name("time").map(|m| m.as_str().to_string());
            (caps["date"].to_string(), time, scheduletype)
        })
        .collect();
    if schedules.is_empty() {
        return None;
    } else {
        return Some(schedules);
    }
}

fn parse_schedules(content: String) -> Vec<Schedule> {
    let mut current_head: Option<String> = None;
    let mut result: Vec<Schedule> = vec![];
    for content in content.lines() {
        let li = content.to_string();
        if let Some(head) = extract_head(li.clone()) {
            current_head = Some(head);
        } else if let Some(schedule_and_type) = extract_schedule_and_type(li.clone()) {
            if current_head.is_none() {
                continue;
            }
            for schedule in schedule_and_type.iter() {
                result.push(Schedule {
                    head: current_head.clone().unwrap(),
                    schedule: schedule.0.clone(),
                    time: schedule.1.clone(),
                    scheduletype: schedule.2.clone(),
                })
            }
            info!(
                "Extracted head: {:?}, schedule and type: {:?}",
                current_head, schedule_and_type
            );
        }
    }
    return result;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_schedule_and_type_schedule() {
        let sample_input = "SCHEDULED: <2024-06-01 Sat 10:00>".to_string();
        let expected = Some(vec![(
            "2024-06-01".to_string(),
            Some("10:00".to_string()),
            ScheduleType::Schedule,
        )]);
        assert_eq!(extract_schedule_and_type(sample_input), expected);
    }
    #[test]
    fn test_extract_schedule_and_type_schedule_without_time() {
        let sample_input = "SCHEDULED: <2024-06-01 Sat>".to_string();
        let expected = Some(vec![(
            "2024-06-01".to_string(),
            None,
            ScheduleType::Schedule,
        )]);
        assert_eq!(extract_schedule_and_type(sample_input), expected);
    }

    #[test]
    fn test_extract_schedule_and_type_ignores_mismatched_weekday() {
        let sample_input = "SCHEDULED: <2025-12-15 Wed>".to_string();
        let expected = Some(vec![(
            "2025-12-15".to_string(),
            None,
            ScheduleType::Schedule,
        )]);
        assert_eq!(extract_schedule_and_type(sample_input), expected);
    }

    #[test]
    fn test_extract_schedule_and_type_deadline() {
        let sample_input = "DEADLINE: <2024-06-01 Sat 10:00>".to_string();
        let expected = Some(vec![(
            "2024-06-01".to_string(),
            Some("10:00".to_string()),
            ScheduleType::Deadline,
        )]);
        assert_eq!(extract_schedule_and_type(sample_input), expected);
    }

    #[test]
    fn test_extract_schedule_and_type_deadline_plus_one() {
        let sample_input = "DEADLINE: <2026-05-13 Wed 10:00 +1w>".to_string();
        let expected = Some(vec![(
            "2026-05-13".to_string(),
            Some("10:00".to_string()),
            ScheduleType::Deadline,
        )]);
        assert_eq!(extract_schedule_and_type(sample_input), expected)
    }

    #[test]
    fn test_extract_schedule_and_type_both() {
        let sample_input =
            "SCHEDULED: <2026-06-05 Fri> DEADLINE: <2026-06-05 Fri 21:00>".to_string();
        let expected = Some(vec![
            ("2026-06-05".to_string(), None, ScheduleType::Schedule),
            (
                "2026-06-05".to_string(),
                Some("21:00".to_string()),
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
                schedule: "2024-06-01".to_string(),
                time: Some("10:00".to_string()),
                scheduletype: ScheduleType::Schedule,
            },
            Schedule {
                head: "Test Schedule".to_string(),
                schedule: "2024-06-01".to_string(),
                time: None,
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
            schedule: "2024-06-01".to_string(),
            time: Some("10:00".to_string()),
            scheduletype: ScheduleType::Schedule,
        }];
        assert_eq!(extract_org_head_and_schedules(sample_input), expected);
    }

    #[test]
    fn test_parse_schedules_keeps_head_through_body_lines() {
        let sample_input =
            "* Test Schedule\nnotes before schedule\nSCHEDULED: <2024-06-01 Sat>".to_string();
        let expected = vec![Schedule {
            head: "Test Schedule".to_string(),
            schedule: "2024-06-01".to_string(),
            time: None,
            scheduletype: ScheduleType::Schedule,
        }];
        assert_eq!(parse_schedules(sample_input), expected);
    }
}
