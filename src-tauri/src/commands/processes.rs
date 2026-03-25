use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

#[tauri::command]
pub fn kill_process(pid: u32) -> Result<(), String> {
    let nix_pid = Pid::from_raw(pid as i32);
    kill(nix_pid, Signal::SIGTERM).map_err(|e| match e {
        nix::errno::Errno::EPERM => {
            format!("Permission denied: cannot kill PID {pid} (owned by another user)")
        }
        nix::errno::Errno::ESRCH => format!("Process {pid} not found (already exited)"),
        other => format!("Failed to kill PID {pid}: {other}"),
    })
}
