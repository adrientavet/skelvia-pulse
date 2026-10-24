//! Lecture sous Windows, avec les fonctions standard de Windows (les mêmes
//! qu'utilise ActivityWatch : `GetForegroundWindow`, `GetLastInputInfo`).

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, BOOL, HWND, LPARAM};
use windows::Win32::System::StationsAndDesktops::{
    CloseDesktop, OpenInputDesktop, DESKTOP_CONTROL_FLAGS, DESKTOP_SWITCHDESKTOP,
};
use windows::Win32::System::SystemInformation::GetTickCount64;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, GetLastInputInfo, LASTINPUTINFO, VK_LBUTTON, VK_RBUTTON};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId,
};

use crate::Foreground;

fn window_pid(hwnd: HWND) -> u32 {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    pid
}

/// Nom du fichier du programme (`EXCEL.EXE`), sans le chemin complet.
fn process_name(pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        let res = QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut size);
        let _ = CloseHandle(handle);
        res.ok()?;
        let path = String::from_utf16_lossy(&buf[..size as usize]);
        path.rsplit('\\').next().map(str::to_string)
    }
}

/// Les applications du Microsoft Store s'affichent dans une fenêtre
/// `ApplicationFrameHost.exe` ; le vrai programme est une fenêtre enfant
/// d'un autre processus.
fn store_app_pid(hwnd: HWND, host_pid: u32) -> Option<u32> {
    struct Search {
        host_pid: u32,
        found: u32,
    }
    unsafe extern "system" fn visit(child: HWND, lparam: LPARAM) -> BOOL {
        let search = &mut *(lparam.0 as *mut Search);
        let pid = window_pid(child);
        if pid != 0 && pid != search.host_pid {
            search.found = pid;
            return BOOL(0); // trouvé : on arrête
        }
        BOOL(1)
    }
    let mut search = Search { host_pid, found: 0 };
    unsafe {
        let _ = EnumChildWindows(hwnd, Some(visit), LPARAM(&mut search as *mut Search as isize));
    }
    (search.found != 0).then_some(search.found)
}

/// La fenêtre au premier plan, ou `None` s'il n'y en a pas (bureau
/// verrouillé, changement de fenêtre en cours).
pub fn foreground() -> Option<Foreground> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return None;
        }
        let len = GetWindowTextLengthW(hwnd).max(0) as usize;
        let mut buf = vec![0u16; len + 1];
        let n = GetWindowTextW(hwnd, &mut buf).max(0) as usize;
        let title = String::from_utf16_lossy(&buf[..n.min(buf.len())]);

        let pid = window_pid(hwnd);
        let mut app = process_name(pid).unwrap_or_default();
        if app.eq_ignore_ascii_case("ApplicationFrameHost.exe") {
            if let Some(name) = store_app_pid(hwnd, pid).and_then(process_name) {
                app = name;
            }
        }
        Some(Foreground { app, title })
    }
}

/// Secondes écoulées depuis la dernière action de la souris ou du clavier,
/// toutes applications confondues. Windows ne donne que l'heure de cette
/// dernière action, rien sur sa nature.
pub fn seconds_since_last_input() -> Option<f64> {
    unsafe {
        let mut info = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
        if !GetLastInputInfo(&mut info).as_bool() {
            return None;
        }
        // GetLastInputInfo donne un compteur sur 32 bits, qui repasse à zéro
        // tous les 49,7 jours ; on compare donc sur les 32 bits du bas.
        let now = (GetTickCount64() & 0xFFFF_FFFF) as u32;
        Some(now.wrapping_sub(info.dwTime) as f64 / 1000.0)
    }
}

/// Vrai si la session est verrouillée (ou si un écran sécurisé de Windows
/// est affiché, comme une demande d'autorisation administrateur). Méthode
/// courante : le bureau de saisie ne peut pas être ouvert dans ce cas.
pub fn is_session_locked() -> bool {
    unsafe {
        match OpenInputDesktop(DESKTOP_CONTROL_FLAGS(0), false, DESKTOP_SWITCHDESKTOP) {
            Ok(desk) => {
                let _ = CloseDesktop(desk);
                false
            }
            Err(_) => true,
        }
    }
}

/// Un bouton de la souris est-il enfoncé en ce moment ? Appelé seulement
/// pendant que l'on fait glisser la pastille, pour savoir quand on la lâche.
/// Les deux boutons sont regardés : Windows renvoie l'état des boutons
/// physiques, inversés pour les gauchers qui l'ont réglé ainsi.
pub fn mouse_button_down() -> bool {
    // Bit de poids fort : bouton enfoncé au moment de l'appel.
    unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) < 0 || GetAsyncKeyState(VK_RBUTTON.0 as i32) < 0 }
}
