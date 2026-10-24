//! Écrit les SVG des icônes : `cargo run -p pulse-core --example icons -- <dossier>`.
use pulse_core::mascot::{icon_svg, simple_svg, BLUE};

fn main() {
    let dir = std::env::args().nth(1).expect("dossier de sortie");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/app-icon.svg"), icon_svg(BLUE, 1024)).unwrap();
    std::fs::write(format!("{dir}/tray-active.svg"), simple_svg(BLUE, false, 64)).unwrap();
    std::fs::write(format!("{dir}/tray-paused.svg"), simple_svg("#8B9BB4", true, 64)).unwrap();
}
