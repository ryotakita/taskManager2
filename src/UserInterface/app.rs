use crate::util::{RandomSignal, SinSignal, StatefulList, TabsState};
use std::fs::File;
use std::io::Write;
use std::io::{self, BufRead, BufReader};
use once_cell::sync::OnceCell;
use main;
use std::fmt::{self, Formatter, Display};
use std::fs;
use std::path;
use std::error::Error;
use winrt;
use windows::{
    storage::StorageFile,
    system::Launcher,
};
use regex::Regex;

winrt::import!(
    dependencies
        os
    types
        windows::system::Launcher
);

const TASKS: [&str; 24] = [
    "Item1", "Item2", "Item3", "Item4", "Item5", "Item6", "Item7", "Item8", "Item9", "Item10",
    "Item11", "Item12", "Item13", "Item14", "Item15", "Item16", "Item17", "Item18", "Item19",
    "Item20", "Item21", "Item22", "Item23", "Item24",
];

const CLIENTS: [&str; 24] = [
    "Clients1", "Clients2", "Clients3", "Clients4", "Clients5", "Clients6", "Clients7", "Clients8", "Clients9", "Clients10",
    "Clients11", "Clients12", "Clients13", "Clients14", "Clients15", "Clients16", "Clients17", "Clients18", "Clients19",
    "Clients20", "Clients21", "Clients22", "Clients23", "Clients24",
];

const DATES: [&str; 24] = [
    "21-07-1", "21-07-2", "21-07-3", "21-07-4", "21-07-5", "21-07-6", "21-07-7", "21-07-8", "21-07-9", "21-07-10",
    "21-07-11", "21-07-12", "21-07-13", "21-07-14", "21-07-15", "21-07-16", "21-07-17", "21-07-18", "21-07-19",
    "21-07-20", "21-07-21", "21-07-22", "21-07-23", "21-07-24",
];

const LOGS: [(&str, &str); 26] = [
    ("Event1", "INFO"),
    ("Event2", "INFO"),
    ("Event3", "CRITICAL"),
    ("Event4", "ERROR"),
    ("Event5", "INFO"),
    ("Event6", "INFO"),
    ("Event7", "WARNING"),
    ("Event8", "INFO"),
    ("Event9", "INFO"),
    ("Event10", "INFO"),
    ("Event11", "CRITICAL"),
    ("Event12", "INFO"),
    ("Event13", "INFO"),
    ("Event14", "INFO"),
    ("Event15", "INFO"),
    ("Event16", "INFO"),
    ("Event17", "ERROR"),
    ("Event18", "ERROR"),
    ("Event19", "INFO"),
    ("Event20", "INFO"),
    ("Event21", "WARNING"),
    ("Event22", "INFO"),
    ("Event23", "INFO"),
    ("Event24", "WARNING"),
    ("Event25", "INFO"),
    ("Event26", "INFO"),
];

const EVENTS: [(&str, u64); 24] = [
    ("B1", 9),
    ("B2", 12),
    ("B3", 5),
    ("B4", 8),
    ("B5", 2),
    ("B6", 4),
    ("B7", 5),
    ("B8", 9),
    ("B9", 14),
    ("B10", 15),
    ("B11", 1),
    ("B12", 0),
    ("B13", 4),
    ("B14", 6),
    ("B15", 4),
    ("B16", 6),
    ("B17", 4),
    ("B18", 7),
    ("B19", 13),
    ("B20", 8),
    ("B21", 11),
    ("B22", 9),
    ("B23", 3),
    ("B24", 5),
];

pub struct Signal<S: Iterator> {
    source: S,
    pub points: Vec<S::Item>,
    tick_rate: usize,
}

impl<S> Signal<S>
where
    S: Iterator,
{
    fn on_tick(&mut self) {
        for _ in 0..self.tick_rate {
            self.points.remove(0);
        }
        self.points
            .extend(self.source.by_ref().take(self.tick_rate));
    }
}

pub struct Signals {
    pub sin1: Signal<SinSignal>,
    pub sin2: Signal<SinSignal>,
    pub window: [f64; 2],
}

