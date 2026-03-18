mod commands;
mod config;
mod io;
mod parsing;
mod templates;

fn main() {
    if let Err(e) = run_cli() {
        eprintln!("uto CLI error: {e}");
        std::process::exit(1);
    }
}

fn run_cli() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    match args[1].as_str() {
        "init" => commands::init::run(&args[2..]),
        "run" => commands::run::run(&args[2..]),
        "report" => commands::report::run(&args[2..]),
        "ui" => commands::ui::run(&args[2..]),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        other => Err(format!(
            "Unknown command '{other}'. Supported commands: init, run, report, ui"
        )),
    }
}

fn print_help() {
    println!(
        "UTO CLI\n\n\
         Commands:\n  \
           uto init <project-dir> [--template web|mobile] [--uto-root <path>]\n  \
           uto run --project <project-dir> [--target web|mobile] [--report-json <path>] [--driver-trace]\n  \
           uto report --project <project-dir> [--input <report-path>] [--html] [--html-output <report.html>]\n  \
           uto ui [--project <project-dir>] [--port <port>] [--open] [--watch] [--report <report-path>]"
    );
}
