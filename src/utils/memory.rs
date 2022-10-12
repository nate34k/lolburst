use process_memory::*;

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
