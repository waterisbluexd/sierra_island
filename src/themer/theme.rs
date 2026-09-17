use serde::Deserialize;
use std::{env, fs, path::PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct Special {
    pub background: String,
    pub foreground: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub cursor: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Colors {
    pub color0: String,
    pub color1: String,
    pub color2: String,
    pub color3: String,
    pub color4: String,
    pub color5: String,
    pub color6: String,
    pub color7: String,
    pub color8: String,
    pub color9: String,
    pub color10: String,
    pub color11: String,
    pub color12: String,
    pub color13: String,
    pub color14: String,
    pub color15: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    pub special: Special,
    pub colors: Colors,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            special: Special {
                background: "#19120f".into(),
                foreground: "#cbccbe".into(),
                cursor: "#cbccbe".into(),
            },
            colors: Colors {
                color0: "#19120f".into(),
                color1: "#7E8979".into(),
                color2: "#978B6A".into(),
                color3: "#CAAF73".into(),
                color4: "#8E998B".into(),
                color5: "#A59F8A".into(),
                color6: "#98A390".into(),
                color7: "#cbccbe".into(),
                color8: "#8e8e85".into(),
                color9: "#7E8979".into(),
                color10: "#978B6A".into(),
                color11: "#CAAF73".into(),
                color12: "#8E998B".into(),
                color13: "#A59F8A".into(),
                color14: "#98A390".into(),
                color15: "#cbccbe".into(),
            },
        }
    }
}

impl Theme {
    pub fn path() -> PathBuf {
        PathBuf::from(env::var("HOME").expect("HOME environment variable not set"))
            .join(".cache/wal/colors.json")
    }

    pub fn load() -> Self {
        fs::read_to_string(Self::path())
            .ok()
            .filter(|s| !s.trim().is_empty())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| {
                eprintln!("Failed to load pywal colors.json, using default theme");
                Self::default()
            })
    }
}
