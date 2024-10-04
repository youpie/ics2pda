use dotenvy::dotenv_override;
use dotenvy::var;
use icalendar::Calendar;
use icalendar::CalendarComponent;
use icalendar::Component;
use icalendar::Event;

type GenResult<T> = Result<T, Box<dyn std::error::Error>>;

struct MipCalendar {
    hash: i128,
    filename: String,
    calendar_entry: icalendar::Calendar,
}

// impl MipCalendar {
//     fn new() -> GenResult<Self> {
//         Ok(Self {})
//     }
// }

fn parse_calendar_env(env_string: String) -> Vec<String> {
    let string_split: Vec<&str> = env_string.split(';').collect();
    let owned_split: Vec<String> = string_split.into_iter().map(|x| x.to_owned()).collect();
    owned_split
}

#[tokio::main]
async fn main() {
    dotenv_override().ok();
    let calendar_string = var("calendar_list").unwrap();
    let calendar_list = parse_calendar_env(calendar_string);
    load_new_entries(calendar_list).await.unwrap();
}

async fn load_new_entries(calendars: Vec<String>) -> GenResult<()> {
    for calendar in calendars {
        let entries = load_calendar(calendar).await?;
        let mut mip_entries: Vec<MipCalendar> = vec![];
        for entry in entries.iter() {
            add_ids_to_description(entry);
            // add_category();
            // create_hash();
            // generate_filenames();
            // save_to_disk();
        }
    }
    Ok(())
}

async fn load_calendar(calendar_url: String) -> GenResult<Calendar> {
    // dbg!(&calendar_url);
    let mut ics_data = reqwest::get(calendar_url).await?.text().await?;
    //ics_data = ics_data.replace(" > ", ",");
    let ics_data_test = icalendar::parser::unfold(&ics_data);
    let parsed_calendar = icalendar::parser::read_calendar(&ics_data_test)?;
    Ok(parsed_calendar.into())
}

fn add_ids_to_description(entry: &CalendarComponent) -> GenResult<()> {
    let entry = match entry.as_event() {
        Some(x) => x,
        None => return Ok(()),
    };
    let mut entry_description = entry.get_description().unwrap_or("").to_lowercase();
    entry_description.push_str(&format!(
        "\nDO NOT EDIT NEXT LINE:\n{}",
        entry.get_uid().unwrap_or("no UID")
    ));
    println!("entry description: {}", entry_description);
    Ok(())
}

fn add_category() {}

fn create_hash() {}

fn generate_filenames() {}

fn save_to_disk() {}
