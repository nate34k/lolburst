use std::{
    io, thread,
    time::{self, Duration, Instant}, sync::{Arc, atomic::Ordering},
};
use std::sync::atomic::AtomicBool;

use crossbeam::{
    channel::{unbounded, Receiver},
    select,
};
use crossterm::event::{self, Event, KeyCode};
use process_memory::*;
use reqwest::Client;
use serde_json::Value;
use slice_deque::SliceDeque;
use sysinfo::{RefreshKind, ProcessRefreshKind, System, SystemExt, ProcessExt, PidExt};
use tui::{backend::Backend, widgets::TableState, Terminal};
use tui_logger::TuiWidgetState;

use crate::{
    champion::{self, Champion},
    config::Config,
    handlers::keyboard::{handle_keyboard, KeyboardHandler},
    network, ui,
    ui::burst_table::BurstTable,
    utils::{deserializer, teams}, data::LiveGame,
};

pub struct App {
    pub burst_table_state: TableState,
    pub burst_table_items: Vec<Vec<String>>,
    pub burst_last: Vec<String>,
    pub logger_state: TuiWidgetState,
    pub draw_logger: bool,
    pub logger_scroll_mode: bool,
    pub gold: ui::gold::Gold,
    pub cs: ui::cs::CS,
    pub vs: ui::vs::VS,
    last_tick: time::Instant,
    pub use_sample_data: bool,
}

impl App {
    pub fn new(c: &Config) -> App {
        App {
            burst_table_state: TableState::default(),
            burst_table_items: vec![
                vec!["Lucian".to_string(), "1".to_string(), "0.0".to_string()],
                vec!["Ahri".to_string(), "1".to_string(), "0.0".to_string()],
                vec!["Orianna".to_string(), "1".to_string(), "0.0".to_string()],
                vec!["Diana".to_string(), "1".to_string(), "0.0".to_string()],
                vec!["Blitzcrank".to_string(), "1".to_string(), "0.0".to_string()],
            ],
            burst_last: vec!["0.0".to_string(); 5],
            logger_state: TuiWidgetState::default(),
            draw_logger: false,
            logger_scroll_mode: false,
            gold: ui::gold::Gold::new(),
            cs: ui::cs::CS::new(),
            vs: ui::vs::VS::new(),
            last_tick: time::Instant::now(),
            use_sample_data: c.use_sample_data,
        }
    }

    fn on_tick(&mut self, game_time: f64, cur_gold: f64, cur_cs: i64, cur_vs: f64) {
        self.gold.on_tick(game_time, cur_gold);
        self.cs.on_tick(game_time, cur_cs);
        self.vs.on_tick(game_time, cur_vs);
    }

    fn check_elapsed_time(&mut self, sample_rate: u64) {
        if self.last_tick.elapsed() >= Duration::from_millis(sample_rate * 1000) {
            self.last_tick = Instant::now();
        }
    }

    fn get_timeout(&self, sample_rate: u64) -> Duration {
        Duration::from_millis(sample_rate * 1000)
            .checked_sub(self.last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_millis(0))
    }

    fn reset_datasets(&mut self, config: &Config, game_time: f64) {
        self.gold.reset_dataset(config, game_time);
        self.cs.reset_dataset(config, game_time);
        self.vs.reset_dataset(config, game_time);
    }
}

// pub fn get_pid(process_name: &str) -> process_memory::Pid {
//     let r = RefreshKind::new().with_processes(ProcessRefreshKind::new());

//     let sys = System::new_with_specifics(r);

//     for (pid, proc) in sys.processes() {
//         if proc.name() == process_name {
//             println!("Found process: {} with pid: {}", proc.name(), pid);
//             return pid.as_u32();
//         }
//     }
//     0
// }

