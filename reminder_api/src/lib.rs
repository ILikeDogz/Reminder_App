use std::{fs::OpenOptions, path::Path};
use chrono::{Local, NaiveDate, NaiveTime, Timelike, Duration, NaiveDateTime};
use serde_json::Value;
use notify_rust::{Notification, Timeout};

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Reminder {
    //I wanna remove pub to each other these later
    pub title: String,
    pub description: String,
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub notify_when: i64,
    should_notify: bool,
    did_notify: bool,
    // reminder_when: time,
}
impl Default for Reminder {
    fn default() -> Self {
        Reminder::new()
    }
}
impl Clone for Reminder {
    fn clone(&self) -> Self {
        let reminder = Reminder {
            title: self.title.clone(),
            description: self.description.clone(),
            date: self.date.clone(),
            time: self.time.clone(),
            notify_when: self.notify_when.clone(),
            should_notify: self.should_notify.clone(),
            did_notify: self.did_notify.clone(),
        };
        reminder
    }
}
impl Reminder {
    pub fn new() -> Reminder {
        //creates new
        Reminder {
            title: String::new(),
            description: String::new(),
            date: Local::now().date_naive(),
            time: Local::now().time(),
            notify_when: 0,
            should_notify: false,
            did_notify: false,
        }
    }
    pub fn is_not_filled(&self) -> bool {
        if self.title.is_empty() || self.description.is_empty() {
            return true;
        }
        false
    }
    pub fn should_notify(&self) -> &bool{
        &self.should_notify
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ReminderList {
    list: Vec<Reminder>,
}
impl Default for ReminderList {
    fn default() -> Self {
        Self {
            list: {
                let file_path = "output.json";
                if Path::new(&file_path).exists() {
                    //if file exists
                    //reads file to a String
                    let file_contents =
                        std::fs::read_to_string(&file_path).expect("Failed to read the file.");
                    if file_contents.is_empty() {
                        //incase for some reason file exists and is empty, list has to be a vector, otherwise nothing can be added to it, and everything falls apart
                        Vec::new()
                    } else {
                        //reading json and making list the resulting vector
                        //converts JSON String to a JSON Value
                        let parsed_json: Value =
                            serde_json::from_str(&file_contents).expect("Failed to parse JSON");
                        //deserializes the JSON value into the Vector to fill out the type of list
                        serde_json::from_value(parsed_json)
                            .expect("Failed to deserialize JSON into struct")
                    }
                } else {
                    //list has to be set as a vector, otherwise nothing can be added to it, and everything falls apart
                    Vec::new()
                }
            },
        }
    }
}
impl Clone for ReminderList {
    fn clone(&self) -> Self {
        Self {
            list: self.list.clone(),
        }
    }
}
impl ReminderList {
    pub fn list(&self) -> &Vec<Reminder> {
        //provides read access to the list, without making field public
        &self.list
    }
    pub fn save_reminder(&mut self, reminder: &Reminder) {
        //creates a new reminder by copying
        let new_reminder = reminder.clone();
        
        self.list.push(new_reminder);
    }
    pub fn delete_reminder(&mut self, reminder: &Reminder){
        //iterates to position in list is equal to the reminder, then deletes what is at that point
        self.list.remove(self.list.iter().position(|x| *x == *reminder).expect("not found"));
    }
    pub fn save_list_to_json(&mut self, truncate: bool) {
        let file_path = "output.json";
        let file = OpenOptions::new() //Opens file
            //grants file permissions when opening
            .write(true)
            .create(true)
            .truncate(truncate)
            .open(file_path)
            .expect("Failed to open JSON file for writing");
        serde_json::to_writer(&file, &self.list).expect("Failed to write JSON");
        //writes to json
    }
    pub fn check_for_notification(&mut self){
        for item in &mut self.list{
            let time = NaiveDateTime::new(item.date, item.time) - Duration::hours(item.notify_when);
            if time.hour() == Local::now().hour() && time.minute() == Local::now().minute() && time.date() == Local::now().date_naive() && item.did_notify == false{
                item.should_notify = true;
            }
        }
    }
    pub fn send_notification(&mut self, reminder: &Reminder){
        for item in &mut self.list{
            if item == reminder{
                item.did_notify = true;
                item.should_notify = false;
                Notification::new()
                .summary(&item.title)
                .body(&format!("{}\nDate: {}\nTime: {}",item.description, item.date, item.time))
                .icon("wow")
                .timeout(Timeout::Milliseconds(6000)) //milliseconds
                .show().unwrap();
            }
        }
        self.save_list_to_json(true);
    }
}

