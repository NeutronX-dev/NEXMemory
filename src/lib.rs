#![allow(non_snake_case)]
#![allow(unused)]

use std::error::Error;

#[link(name = "kernel32")]
extern "C" {
    fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: bool, dwProcessId: u32) -> u32;
    fn QueryFullProcessImageNameA(hProcess: u32, dwFlags: u32, lpExeName: *mut u8, lpdwSize: *mut u32) -> bool;
    fn ReadProcessMemory(hProcess: u32, lpBaseAddress: *mut u32, lpBuffer: *mut std::ffi::c_void, nSize: u32, lpNumberOfBytesRead: *mut u32) -> bool;
    fn WriteProcessMemory(hProcess: u32, lpBaseAddress: *mut u32, lpBuffer: *mut std::ffi::c_void, nSize: u32, lpNumberOfBytesRead: *mut u32) -> bool;
    fn CloseHandle(hObject: u32) -> bool;

    fn CreateToolhelp32Snapshot(dwFlags: u32, th32ProcessID: u32) -> u32;
    fn Process32Next(hSnapshot: u32, lppe: *mut ProcessEntry32) -> bool;

    fn TerminateProcess(hProcess: u32, uExitCode: u32) -> bool;
}

#[derive(Debug)]
pub enum NEXMemoryError {
    EnumProcessError,
    ProcessNotFound,
    UnableToReadMemory,
    UnableToWriteMemory,

    UnableToTakeSnapshot,
    ProcessReadError,
    OpenProcessError,

    Other(Box<dyn Error>),
}

impl Error for NEXMemoryError {}

impl std::fmt::Display for NEXMemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::EnumProcessError => {
                write!(f, "Failed to enumerate processes")
            }
            Self::ProcessNotFound => {
                write!(f, "The proccess was not found")
            },
            Self::UnableToReadMemory => {
                write!(f, "Unable to read process memory")
            },
            Self::UnableToWriteMemory => {
                write!(f, "Unable to write process memory")
            },
            Self::UnableToTakeSnapshot => {
                write!(f, "Unable to take snapshot")
            },
            Self::ProcessReadError => {
                write!(f, "Error while reading proccess")
            }
            Self::OpenProcessError => {
                write!(f, "Error while opening process")
            }
            Self::Other(e) => e.fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct Process {
    handle: u32,
    pid: u32,
}

impl Process {
    /// 
    #[must_use]
    pub fn new(handle: u32, pid: u32) -> Process {
        let res: Process = Process { handle: handle, pid: pid };
        res
    }

    /// Reads the value inside [address] and inserts it into the [read_into].
    /// # Usage
    /// ```
    /// let mut variable: u32 = 0;
    /// process.read_memory(&mut variable, 0x000000);
    /// ```
    pub fn read_memory<T>(&self, read_into: *mut T, address: u32) -> Result<u32, NEXMemoryError> {
        let mut bytes_read: u32 = 0; 

        if !unsafe { ReadProcessMemory(self.handle, address as usize as *mut _, read_into as *mut _, std::mem::size_of_val(&read_into) as u32, &mut bytes_read) } {
            return Err(NEXMemoryError::UnableToReadMemory);
        }
        
        return Ok(bytes_read);
    }

    /// Writes [write] into [address].
    /// # Usage
    /// ```
    /// let mut variable: u32 = 100;
    /// process.write_memory(&mut variable, 0x000000);
    /// ```
    pub fn write_memory<T>(&self, write: *mut T, address: u32) -> Result<u32, NEXMemoryError> {
        let mut bytes_written: u32 = 0; 
            if !unsafe { WriteProcessMemory(self.handle, address as usize as *mut _, write as *mut _, std::mem::size_of_val(&write) as u32, &mut bytes_written) } {
                return Err(NEXMemoryError::UnableToWriteMemory);
        }
        
        return Ok(bytes_written);
    }

    /// Returns the path to the executable
    /// # Usage
    /// ```
    /// process.process_name().unwrap();
    /// ```
    pub fn process_name(&self) -> Result<String, NEXMemoryError> {
        let mut executable_path: [u8; 1024] = [0_u8; 1024];
        let mut characters_written: u32 = (executable_path.len() * std::mem::size_of::<u32>()) as u32;
        if unsafe { QueryFullProcessImageNameA(self.handle, 0, executable_path.as_mut_ptr(), &mut characters_written ) } {
            return match String::from_utf8(executable_path.iter().filter(|x| **x != 0).map(|x| x.clone()).collect()) {
                Ok(proc_name) => Ok(proc_name),
                Err(e) => Err(NEXMemoryError::Other(Box::new(e))) 
            }
        }

        Err(NEXMemoryError::ProcessNotFound)
    }

    pub fn terminate(&self, code: u32) -> bool {
        unsafe { TerminateProcess(self.handle, code) }
    }
}

#[repr(C)]
struct ProcessEntry32 {
    dwSize: u32,
    cntUsage: u32,
    th32ProcessID: u32,
    th32DefaultHeapID: u32,
    th32ModuleID: u32,
    cntThreads: u32,
    th32ParentProcessID: u32,
    pcPriClassBase: i32,
    dwFlags: u32,
    szExeFile: [u8; 1080],
}


// Close Handle when Destructed
impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

/// Returns the handle of the first process that matches with the set condition.
/// # Usage
/// ```
/// let handle: u32 = NEXMemory::process_match_name(|proc| proc.contains("example.exe")).unwrap();
/// ````
#[must_use]
pub fn process_match_name<F>(predicate: F) -> Result<Process, NEXMemoryError> where F: Fn(String) -> bool, {
    let mut processes: [u32; 1024] = [0_u32; 1024];
    let mut bytes_written: u32 = 0;

    let snapshot: u32 = unsafe { CreateToolhelp32Snapshot(0x00000002, 0)};
    if snapshot == 0 {
        return Err(NEXMemoryError::UnableToTakeSnapshot);
    }

    let mut entry: ProcessEntry32 = ProcessEntry32 { dwSize: std::mem::size_of::<ProcessEntry32>() as u32, cntUsage: 0, th32ProcessID: 0, th32DefaultHeapID: 0, th32ModuleID: 0, cntThreads: 0, th32ParentProcessID: 0, pcPriClassBase: 0, dwFlags: 0, szExeFile: [0_u8; 1080] };
    while ( unsafe { Process32Next(snapshot, &mut entry as *mut ProcessEntry32) } ) {
        match String::from_utf8(entry.szExeFile.iter().filter(|x| **x != 0).map(|x| x.clone()).collect()) {
            Ok(proc_name) => {
                if predicate(proc_name) {
                    let handle: u32 = unsafe { OpenProcess(0x1f0fff, true, entry.th32ProcessID) };
                    if handle == 0 {
                        return Err(NEXMemoryError::OpenProcessError);
                    }
                    return Ok(Process::new(handle, entry.th32ProcessID));
                    break;
                }
            },
            Err(e) => return Err(NEXMemoryError::Other(Box::new(e))),
        }
    }
    Err(NEXMemoryError::ProcessNotFound)
}
