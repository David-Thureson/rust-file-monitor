use chrono::NaiveDateTime;
use crate::model::Project;
use std::fs;
use std::collections::BTreeMap;

#[derive(serde::Serialize, Debug)]
pub struct Summary {
    project_name: String,
    scans: Vec<SummaryScan>,
    files: BTreeMap<String, MonitoredFile>,
}

#[derive(serde::Serialize, Debug)]
pub struct SummaryScan {
    time: NaiveDateTime,
    is_gen: bool,
    checked_file_count: usize,
    changed_file_count: usize,
}

#[derive(serde::Serialize, Debug)]
pub struct MonitoredFile {
    subfolder: String,
    name: String,
    time_added: Option<NaiveDateTime>,
    time_latest_edit: Option<NaiveDateTime>,
    time_latest_gen: Option<NaiveDateTime>,
    gen_count: usize,
    edit_count: usize,
}

impl Summary {
    pub fn new(project_name: &str) -> Self {
        Self {
            project_name: project_name.to_string(),
            scans: vec![],
            files: Default::default(),
        }
    }

    pub fn scan(project: &Project, is_gen: bool) {
        // For now don't deserialize the Summary object. Just create one.
        let mut summary = Summary::new(&project.name);
        summary.scan_self(project, is_gen);
        dbg!(summary); panic!();
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
                    println!("{}: {}", &file_time, &file_name);
                    match previous_scan_time {
                        Some(previous_scan_time) => {
                            if file_time > previous_scan_time {
                                changed_file_count += 1;
                                let key = MonitoredFile::make_key(subfolder, &file_name);
                                match self.files.get_mut(&key) {
                                    Some(file) => {
                                        if is_gen {
                                            file.time_latest_gen = Some(file_time);
                                        } else {
                                            file.time_latest_edit = Some(file_time);
                                        }
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
        self.scans.push(scan);
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
}