impl Signals {
    fn on_tick(&mut self) {
        self.sin1.on_tick();
        self.sin2.on_tick();
        self.window[0] += 1.0;
        self.window[1] += 1.0;
    }
}

pub struct Server<'a> {
    pub name: &'a str,
    pub location: &'a str,
    pub coords: (f64, f64),
    pub status: &'a str,
}

#[derive(Clone)]
pub struct Task  {
    pub folder_name: String,
}

impl Task {
    pub fn new(folder_nmae: String) -> Self {
        Task {
            folder_name: folder_nmae.to_string().clone(),
        }
    }
}

pub fn read_dir(path: &str) -> Result<Vec<path::PathBuf>, Box<dyn Error>> {
    let dir = fs::read_dir(path)?;
    let mut files: Vec<path::PathBuf> = Vec::new();
    for item in dir.into_iter() {
        files.push(item?.path());
    }
    Ok(files)
}

pub fn launch_file(path: &str) -> winrt::Result<()> {
    // ファイルパスから `StorageFile` オブジェクトを取得
    let file = StorageFile::get_file_from_path_async(path).unwrap().get().unwrap();

    // 既定のプログラムを使用して `file` を開く
    Launcher::launch_file_async(file).unwrap().get().unwrap();
    Ok(())
}

impl Display for Task {
    // `f` is a buffer, and this method must write the formatted string into it
    // `f` はバッファです。このメソッドは
    // ここにフォーマットされた文字列を書き込みます。
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // `write!` is like `format!`, but it will write the formatted string
        // into a buffer (the first argument)
        // `write!`は`format!`に似ていますが、フォーマットされた文字列を
        // バッファ（第一引数）に書き込みます。
        let current_dir = Regex::new(r"\\[^\\]*$").unwrap();
        let mut current_dir = current_dir.find(&self.folder_name).unwrap().as_str().to_string();
        current_dir.remove(0); // 先頭の\が邪魔なので消しておく
        write!(f, "{}", current_dir)
    }
}

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub show_chart: bool,
    pub progress: f64,
    pub sparkline: Signal<RandomSignal>,
    pub folders: StatefulList<Task>,
    pub logs: StatefulList<(&'a str, &'a str)>,
    pub signals: Signals,
    pub barchart: Vec<(&'a str, u64)>,
    pub servers: Vec<Server<'a>>,
    pub enhanced_graphics: bool,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, enhanced_graphics: bool) -> App<'a> {
        let mut rand_signal = RandomSignal::new(0, 100);
        let sparkline_points = rand_signal.by_ref().take(300).collect();
        let mut sin_signal = SinSignal::new(0.2, 3.0, 18.0);
        let sin1_points = sin_signal.by_ref().take(100).collect();
        let mut sin_signal2 = SinSignal::new(0.1, 2.0, 10.0);
        let sin2_points = sin_signal2.by_ref().take(200).collect();

        let mut task_list: Vec<Task> = Vec::new();
        for folders in read_dir("E:\\SRC"){
            for folder in folders {
                task_list.push(Task::new(folder.to_str().unwrap().to_string()));
            }
        }

        App {
            title,
            should_quit: false,
            tabs: TabsState::new(vec!["Tab0", "Tab1", "Tab2"]),
            show_chart: false,
            progress: 0.0,
            sparkline: Signal {
                source: rand_signal,
                points: sparkline_points,
                tick_rate: 1,
            },
            folders: StatefulList::with_items(task_list),
            logs: StatefulList::with_items(LOGS.to_vec()),
            signals: Signals {
                sin1: Signal {
                    source: sin_signal,
                    points: sin1_points,
                    tick_rate: 5,
                },
                sin2: Signal {
                    source: sin_signal2,
                    points: sin2_points,
                    tick_rate: 10,
                },
                window: [0.0, 20.0],
            },
            barchart: EVENTS.to_vec(),
            servers: vec![
                Server {
                    name: "NorthAmerica-1",
                    location: "New York City",
                    coords: (40.71, -74.00),
                    status: "Up",
                },
                Server {
                    name: "Europe-1",
                    location: "Paris",
                    coords: (48.85, 2.35),
                    status: "Failure",
                },
                Server {
                    name: "SouthAmerica-1",
                    location: "São Paulo",
                    coords: (-23.54, -46.62),
                    status: "Up",
                },
                Server {
                    name: "Asia-1",
                    location: "Singapore",
                    coords: (1.35, 103.86),
                    status: "Up",
                },
            ],
            enhanced_graphics,
        }
    }

    pub fn next_dir(&self, path: &str) -> StatefulList<Task>{
        let mut task_list: Vec<Task> = Vec::new();
        for folders in read_dir(path){
            for folder in folders {
                task_list.push(Task::new(folder.to_str().unwrap().to_string()));
            }
        }
        StatefulList::with_items(task_list)
    }

    pub fn on_up(&mut self) {
        self.folders.previous();
    }

    pub fn on_down(&mut self) {
        self.folders.next();
    }

    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    pub fn on_enter_dir(&mut self) {
        match self.folders.state.selected() {
            Some(x) => {
                let path_target = &self.folders.items[x].folder_name;
                let path_target = path::Path::new(path_target);
                let path_target = path::PathBuf::from(path_target);
                match path_target.is_dir() {
                    true => {
                        self.folders = self.next_dir(path_target.to_str().unwrap());
                    },
                    false => {
                        launch_file(path_target.to_str().unwrap());
                    }
                };
            }
            _ => {}
        }
    }

    pub fn on_back_dir(&mut self) {
        match self.folders.state.selected() {
            Some(x) => {
                let path_target = &self.folders.items[x].folder_name;
                let path_target = path::Path::new(path_target);
                let path_target = path::PathBuf::from(path_target);
                let path_parent = path_target.parent().unwrap().parent();
                match path_parent {
                    Some(x) => {
                        self.folders = self.next_dir(x.to_str().unwrap());
                    },
                    None => {},
                }
            },
            _ => {}
        }
    }

    // TODO:ファイル読込処理
    pub fn get_path_of_number(&mut self, number: usize) -> path::PathBuf {
        let path_target = match number {
            1 => "C:\\Users\\ryota-kita",
            2 => "C:\\Users\\ryota-kita\\Documents\\working",
            3 => "E:\\SRC",
            4 => "Z:\\",
            5 => "E:\\機能設計書",
            _ => "E:",
        };
        let path_target = path::Path::new(path_target);

        path::PathBuf::from(path_target)
    }

    pub fn on_key(&mut self, c: char, pos: (u16, u16)) {
        match c.is_numeric() {
            true => {
                let path_target = self.get_path_of_number(c as usize - 48);
                match path_target.is_dir() {
                    true => {
                        self.folders = self.next_dir(path_target.to_str().unwrap());
                    },
                    false => {
                        launch_file(path_target.to_str().unwrap());
                    }
                };
            }
            false =>{
                match c {
                    'q' => {
                        self.should_quit = true;
                        let mut file = File::create("task.txt").expect("writeError");

                        for task in self.folders.items.iter() {
                            file.write(format!("{}\n",task).as_bytes()).unwrap();
                        }
                    }
                    't' => {
                        self.show_chart = !self.show_chart;
                    }
                    'j' => { self.on_down(); }
                    'k' => { self.on_up(); }
                    'c' => { self.on_enter_dir(); }
                    'l' => { self.on_enter_dir(); }
                    'h' => { self.on_back_dir(); }
                    _ => {}
                }
            }


        }
    }

    pub fn add_task(&mut self) {

    }

    pub fn on_tick(&mut self) {
        // Update progress
        self.progress += 0.001;
        if self.progress > 1.0 {
            self.progress = 0.0;
        }

        self.sparkline.on_tick();
        self.signals.on_tick();

        let log = self.logs.items.pop().unwrap();
        self.logs.items.insert(0, log);

        let event = self.barchart.pop().unwrap();
        self.barchart.insert(0, event);
    }
}
