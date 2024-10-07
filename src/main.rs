use derive_new::new;
use dotenvy::dotenv_override;
use dotenvy::var;
use icalendar::Calendar;
use icalendar::CalendarComponent;
use icalendar::Component;
use icalendar::Event;
use regex::Regex;
use serde::*;
use std::fs::File;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::i64;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

type GenResult<T> = Result<T, Box<dyn std::error::Error>>;

struct MipCalendarComplete {
    metadata: MipCalendar,
    event: Event,
}

impl MipCalendarComplete {
    fn new(calendar_entry: &CalendarComponent) -> Result<Self, ()> {
        let event = match calendar_entry.as_event() {
            Some(x) => x.clone(),
            None => return Err(()),
        };
        let metadata = MipCalendar::new(&event);
        Ok(Self { event, metadata })
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct MipCalendar {
    hash: i128,
    uid: String,
}

impl MipCalendar {
    fn new(entry: &Event) -> Self {
        let mut hasher = DefaultHasher::new();
        let mut hash_text = format!("{:?}", entry.get_start().unwrap());
        hash_text.push_str(&format!("{:?}", entry.get_end().unwrap()));
        hash_text.push_str(entry.get_summary().unwrap());
        hash_text.hash(&mut hasher);
        let uid = entry.get_uid().unwrap_or("").to_string();
        let hash = hasher.finish() as i128 - i64::MAX as i128;
        Self { hash, uid }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MipCalendars {
    mip_calendars: Vec<MipCalendar>,
}

impl MipCalendars {
    fn new(mip_calendars: Vec<MipCalendar>) -> Self {
        Self { mip_calendars }
    }
}

#[derive(new, Debug)]
struct CalendarInfo {
    link: String,
    username: Option<String>,
    password: Option<String>,
}

fn parse_calendar_env() -> Vec<CalendarInfo> {
    let mut calendar_list: Vec<CalendarInfo> = vec![];
    let mut i = 0;
    let mut go_on = true;
    while go_on {
        if let Ok(calendar_name) = var(format!("calendar_{}", i)) {
            let user_name = var(format!("username_{}", i)).ok();
            let password_name = var(format!("password_{}", i)).ok();
            calendar_list.push(CalendarInfo::new(calendar_name, user_name, password_name));
            i += 1;
        } else {
            go_on = false;
        }
    }
    calendar_list
}

#[tokio::main]
async fn main() {
    dotenv_override().ok();
    let calendar_list = parse_calendar_env();
    let events = load_new_entries(calendar_list).await.unwrap();
    save_events_to_disk(&events).await.unwrap();
    serialise_and_save(&events).await.unwrap();
}

async fn load_new_entries(calendars: Vec<CalendarInfo>) -> GenResult<Vec<MipCalendarComplete>> {
    let mut mip_entries: Vec<MipCalendarComplete> = vec![];
    for calendar in calendars {
        let entries = load_calendar(calendar).await?;
        let calendar_name = get_calendar_name(entries.get_name().unwrap_or("Agenda (default)"));
        println!("{}", calendar_name);
        for entry in entries.iter() {
            if let Ok(mip_entry) = MipCalendarComplete::new(entry) {
                let mut mip_entry = mip_entry;
                add_ids_to_description(&mut mip_entry);
                add_category(&calendar_name, &mut mip_entry);
                mip_entry.event.uid("");
                mip_entry.event.add_property("TRANSP", "");
                mip_entry.event.add_property("RRULE", "");
                mip_entries.push(mip_entry)
            }
        }
    }
    Ok(mip_entries)
}

async fn load_calendar(calendar_url: CalendarInfo) -> GenResult<Calendar> {
    let client = reqwest::Client::new();
    let mut response = client.get(calendar_url.link);
    if let Some(username) = calendar_url.username {
        response = response.basic_auth(username, calendar_url.password);
    }
    let ics_data = response.send().await?.text().await?;
    let ics_data_test = icalendar::parser::unfold(&ics_data);
    let parsed_calendar = icalendar::parser::read_calendar(&ics_data_test)?;
    Ok(parsed_calendar.into())
}

fn get_calendar_name(input: &str) -> String {
    // let re = Regex::new(r"\(([^)]+)\)").unwrap();
    // if let Some(captures) = re.captures(input) {
    //     captures[1].to_string()
    // } else {
    //     "Default".to_string()
    // }
    input.to_string()
}

fn add_ids_to_description(entry: &mut MipCalendarComplete) {
    let mut entry_description = entry.event.get_description().unwrap_or("").to_lowercase();
    if !entry_description.is_empty() {
        entry_description.push_str("\\n");
    }
    entry_description.push_str(&format!(
        "do not edit next line:\\n{}",
        entry.event.get_uid().unwrap_or("no UID")
    ));
    entry.event.description(&entry_description);
}

fn add_category(calendar_name: &str, mip_calendar: &mut MipCalendarComplete) {
    mip_calendar
        .event
        .add_property("CATEGORIES", format!("{}", calendar_name));
}

async fn save_events_to_disk(events: &Vec<MipCalendarComplete>) -> GenResult<()> {
    let path = var("save_location").expect("No save location set");
    let mut output_check = File::create(format!("{}/ics2pda", path)).unwrap();
    write!(
        output_check,
        "This folder is used to store calendar entries to sync with a WM PDA",
    )?;
    let mut i = 0;
    for event in events {
        let mut calendar = Calendar::new();
        calendar.push(event.event.clone());
        let mut calendar_disp = format!("{}", calendar);
        calendar_disp = calendar_disp.replace("UID:", "flats");
        calendar_disp = calendar_disp.replace("TRANSP:", "flats");
        calendar_disp = calendar_disp.replace("RRULE:", "flats");
        let mut test = calendar_disp
            .lines()
            .filter(|line| line.trim() != "flats")
            .collect::<Vec<&str>>()
            .join("\n");
        test = test.replace("\\n", r#"\r\n"#);
        test = test.replace(r"\\r", "");
        i += 1;
        let hash = (i).to_string();
        let mut output = File::create(format!("{}/{}", path, hash)).unwrap();
        write!(output, "{}", test)?;
    }
    Ok(())
}

// async fn remove_previous_entries(path: &Path) -> GenResult<()> {
//     let test_file_path = Path::new(&format!("{}/{}", path.display(), "ics2pda"));
//     dbg!(test_file_path);
//     Ok(())
// }

async fn serialise_and_save(events: &Vec<MipCalendarComplete>) -> GenResult<()> {
    Ok(())
}
