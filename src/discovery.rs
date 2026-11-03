use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DiscoveredApplication {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub file_path: PathBuf,
    #[allow(dead_code)]
    pub exec: Option<String>,
}

#[derive(Debug, Default)]
pub struct ApplicationDiscovery {
    applications: HashMap<String, DiscoveredApplication>,
}

impl ApplicationDiscovery {
    /// Discover applications from standard Linux desktop file locations
    pub fn discover_applications(&mut self) -> io::Result<()> {
        // Scan system and user directories
        let system_dir = PathBuf::from("/usr/share/applications");
        let user_dir = match home::home_dir() {
            Some(home) => home.join(".local/share/applications"),
            None => return Ok(()),
        };

        // Scan system directory
        if system_dir.exists() && system_dir.is_dir() {
            self.scan_directory(&system_dir)?;
        }

        // Scan user directory
        if user_dir.exists() && user_dir.is_dir() {
            self.scan_directory(&user_dir)?;
        }

        Ok(())
    }

    /// Scan a directory for desktop files
    fn scan_directory(&mut self, dir: &Path) -> io::Result<()> {
        if !dir.exists() || !dir.is_dir() {
            return Ok(());
        }

        let entries = fs::read_dir(dir)?;

        for entry in entries.flatten() {
            if let Ok(app) = self.parse_desktop_file(&entry.path()) {
                self.applications.insert(app.name.clone(), app);
            }
        }

        Ok(())
    }

    /// Parse a desktop file and extract application information
    fn parse_desktop_file(&self, path: &Path) -> io::Result<DiscoveredApplication> {
        if !path.is_file() || !path.extension().is_some_and(|ext| ext == "desktop") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not a desktop file",
            ));
        }

        let mut content = String::new();
        let mut file = fs::File::open(path)?;
        file.read_to_string(&mut content)?;

        let mut name = String::new();
        let mut hidden = false;
        let mut nodisplay = false;
        let mut exec = None;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "Name" => {
                        if name.is_empty() {
                            name = value.trim().to_string();
                        }
                    }
                    "Hidden" => hidden = value.trim().to_lowercase() == "true",
                    "NoDisplay" => nodisplay = value.trim().to_lowercase() == "true",
                    "Exec" => exec = Some(value.trim().to_string()),
                    _ => {}
                }
            }
        }

        // Skip hidden or non-displayable applications
        if hidden || nodisplay || name.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Skipped by filter",
            ));
        }

        Ok(DiscoveredApplication {
            name,
            file_path: path.to_path_buf(),
            exec,
        })
    }

    /// Search applications by name (case-insensitive)
    pub fn search_applications(&self, query: &str) -> Vec<&DiscoveredApplication> {
        if query.is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();

        self.applications
            .values()
            .filter(|app| app.name.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Get all discovered application names
    pub fn get_application_names(&self) -> Vec<String> {
        self.applications
            .values()
            .map(|app| app.name.clone())
            .collect()
    }

    /// Get count of discovered applications
    pub fn count(&self) -> usize {
        self.applications.len()
    }

    /// Print diagnostic information about discovered applications
    pub fn print_diagnostic(&self) {
        let count = self.count();
        let names = self.get_application_names();

        println!("=== Application Discovery Diagnostic ===");
        println!("Total applications discovered: {}", count);

        if !names.is_empty() {
            println!("\nSample application names (showing first 10):");
            for (i, name) in names.iter().take(10).enumerate() {
                println!("  {}. {}", i + 1, name);
            }
        }

        println!("\n=== End Diagnostic ===");
    }

    /// Print search diagnostic information
    pub fn print_search_diagnostic(&self, query: &str) {
        let matches = self.search_applications(query);

        println!("=== Application Search Diagnostic ===");
        println!("Query: '{}'", query);
        println!("Total matches: {}", matches.len());

        if !matches.is_empty() {
            println!("\nMatching application names:");
            for (i, app) in matches.iter().enumerate() {
                println!("  {}. {}", i + 1, app.name);
            }
        }

        println!("\n=== End Search Diagnostic ===");
    }
}
