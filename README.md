# NEXMemory
## Easily read, and write into a process's memory.

# Getting Started
Firstly, you need to import `NEXMemory` into your project, you can do this by writing the following below `[dependencies]` in `Cargo.toml`:
```
NEXMemory = "0.1.0"
```
## ALTERNATIVELY
Running this in your terminal:
```
cargo add NEXMemory
```

# Sample
```rs
fn main() {
    // Getting handle for example.exe & Building a process struct
    let handle: u32 = NEXMemory::process_match_name(|proc| proc.contains("example.exe")).unwrap();
    let process = NEXMemory::Process::new(handle);

    // Reading Example
    let mut read_value: u32 = 0;
    println!("Read {} bytes.", {
        process.read_memory(&mut read_value, 0x0000110).unwrap() // Random Address
    });

    // Writing Example
    let mut write_value: u32 = 0;
    println!("Wrote {} bytes.", {
        process.write_memory(&mut write_value, 0x0000110).unwrap() // Random Address
    });
}
```

# Note
The code is probably bad, I am still learning, however if you find anything that as poorly written, a bad practice, etc... feel free to correct it.

# LICENSE
![gnu-logo](https://raw.githubusercontent.com/NeutronX-dev/SimpleSchedule/main/logos/gplv3-88x31.png)

This program is free software: you can redistribute it and/or modify
it under the terms of the [GNU General Public License](https://github.com/NeutronX-dev/ws.js/blob/main/LICENSE) as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.