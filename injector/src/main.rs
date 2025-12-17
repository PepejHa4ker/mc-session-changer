
use anyhow::Result;
use egui::{CentralPanel, Context as EguiCtx};
use std::cmp::Ordering;
use std::ffi::OsStr;
use std::io::Write;
use std::iter;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use sysinfo::{ProcessesToUpdate, System};
use winapi::shared::minwindef::{FALSE, LPVOID};
use winapi::um::handleapi::CloseHandle;
use winapi::um::libloaderapi::{GetModuleHandleW, GetProcAddress};
use winapi::um::memoryapi::{VirtualAllocEx, WriteProcessMemory};
use winapi::um::processthreadsapi::{CreateRemoteThread, OpenProcess};
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::wincon::FreeConsole;
use winapi::um::winnt::{
    MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE, PROCESS_CREATE_THREAD, PROCESS_QUERY_INFORMATION,
    PROCESS_VM_OPERATION, PROCESS_VM_WRITE, SYNCHRONIZE,
};

const EMBEDDED_DLL: &[u8] = include_bytes!("../../target/release/mc_session_changer.dll");
static DEFAULT_DLL_NAME: &str = "mc-session-changer.dll";

#[derive(Clone, Debug)]
struct ProcItem {
    pid: u32,
    name: String,
}

fn wide_str(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(iter::once(0)).collect()
}

fn list_processes() -> Vec<ProcItem> {
    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);

    let mut items: Vec<ProcItem> = system
        .processes()
        .iter()
        .map(|(pid, proc)| ProcItem {
            pid: pid.as_u32(),
            name: proc.name().to_str().unwrap().to_string(),
        })
        .filter(|p| p.name.starts_with("java"))
        .collect();

    items.sort_by(|a, b| {
        let cmp_name = a
            .name
            .to_lowercase()
            .cmp(&b.name.to_lowercase());
        if cmp_name == Ordering::Equal {
            a.pid.cmp(&b.pid)
        } else {
            cmp_name
        }
    });

    items
}

fn inject_from_memory(pid: u32, dll_data: &[u8]) -> Result<()> {
    let temp_dir = std::env::temp_dir();
    let temp_dll_path = temp_dir.join(format!("{}{}",DEFAULT_DLL_NAME,pid));

    {
        let mut file = std::fs::File::create(&temp_dll_path)?;
        file.write_all(dll_data)?;
    }

    unsafe {
        let h_proc = OpenProcess(
            PROCESS_CREATE_THREAD
                | PROCESS_QUERY_INFORMATION
                | PROCESS_VM_OPERATION
                | PROCESS_VM_WRITE
                | SYNCHRONIZE,
            FALSE,
            pid,
        );
        anyhow::ensure!(!h_proc.is_null(), "OpenProcess failed for PID {}", pid);

        let dll_w: Vec<u16> = wide_str(temp_dll_path.to_str().unwrap());
        let size = dll_w.len() * 2;

        let remote_mem: LPVOID =
            VirtualAllocEx(h_proc, ptr::null_mut(), size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
        anyhow::ensure!(!remote_mem.is_null(), "VirtualAllocEx failed");

        let ok = WriteProcessMemory(
            h_proc,
            remote_mem,
            dll_w.as_ptr() as *const _,
            size,
            ptr::null_mut(),
        );
        anyhow::ensure!(ok != 0, "WriteProcessMemory failed");

        let h_k32 = GetModuleHandleW(wide_str("Kernel32.dll").as_ptr());
        anyhow::ensure!(!h_k32.is_null(), "GetModuleHandleW(Kernel32.dll) failed");

        let load_lib = GetProcAddress(h_k32, b"LoadLibraryW\0".as_ptr() as _);
        anyhow::ensure!(!load_lib.is_null(), "GetProcAddress(LoadLibraryW) failed");

        let h_thread = CreateRemoteThread(
            h_proc,
            ptr::null_mut(),
            0,
            Some(std::mem::transmute(load_lib)),
            remote_mem,
            0,
            ptr::null_mut(),
        );
        anyhow::ensure!(!h_thread.is_null(), "CreateRemoteThread failed");

        WaitForSingleObject(h_thread, 30_000);
        CloseHandle(h_thread);
        CloseHandle(h_proc);
    }

    let _ = std::fs::remove_file(&temp_dll_path);

    Ok(())
}

struct AppState {
    processes: Vec<ProcItem>,
    selected: Option<usize>,
    status: String,
    last_refresh: std::time::Instant,
}

struct App {
    state: AppState,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.ui(ctx);
    }
}

impl AppState {
    fn new() -> Self {
        let processes = list_processes();

        Self {
            processes,
            selected: None,
            status: String::new(),
            last_refresh: std::time::Instant::now(),
        }
    }

    fn ui(&mut self, ctx: &EguiCtx) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Injector");
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Обновить процессы").clicked() {
                    self.processes = list_processes();
                    self.status.clear();
                    if self.selected
                        .and_then(|i| self.processes.get(i))
                        .is_none()
                    {
                        self.selected = None;
                    }
                    self.last_refresh = std::time::Instant::now();
                }

                let elapsed = self.last_refresh.elapsed().as_secs();
                ui.label(format!("Обновлено: {}с назад", elapsed));
            });

            ui.separator();
            ui.label(format!("Процессы (всего: {}):", self.processes.len()));

            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    for (i, p) in self.processes.iter().enumerate() {
                        let selected = Some(i) == self.selected;
                        let label_text = format!("{} (PID: {})", p.name, p.pid);

                        if ui
                            .selectable_label(selected, label_text)
                            .clicked()
                        {
                            self.selected = Some(i);
                        }
                    }
                });

            ui.separator();

            let can_inject = self.selected.is_some();
            if ui.add_enabled(can_inject, egui::Button::new("Инжект")).clicked() {
                if let Some(i) = self.selected {
                    let pid = self.processes[i].pid;
                    match inject_from_memory(pid, EMBEDDED_DLL) {
                        Ok(_) => {
                            self.status = format!("Успех: инжект в PID {} ({})", pid, self.processes[i].name);
                        }
                        Err(e) => {
                            self.status = format!("Ошибка: {e:#}");
                        }
                    }
                }
            }

            ui.separator();

            if !self.status.is_empty() {
                let color = if self.status.starts_with("Успех") {
                    egui::Color32::GREEN
                } else if self.status.starts_with("Ошибка") {
                    egui::Color32::RED
                } else {
                    egui::Color32::WHITE
                };
                ui.colored_label(color, &self.status);
            }
        });
    }
}

fn main() -> Result<()> {
    unsafe {
        FreeConsole();
    }
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([550.0, 650.0])
            .with_resizable(false)
            .with_title("Spoofer injector"),
        ..Default::default()
    };

    eframe::run_native(
        "Injector",
        native_options,
        Box::new(|_cc| Ok(Box::new(App { state: AppState::new() }) as Box<dyn eframe::App>)),
    ).expect("Failed to run spoofer injector");

    Ok(())
}