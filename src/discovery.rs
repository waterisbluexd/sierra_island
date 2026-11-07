use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DiscoveredApplication {
    pub name: String,
    pub display_name: String,
    pub search_name: String,
    pub search_metadata: String,
    pub exec: Option<String>,
}

#[derive(Debug, Default)]
pub struct ApplicationDiscovery {
    applications: HashMap<String, DiscoveredApplication>,
}

impl ApplicationDiscovery {
    /// Discover applications from standard Linux desktop file locations
    pub fn discover_applications(&mut self) -> io::Result<()> {
        let system_dir = PathBuf::from("/usr/share/applications");
        let user_dir = match home::home_dir() {
            Some(home) => home.join(".local/share/applications"),
            None => return Ok(()),
        };

        if system_dir.exists() && system_dir.is_dir() {
            self.scan_directory(&system_dir)?;
        }

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
        let mut display_name = String::new();
        let mut generic_name = String::new();
        let mut keywords = String::new();
        let mut hidden = false;
        let mut nodisplay = false;
        let mut exec = None;

        // Desktop file spec: only the [Desktop Entry] group counts for primary Name.
        // Subsequent groups (Desktop Action *) are ignored for the primary display name.
        let mut in_desktop_entry = false;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Track group headers: only [Desktop Entry] provides the primary Name.
            if line.starts_with('[') && line.ends_with(']') {
                in_desktop_entry = line == "[Desktop Entry]";
                continue;
            }

            if !in_desktop_entry {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "Name" => {
                        if display_name.is_empty() {
                            display_name = value.trim().to_string();
                        }
                    }
                    "GenericName" => {
                        if generic_name.is_empty() {
                            generic_name = value.trim().to_string();
                        }
                    }
                    "Keywords" => {
                        if !keywords.is_empty() {
                            keywords.push(';');
                        }
                        keywords.push_str(value.trim());
                    }
                    "Hidden" => hidden = value.trim().to_lowercase() == "true",
                    "NoDisplay" => nodisplay = value.trim().to_lowercase() == "true",
                    "Exec" => exec = Some(value.trim().to_string()),
                    _ => {}
                }
            }
        }

        // Use the desktop file basename (without .desktop) as the unique identifier.
        if name.is_empty() {
            name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
        }

        // Fallback display name: use the identifier if no Name field exists.
        if display_name.is_empty() {
            display_name = name.clone();
        }

        // Skip hidden or non-displayable applications
        if hidden || nodisplay {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Skipped by filter",
            ));
        }

        let search_name = display_name.to_lowercase();
        let search_metadata = format!("{} {}", generic_name, keywords).to_lowercase();

        Ok(DiscoveredApplication {
            name,
            display_name,
            search_name,
            search_metadata,
            exec,
        })
    }

    /// Score a search match for an application against a normalized query.
    /// Lower score = better match. Returns None if no match.
    fn match_score(&self, query_lower: &str, app: &DiscoveredApplication) -> Option<u32> {
        let name_lower = &app.search_name;

        // Priority 1: display name starts with the query
        if name_lower.starts_with(query_lower) {
            return Some(1);
        }

        // Priority 2: a word in the display name starts with the query
        if name_lower
            .split_whitespace()
            .any(|word| word.starts_with(query_lower))
        {
            return Some(2);
        }

        // Priority 3: query appears elsewhere in the display name
        if name_lower.contains(query_lower) {
            return Some(3);
        }

        // Priority 4: metadata match (GenericName, Keywords) — only for queries of 2+ chars
        if query_lower.len() >= 2 && app.search_metadata.contains(query_lower) {
            return Some(4);
        }

        None
    }

    /// Search applications by query with priority-based ranking.
    pub fn search_applications(&self, query: &str) -> Vec<&DiscoveredApplication> {
        if query.is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();

        let mut scored: Vec<(u32, &DiscoveredApplication)> = self
            .applications
            .values()
            .filter_map(|app| {
                self.match_score(&query_lower, app)
                    .map(|score| (score, app))
            })
            .collect();

        // Sort by score (better match first), then alphabetically by display name for stability.
        scored.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.display_name.cmp(&b.1.display_name))
        });

        scored.into_iter().map(|(_, app)| app).collect()
    }

    /// Get a discovered application by its desktop-file identifier.
    pub fn get_application(&self, name: &str) -> Option<&DiscoveredApplication> {
        self.applications.get(name)
    }

    /// Get all discovered application display names
    pub fn get_application_names(&self) -> Vec<String> {
        self.applications
            .values()
            .map(|app| app.display_name.clone())
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
                println!("  {}. {}", i + 1, app.display_name);
            }
        }

        println!("\n=== End Diagnostic ===");
    }
}
