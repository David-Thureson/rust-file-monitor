use chrono::{NaiveDateTime, Duration};
use crate::model::{Project, FILE_NAME_SUMMARY, PATH_HISTORY};
use std::fs;
use std::collections::BTreeMap;
use serde::Deserialize;
use std::ops::Sub;
// use serde_json::Result;

const LABEL_ADD: &str = "Add";
const LABEL_EDIT: &str = "Edit";
const LABEL_GEN: &str = "Gen";

#[derive(serde::Serialize, Debug, Deserialize)]
pub struct Summary {
    pub project_name: String,
    pub scans: Vec<SummaryScan>,
    pub files: BTreeMap<String, MonitoredFile>,
}

#[derive(serde::Serialize, Debug, Deserialize)]
pub struct SummaryScan {
    pub time: NaiveDateTime,
    pub is_gen: bool,
    pub checked_file_count: usize,
    pub changed_file_count: usize,
}

#[derive(serde::Serialize, Debug, Deserialize)]
pub struct MonitoredFile {
    pub subfolder: String,
    pub name: String,
    pub time_added: Option<NaiveDateTime>,
    pub time_latest_edit: Option<NaiveDateTime>,
    pub time_latest_gen: Option<NaiveDateTime>,
    pub gen_count: usize,
    pub edit_count: usize,
}

impl Summary {
    fn new(project_name: &str) -> Self {
        Self {
            project_name: project_name.to_string(),
            scans: vec![],
            files: Default::default(),
        }
    }

    pub fn read_or_create(project: &Project) -> Self {
        let path_file = Self::get_summary_file_path(project);
        //bg!(&path_file);
        if util::file::path_exists(&path_file) {
            let data = util::file::read_file_to_string_r(&path_file).unwrap();
            let summary: Summary = serde_json::from_str(&data).unwrap();
            assert_eq!(summary.project_name, project.name);
            summary
        } else {
            Summary::new(&project.name)
        }
    }

    fn write(&self, project: &Project) {
        //bg!(&self.project_name, &self.scans.len(), &self.files.len());
        let path_file = Self::get_summary_file_path(project);
        //bg!(&path_file);
        let data = serde_json::to_string(self).unwrap();
        util::file::write_file_r(path_file, &data).unwrap();
    }

    pub fn scan(project: &Project, is_gen: bool) {
        // For now don't deserialize the Summary object. Just create one.
        // let mut summary = Summary::new(&project.name);
        let mut summary = Self::read_or_create(project);
        summary.scan_self(project, is_gen);
        summary.print_activity(120);
        summary.write(project);
        // dbg!(summary); panic!();
    }

    pub fn scan_self(&mut self, project: &Project, is_gen: bool) {
        assert_eq!(self.project_name, project.name);
        let previous_scan_time = self.scans.last().map(|scan| scan.time);
        let mut checked_file_count = 0;
        let mut changed_file_count = 0;
        for subfolder in project.subfolders.iter() {
            let path_subfolder = format!("{}/{}", project.path_root, subfolder);
            for dir_entry_result in fs::read_dir(path_subfolder).unwrap() {
                let dir_entry = dir_entry_result.as_ref().unwrap();
                if dir_entry.metadata().unwrap().is_file() {
                    let file_name = util::file::dir_entry_to_file_name(dir_entry);
                    let file_time = dir_entry.metadata().unwrap().modified().unwrap();
                    let file_time = util::date_time::systemtime_as_naive_date_time(&file_time);
                    checked_file_count += 1;
                    match previous_scan_time {
                        Some(previous_scan_time) => {
                            //if file_name.eq("algorithms.txt") { //bg!(previous_scan_time, file_time); }
                            if file_time > previous_scan_time {
                                changed_file_count += 1;
                                let key = MonitoredFile::make_key(subfolder, &file_name);
                                match self.files.get_mut(&key) {
                                    Some(file) => {
                                        if is_gen {
                                            file.time_latest_gen = Some(file_time);
                                            file.gen_count += 1;
                                        } else {
                                            file.time_latest_edit = Some(file_time);
                                            file.edit_count += 1;
                                        }
                                        //bg!(&file);
                                    },
                                    None => {
                                        let mut file = MonitoredFile::new(subfolder, &file_name);
                                        file.time_added = Some(file_time);
                                        self.files.insert(key, file);
                                    }
                                }
                            }
                        },
                        None => {
                            // This is the first scan for this project, so we wont record any edits
                            // or gens. Simply record that we know about this file.
                            let key = MonitoredFile::make_key(subfolder, &file_name);
                            self.files.insert(key, MonitoredFile::new(subfolder, &file_name));
                        }
                    }
                }
            }
        }
        let scan = SummaryScan::new(is_gen, checked_file_count, changed_file_count);
        //bg!(&scan);
        self.scans.push(scan);
    }

    fn get_summary_file_path(project: &Project) -> String {
        format!("{}/{}/{}", PATH_HISTORY, project.name, FILE_NAME_SUMMARY)
    }

    pub fn print_activity(&self, minutes_back: usize) {
        println!("\nfile-monitor::Summary object for \"{}\" project:", self.project_name);
        let time_cutoff = util::date_time::naive_date_time_now().sub(Duration::minutes(minutes_back as i64));
        let mut files = self.files.values()
            .filter(|file| file.get_time_latest().map_or(false, |(time, _is_gen)| time >= time_cutoff))
            .map(|file| (file.get_key(), file.get_time_latest().unwrap()))
            .collect::<Vec<_>>();
        files.sort_by_cached_key(|file| file.0.clone());
        for file in files.iter() {
            let (time, label) = file.1;
            println!("\t{}: {} {:?}", file.0, label, time);
        }
        println!();
    }

}

impl SummaryScan {
    pub fn new(is_gen: bool, checked_file_count: usize, changed_file_count: usize) -> Self {
        Self {
            time: util::date_time::naive_date_time_now(),
            is_gen,
            checked_file_count,
            changed_file_count,
        }
    }

}

impl MonitoredFile {
    pub fn new(subfolder: &str, name: &str) -> Self {
        Self {
            subfolder: subfolder.to_string(),
            name: name.to_string(),
            time_added: None,
            time_latest_edit: None,
            time_latest_gen: None,
            gen_count: 0,
            edit_count: 0
        }
    }

    pub fn make_key(subfolder: &str, name: &str) -> String {
        format!("{}/{}", subfolder, name)
    }

    pub fn get_key(&self) -> String {
        Self::make_key(&self.subfolder, &self.name)
    }

    pub fn get_time_latest(&self) -> Option<(NaiveDateTime, &'static str)> {
        match self.time_added {
            Some(time_add) => Some((time_add, LABEL_ADD)),
            None => {
                match (self.time_latest_edit, self.time_latest_gen) {
                    (Some(time_gen), Some(time_edit)) => if time_gen > time_edit {
                        Some((time_gen, LABEL_GEN))
                    } else {
                        Some((time_edit, LABEL_EDIT))
                    }
                    (Some(time_gen), None) => Some((time_gen, LABEL_GEN)),
                    (None, Some(time_edit)) => Some((time_edit, LABEL_EDIT)),
                    (None, None) => None,
                }
            },
        }
    }
}
