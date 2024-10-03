type GenResult<T> = Result<T, Box<dyn std::error::Error>>;

struct mip_calendar {
    hash: i128,
    filename: String,
    calendar: icalendar::Calendar,
}

impl mip_calendar {
    fn new() -> GenResult<Self> {
        Self {}
    }
}

fn parse_calendar_env() {}

fn main() {
    parse_calendar_env();
    load_new_entries();
}

fn load_new_entries(calendars: Vec<String>) -> GenResult<()> {
    for calendar in calendars {
        let entries = load_calendar(calendar);
        for entry in entries {
            add_ids_to_description();
            add_category();
            create_hash();
            generate_filenames();
            save_to_disk();
        }
    }
}

fn load_calendar(calendars: String) {}

fn add_ids_to_description() {}

fn add_category() {}

fn create_hash() {}

fn generate_filenames() {}

fn save_to_disk() {}
