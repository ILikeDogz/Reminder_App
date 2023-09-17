use std::fmt;
use eframe::{
    egui::{self, Visuals},
    run_native,
};
use chrono::NaiveTime;
use egui::{vec2, ScrollArea, TopBottomPanel, Window, FontId, RichText, ComboBox};
use reminder_api::{Reminder, ReminderList};

const PADDING: f32 = 5.0;

#[derive(PartialEq, Eq)]
enum Theme {
    Light,
    Dark,
}
impl fmt::Display for Theme {
    fn fmt(&self, format_condition: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Theme::Dark => write!(format_condition, "Dark Mode"),
            Theme::Light => write!(format_condition, "Light Mode"),
        }
    }
}
impl Theme {
    //at the moment only swaps themes but could be changed later for additional themes
    fn change_theme(&mut self) {
        *self = match *self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        };
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(420.0, 600.0)),
        ..Default::default()
    };
    run_native(
        "Reminder App (WIP)",                    //name of window
        options,                                 //controls window size
        Box::new(|_cc| Box::<ReminderGui>::default()), //essentially contains the app, allowing it to exist on the heap
    )
}
struct ReminderGui {
    theme: Theme,
    reminder_window_status: bool,
    reminder_list: ReminderList,
    reminder: Reminder,
    hours: u32,
    minutes: u32,
}
impl Default for ReminderGui {
    fn default() -> Self {
        Self {
            theme: Theme::Dark, //true is dark mode (default state)
            reminder_window_status: false,
            reminder_list: ReminderList::default(),
            reminder: Reminder::default(),
            hours: 0,
            minutes: 0,
        }
    }
}
impl ReminderGui {
    //any method related to ui, is in ReminderGui
    fn render_switch_theme_button(&mut self, ui: &mut egui::Ui) {
        if ui.button(format!("Switch {}", &self.theme)).clicked() {
            self.theme.change_theme(); //swaps theme to opposite (dark/light)
        }
    }
    fn render_theme(&mut self, ctx: &egui::Context) {
        match self.theme {
            Theme::Dark => {
                ctx.set_visuals(Visuals::dark());
            }
            Theme::Light => {
                ctx.set_visuals(Visuals::light());
            }
        }
    }
    fn render_reminder_list(&mut self, ui: &mut eframe::egui::Ui) {
        //in order for the delete button to work, self.reminder_list must be open to be mutally borrowed, thus it gets cloned
        for item in self.reminder_list.clone().list() {
            ui.separator();
            ui.label(RichText::new(format!("{}", item.title)).font(FontId::proportional(16.0)));
            ui.label(format!("{}", item.description));
            ui.label(format!("Date: {}", item.date));
            ui.label(format!("Time: {}", item.time));
            ui.label(format!("Notify: {} hour(s) before", item.notify_when));
            ui.add_space(PADDING);
            if ui.button("Delete Reminder").clicked(){
                self.reminder_list.delete_reminder(&item);
                self.reminder_list.save_list_to_json(true);
            }
        }
    }
    fn render_time_controls(&mut self, ui: &mut eframe::egui::Ui){
        ui.horizontal(|ui|{
            ui.label("Hour:");
            //creates selectable boxes for hour and minute
            ComboBox::from_label("Minute:").width(50.0)
                .selected_text(format!("{}", self.hours))
                .show_ui(ui, |ui| {
                    for hour in 0..=23{
                        ui.selectable_value(&mut self.hours, hour, format!("{}", hour));
                    }
                }
            );
            ComboBox::from_label("").width(50.0)
                .selected_text(format!("{}", self.minutes))
                .show_ui(ui, |ui| {
                    for minute in 0..=59{
                        ui.selectable_value(&mut self.minutes, minute, format!("{}", minute));
                    }
                }
            );
        });
        ui.horizontal(|ui| {
            ui.label("Notify:");
            ComboBox::from_label("hour(s) before").width(50.0)
            .selected_text(format!("{}", self.reminder.notify_when))
            .show_ui(ui, |ui| {
                for hour in 0..=24{
                    ui.selectable_value(&mut self.reminder.notify_when, hour, format!("{}", hour));
                }
            });
        });
    }
    fn render_reminder_text_fields(&mut self, ui: &mut eframe::egui::Ui) {
        //summons editable text fields for reminder
        ui.horizontal(|ui| {
            //holds in horizontal
            //labeling that is probably useful
            let title_label = ui.label("Title:  \t\t\t");
            ui.text_edit_singleline(&mut self.reminder.title)
                .labelled_by(title_label.id);
        });
        ui.horizontal(|ui| {
            let description_label = ui.label("Description: ");
            ui.add(egui::TextEdit::multiline(&mut self.reminder.description))
                .labelled_by(description_label.id);
        });
        //in order to set the relative time for the date picker, I had to go into the crate to popup.rs and change the time variable
        ui.horizontal(|ui| {
            ui.label("Date:\t\t\t\t");
            //calls in the date picker button
            ui.add(egui_extras::DatePickerButton::new(&mut self.reminder.date));
        });
        self.render_time_controls(ui);
    }
    fn render_reminder_window(&mut self, ctx: &egui::Context) {
        let mut window_status = true;
        Window::new("Reminder")
            .vscroll(true)
            .fixed_size(vec2(400.0, 300.0))
            .open(&mut window_status)
            .show(ctx, |ui| {
                self.render_reminder_text_fields(ui);
                //magical date picker
                if !self.reminder.is_not_filled() && ui.button("save reminder").clicked() {
                    //saves time to reminder
                    self.reminder.time = self.parse_hours_minutes_to_naive_time();
                    self.reset_hours_minutes();
                    self.reminder_list.save_reminder(&self.reminder);
                    //clears actively held reminder
                    self.reminder = Reminder::new();
                    self.reminder_list.save_list_to_json(false);
                    self.reminder_window_status = false;
                    // add something to show confirmation that it was saved
                } else if self.reminder.is_not_filled() {
                    let _invalid_reminder_present = ui.button("incomplete reminder");
                }
            });
        if window_status == false {
            self.reminder_window_status = false;
        }
    }
    fn render_top_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.add_space(PADDING);
        //contains everything in a top panel
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // egui::menu::bar(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Reminder App");
                self.render_switch_theme_button(ui);
                if ui.button("Create Reminder").clicked() {
                    self.reminder_window_status = true;
                }
                if self.reminder_window_status == true {
                    self.render_reminder_window(ctx);
                }
            });
            // });
            ui.label("Powered by Hopes and Dreams");
            ui.add_space(PADDING);
        });
        //space so nothing goes into the top panel
        ui.add_space(40.0);
    }
    fn parse_hours_minutes_to_naive_time(&mut self) -> NaiveTime{
        NaiveTime::from_hms_opt(self.hours, self.minutes, 0).expect("failed to parse hours and minutes to time")
    }
    fn reset_hours_minutes(&mut self){
        self.hours = 0;
        self.minutes = 0;
    }
    fn handle_notifications(&mut self){
        self.reminder_list.check_for_notification();
        for item in self.reminder_list.clone().list(){
            if *item.should_notify() == true{
                self.reminder_list.send_notification(item);
            }
        }
    }
}
impl eframe::App for ReminderGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            //auto_shrink controls where the scroll is placed
            self.handle_notifications();
            self.render_top_panel(ui, ctx);
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    self.render_theme(ctx);
                    self.render_reminder_list(ui);
                });
        });
    }
}
