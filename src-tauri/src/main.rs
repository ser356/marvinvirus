#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 4 && args[1] == "--apply-elevated" {
        let code = match marvinvirus_lib::apply_elevated_cli(&args[2], &args[3]) {
            Ok(()) => 0,
            Err(_) => 1,
        };
        std::process::exit(code);
    }
    marvinvirus_lib::run()
}
