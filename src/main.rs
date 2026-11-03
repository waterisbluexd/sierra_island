mod containers;
mod discovery;
mod theme;
mod themer;
mod wayland;
mod widgets;

slint::include_modules!();

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--discover".to_string()) {
        let mut discovery = discovery::ApplicationDiscovery::default();

        match discovery.discover_applications() {
            Ok(_) => {
                discovery.print_diagnostic();
            }
            Err(e) => {
                eprintln!("Failed to discover applications: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    if let Some(query_pos) = args.iter().position(|arg| arg == "--search") {
        if query_pos + 1 < args.len() {
            let query = &args[query_pos + 1];
            let mut discovery = discovery::ApplicationDiscovery::default();

            match discovery.discover_applications() {
                Ok(_) => {
                    discovery.print_search_diagnostic(query);
                }
                Err(_) => {
                    eprintln!("Failed to discover applications");
                    std::process::exit(1);
                }
            }
            return;
        } else {
            eprintln!("Error: --search requires a query argument");
            std::process::exit(1);
        }
    }

    wayland::run();
}