// This needs to be refactored and moved to a separate file
pub fn get_pid(process_name: &str) -> (process_memory::Pid, usize) {
    // A helper function to turn a c_char array to a String
    fn utf8_to_string(bytes: &[i8]) -> String {
        use std::ffi::CStr;
        unsafe {
            // Convert the c_char array to a CStr and then to a String
            CStr::from_ptr(bytes.as_ptr())
                .to_string_lossy()
                .into_owned()
        }
    }

    // Define entry to be a PROCESSENTRY32 struct
    let mut entry = winapi::um::tlhelp32::PROCESSENTRY32 {
        dwSize: std::mem::size_of::<winapi::um::tlhelp32::PROCESSENTRY32>() as u32,
        cntUsage: 0,
        th32ProcessID: 0,
        th32DefaultHeapID: 0,
        th32ModuleID: 0,
        cntThreads: 0,
        th32ParentProcessID: 0,
        pcPriClassBase: 0,
        dwFlags: 0,
        szExeFile: [0; winapi::shared::minwindef::MAX_PATH],
    };

    // Define snapshot to be a HANDLE
    let snapshot: winapi::um::winnt::HANDLE;

    // Define pid to be a DWORD and set it to 0
    let mut pid: u32 = 0;

    // Define base_addr to be a usize and set it to 0
    let mut base_addr: usize = 0;

    // Scary unsafe code to get the pid of the process defined in process_name
    unsafe {
        // Create a snapshot of all processes
        snapshot = winapi::um::tlhelp32::CreateToolhelp32Snapshot(
            winapi::um::tlhelp32::TH32CS_SNAPPROCESS,
            0,
        );

        // Check if the snapshot was created successfully
        if winapi::um::tlhelp32::Process32First(snapshot, &mut entry)
            == winapi::shared::minwindef::TRUE
        {
            // Loop through all processes
            while winapi::um::tlhelp32::Process32Next(snapshot, &mut entry)
                == winapi::shared::minwindef::TRUE
            {
                // Check if the process name matches the process name we are looking for
                if utf8_to_string(&entry.szExeFile) == process_name {
                    // Set the pid to the pid of the process we are looking for and stop
                    // looping through processes
                    pid = entry.th32ProcessID;
                    break;
                }
            }
        }
    }

    // Define entry to be a MODULEENTRY32 struct
    let mut entry = winapi::um::tlhelp32::MODULEENTRY32 {
        dwSize: std::mem::size_of::<winapi::um::tlhelp32::MODULEENTRY32>() as u32,
        th32ModuleID: 0,
        th32ProcessID: 0,
        GlblcntUsage: 0,
        ProccntUsage: 0,
        modBaseAddr: std::ptr::null_mut(),
        modBaseSize: 0,
        hModule: std::ptr::null_mut(),
        szModule: [0; winapi::um::tlhelp32::MAX_MODULE_NAME32 + 1],
        szExePath: [0; winapi::shared::minwindef::MAX_PATH],
    };

    // Define snapshot to be a HANDLE
    let snapshot: winapi::um::winnt::HANDLE;

    // Scary unsafe code to get the base address of the processes main module as defined 
    // in process_name, this is the address of the .exe file
    unsafe {
        // Create a snapshot of all modules in the process
        snapshot = winapi::um::tlhelp32::CreateToolhelp32Snapshot(
            winapi::um::tlhelp32::TH32CS_SNAPMODULE,
            pid,
        );
        // Check if the snapshot was created successfully
        if winapi::um::tlhelp32::Module32First(snapshot, &mut entry) 
            == winapi::shared::minwindef::TRUE
        {   
            // Check if the first module is the main module (it should be)
            if utf8_to_string(&entry.szModule) == process_name {
                // Set the base_addr to the base address of the main module
                base_addr = entry.modBaseAddr as usize;
            }
            while winapi::um::tlhelp32::Module32Next(snapshot, &mut entry)
                == winapi::shared::minwindef::TRUE
            {
                // Check if the module name matches the process name we are looking for
                if utf8_to_string(&entry.szModule) == process_name {
                    // Set the base_addr to the base address of the module we are
                    // looking for
                    base_addr = entry.modBaseAddr as usize;
                }
            }
        }
    }
    info!("pid: {}", pid);
    info!("base_addr: {:#01x}", base_addr);

    // Return the pid and base address of the process
    (pid, base_addr)
}

