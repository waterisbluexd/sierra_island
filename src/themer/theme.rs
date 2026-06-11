#[derive(Debug, Clone)]
pub struct Theme {
    pub background: String,
    pub foreground: String,
    pub colors: [String; 16],
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: "#1e1e2e".to_string(),
            foreground: "#cdd6f4".to_string(),

            colors: [
                "#45475a".to_string(), // color0
                "#f38ba8".to_string(), // color1
                "#a6e3a1".to_string(), // color2
                "#f9e2af".to_string(), // color3
                "#89b4fa".to_string(), // color4
                "#f5c2e7".to_string(), // color5
                "#94e2d5".to_string(), // color6
                "#bac2de".to_string(), // color7
                "#585b70".to_string(), // color8
                "#f38ba8".to_string(), // color9
                "#a6e3a1".to_string(), // color10
                "#f9e2af".to_string(), // color11
                "#89b4fa".to_string(), // color12
                "#f5c2e7".to_string(), // color13
                "#94e2d5".to_string(), // color14
                "#a6adc8".to_string(), // color15
            ],
        }
    }
}

impl Theme {
    #[inline]
    pub fn color(&self, index: usize) -> &str {
        &self.colors[index]
    }
}
