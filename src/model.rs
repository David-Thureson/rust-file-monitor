use std::collections::BTreeMap;
use std::thread;
use chrono::{NaiveDateTime, Duration};
use util::format::format_count;
use crate::summary::Summary;

const SECONDS_PAUSE: i64 = 30;
pub const PATH_HISTORY: &str = "C:/File Monitor History";
const NAME_DOKUWIKI: &str = "DokuWiki";
const PATH_DOKUWIKI: &str = "C:/Doku/DokuWikiStick/dokuwiki/data/pages";
const SUBFOLDERS_DOKUWIKI: [&str; 2] = ["tools", "tools/nav"];
// const FILE_NAME_HISTORY: &str = "history.json";
#[allow(dead_code)]
pub const FILE_NAME_SUMMARY: &str = "summary.json";
const FILE_NAME_MARKER_PAUSE: &str = "pause_marker.txt";
const FILE_NAME_MARKER_GEN: &str = "gen_marker.txt";

pub struct Model {
    pub path_history: String,
    pub projects: BTreeMap<String, Project>,
}

#[derive(Clone)]
pub struct Project {
    pub name: String,
    pub path_root: String,
    pub path_history: String,
    pub subfolders: Vec<String>,
    pub minutes: f32,
}

pub enum Marker {
    Gen,
    Pause,
}

impl Model {
    pub(crate) fn new(path_history: &str) -> Self {
        Self {
            path_history: path_history.to_string(),
            projects: Default::default(),
        }
    }

    pub(crate) fn add_project(&mut self, project: Project) {
        let key = project.name.clone();
        assert!(!self.projects.contains_key(&key));
        self.projects.insert(key, project);
    }

    pub fn run(&self) {
        util::file::path_create_if_necessary_r(&self.path_history).unwrap();
        let mut handles = vec![];
        for project in self.projects.values() {
            let project = project.clone();
            let handle = thread::spawn(move || { project.run(); });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }
    }
}

impl Project {
    pub fn new(name: &str, path_root: &str, path_history: &str, subfolders: Vec<String>, minutes: f32) -> Self {
        Self {
            name: name.to_string(),
            path_root: path_root.to_string(),
            path_history: path_history.to_string(),
            subfolders,
            minutes,
        }
    }

    pub(crate) fn run(&self) {
        let seconds = (self.minutes * 60.0) as i64;
        self.print_message(&format!("run() - start: seconds = {}", format_count(seconds)));
        assert!(util::file::path_exists(&self.path_root));
        util::file::path_create_if_necessary_r(&self.path_history).unwrap();
        loop {
            let next_scan_time = now().checked_add_signed(Duration::seconds(seconds)).unwrap();
            self.run_scan(&next_scan_time);
            while now() < next_scan_time {}
        }
    }

    fn run_scan(&self, next_scan_time: &NaiveDateTime) {
        self.print_message("run_scan() - start");
        loop {
            if self.is_marker_present(&Marker::Pause) {
                let next_try_time = now().checked_add_signed(Duration::seconds(SECONDS_PAUSE)).unwrap();
                self.print_message(&format!("run_scan() - paused: next try at {}", format_time(&next_try_time)));
                while now() < next_try_time {}
            } else {
                break;
            }
        }

        Summary::scan(&self, self.is_marker_present(&Marker::Gen));
        self.clear_marker(&Marker::Gen);

        self.print_message(&format!("run_scan() - done: next scan at {}", format_time(next_scan_time)));
    }

    pub fn set_marker(&self, marker: &Marker) {
        util::file::write_file_r(&self.get_marker_file_name(marker), "marker").unwrap();
    }

    pub fn clear_marker(&self, marker: &Marker) {
        let path = self.get_marker_file_name(marker);
        if util::file::path_exists(&path) {
            std::fs::remove_file(path).unwrap();
        }
    }

    pub fn is_marker_present(&self, marker: &Marker) -> bool {
        let path = self.get_marker_file_name(marker);
        util::file::path_exists(&path)
    }

    fn get_marker_file_name(&self, marker: &Marker) -> String {
        format!("{}/{}", self.path_history, marker.get_file_name())
    }

    fn print_message(&self, msg: &str) {
        println!("{}: {}: {}", format_date_time(&now()), self.name, msg);
    }
}

impl Marker {
    pub fn get_file_name(&self) -> &str {
        match self {
            Self::Gen => FILE_NAME_MARKER_GEN,
            Self::Pause => FILE_NAME_MARKER_PAUSE,
        }
    }
}

pub fn set_up_model(minutes: f32) -> Model {
    let mut model = Model::new(PATH_HISTORY);
    model.add_project(set_up_project(NAME_DOKUWIKI, minutes));
    model
}

pub fn set_up_project(project_name: &str, minutes: f32) -> Project {
    match project_name {
        NAME_DOKUWIKI => Project::new(NAME_DOKUWIKI, PATH_DOKUWIKI, &format!("{}/{}", PATH_HISTORY, NAME_DOKUWIKI), SUBFOLDERS_DOKUWIKI.iter().map(|x| x.to_string()).collect(), minutes),
        _ => panic!("Unexpected project name = \"{}\".", project_name),
    }
}

fn format_date_time(time: &NaiveDateTime) -> String {
    util::date_time::naive_date_time_to_seconds_format(time)
}

fn format_time(time: &NaiveDateTime) -> String {
    util::date_time::naive_date_time_to_seconds_format_time_only(time)
}

fn now() -> NaiveDateTime {
    util::date_time::naive_date_time_now()
}