const PROCESS_NAME: &str = "League of Legends.exe";
const LOCAL_PLAYER_OFFSET: usize = 0x_3_14_15_54;
const CREEP_SCORE_OFFSET: usize = 0x_3B_D4;

// This needs to be refactored and moved to a separate file
// This function was made as a proof of concept to see if we could read the
// memory of a process given some known memory offsets
fn get_value() {
    // We need to make sure that we get a handle to a process
    let (pid, base_addr) = get_pid(PROCESS_NAME);
    let handle: ProcessHandle = pid
        .try_into_process_handle()
        .unwrap()
        .set_arch(Architecture::Arch32Bit);
    info!("Arch: {:?}", handle.1);
    // We make a `DataMember`
    let mut member = DataMember::<i32>::new(handle);

    member.set_offset(vec![base_addr + LOCAL_PLAYER_OFFSET]);
    // println!("Offset: {:#01x}", member.read().unwrap());
    let offset = member.read().unwrap() as usize + CREEP_SCORE_OFFSET;
    info!("New Offset: {:#01x}", offset);
    member.set_offset(vec![offset]);

    info!("Memory location: {:#01x}", member.get_offset().unwrap());
    info!("Creep Score: {}", member.read().unwrap());
}

pub async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    config: &Config,
) -> io::Result<()> {
    // Build a client
    let client: Client = network::build_client().await;

    let terminate = Arc::new(AtomicBool::new(false));

    // Spawn threads to handle additional tasks
    let ui_events_rx = setup_ui_events(terminate.clone());

    // Deserialize the Data Dragon into serde_json::Value
    let data_dragon_champions: Value = serde_json::from_str(
        &network::request(&client, crate::DATA_DRAGON_URL)
            .await
            .text()
            .await
            .expect("Failed to parse data for String"),
    )
    .expect("Failed to deserialize String into JSON Value");

    let champion: Champion;
    let player_index: usize;
    let mut cycle: usize = 0;

    println!("Waiting for League of Legends to start...");

    // Check if game is ready
    loop {
        fn sleep(msg: &str, d: u64) {
            warn!("{}", msg);
            println!("{}", msg);
            thread::sleep(Duration::from_secs(d));
        }

        println!("Waiting for League of Legends to start...");

        if terminate.load(Ordering::Relaxed) {
            return Ok(());
        }

        let data: Result<LiveGame, serde_json::Error>;
        tokio::select! {
            d = deserializer::deserializer(&app, &client, cycle) => {
                data = d;
            }
            _ = tokio::time::sleep(Duration::from_secs(5)) => {
                continue;
            }
        }

        //let data = deserializer::deserializer(&app, &client, cycle).await;
        
        // Guard clause to check if the game is ready
        //
        // If the game is not ready, we will wait for 5 seconds and try again
        //
        // This check is done because Riot API will send bogus data during the 
        // loading screen, so we wait until we have data in the events vec before 
        // continuing on with the main loop
        if let Err(_) = data {
            sleep("Error deserializing data, retrying in 5 seconds...", 5);
            continue;
        } else if let None = data.as_ref().unwrap().events {
            sleep("Game is not ready, retrying in 5 seconds...", 5);
            continue;
        } else if let true = data
            .as_ref()
            .unwrap()
            .events
            .as_ref()
            .unwrap()
            .events
            .is_empty()
        {
            sleep("Game is not ready, retrying in 5 seconds...", 5);
            continue;
        }

        println!("Game is ready!");

        let data = data.unwrap();

        let (i, _, c) =
            teams::get_active_player(&data.active_player.unwrap(), &data.all_players.unwrap());
        player_index = i;
        champion = champion::Champion::new(c.as_str());
        app.reset_datasets(config, data.game_data.unwrap().game_time);
        break;
    }

    // get_value();
    terminal.clear().unwrap();

    

    // Applicaiton loop
    loop {
        let time = time::Instant::now();

        draw(terminal, &app);

        app.burst_last = app
            .burst_table_items
            .iter()
            .map(|x| x.last().unwrap().clone())
            .collect::<Vec<_>>();

        let timeout = app.get_timeout(config.sample_rate);

        // Handle UI events
        select! {
            recv(ui_events_rx) -> event => {
                match handle_keyboard(event.unwrap(), &mut app) {
                    KeyboardHandler::Quit => { break; },
                    KeyboardHandler::None => {},
                }
            }
            default(timeout) => {
                // Check if we are using sample data and if so, check if we need to cycle the data back to the beginning
                if app.use_sample_data {
                    debug!("cycle: {}", cycle);
                    if cycle
                        == 6000
                    {
                        cycle = 0;
                        app.gold = ui::gold::Gold::new();
                        app.cs = ui::cs::CS::new();
                        app.vs = ui::vs::VS::new();
                    }
                }

                // Deserialize data from Riot API into Data struct
                let data = &deserializer::deserializer(&app, &client, cycle)
                    .await
                    .unwrap();

                info!("CS: {}", data.all_players.as_ref().unwrap()[player_index].scores.creep_score);

                // If app is on the first cycle, reset the datasets
                if cycle == 0 {
                    app.reset_datasets(config, data.game_data.as_ref().unwrap().game_time);
                }

                // Set burst_table to a new BurstTable
                let burst_table = BurstTable {
                    champion: &champion,
                    data,
                    data_dragon_data: &data_dragon_champions,
                    rotation: &config.rotation,
                };

                // Update app.burst_table_items to the correct Vec<Vec<String>>
                app.burst_table_items = BurstTable::build_burst_table_items(burst_table);

                app.on_tick(
                    data.game_data.as_ref().unwrap().game_time,
                    data.active_player.as_ref().unwrap().current_gold,
                    data.all_players.as_ref().unwrap()[player_index]
                        .scores
                        .creep_score,
                    data.all_players.as_ref().unwrap()[player_index]
                        .scores
                        .ward_score,
                );

                cycle += 1;
            }
        }

        app.check_elapsed_time(config.sample_rate);

        info!("cycle took: {:?}", time.elapsed());
    }
    Ok(())
}

fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &App) {
    terminal
        .draw(|f| {
            let size = f.size();
            ui::ui(f, size, app);
        })
        .unwrap();
}

pub enum UIEvent {
    Key(event::KeyEvent),
    Resize(u16, u16),
}

fn setup_ui_events(terminate: Arc<AtomicBool>) -> Receiver<UIEvent> {
    let (tx, rx) = unbounded();
    thread::spawn(move || loop {
        let event = event::read().unwrap();
        match event {
            Event::Mouse(_) => {}
            Event::Key(key_code) => {
                tx.send(UIEvent::Key(key_code)).unwrap();
            }
            Event::Resize(x, y) => {
                tx.send(UIEvent::Resize(x, y)).unwrap();
            }
        }
        if let Event::Key(key_event) = event {
            if let KeyCode::Char('q') = key_event.code {
                terminate.store(true, Ordering::Relaxed);
                break;
            }
        }
    });

    rx
}

fn get_dataset_length(config: &Config) -> usize {
    (config.dataset_lifetime as f64 / (config.sample_rate as f64)) as usize
}

pub trait Stats {
    fn reset_vecdeque_dataset(&self, config: &Config, game_time: f64) -> SliceDeque<(f64, f64)> {
        // Set offset to sample rate and divide by 1000 to get sapmle rate in seconds
        // Offset is used to determine how far back in time the graph should start
        let offset = config.sample_rate as usize;

        // Closure to create a VecDeque with the correct length and values for graphing
        let reversed_vecdeque_with_offset = || -> SliceDeque<(f64, f64)> {
            let mut x = SliceDeque::new();
            for i in 0..get_dataset_length(config) {
                x.push_back(((game_time - (offset * i) as f64), 0.0));
            }
            x.into_iter().rev().collect()
        };

        // Reassign values to the datasets
        reversed_vecdeque_with_offset()
    }

    fn reset_vec_dataset(&self, config: &Config) -> Vec<(f64, f64)> {
        vec![(0.0, 0.0); get_dataset_length(config)]
    }

    fn string_from_per_min(&self) -> String;
}
