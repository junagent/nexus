use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A skill definition loaded from ~/.nexus/skills/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDef {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub author: String,
    /// System prompt additions to inject when skill is active
    #[serde(default)]
    pub system_prompt: String,
    /// Optional shell commands (executed on skill activation)
    #[serde(default)]
    pub on_activate: Vec<String>,
    /// Optional shell commands (executed on skill deactivation)
    #[serde(default)]
    pub on_deactivate: Vec<String>,
    /// Whether this skill is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_true() -> bool { true }

/// The skills store manages loading, listing, and hot-reloading skills.
pub struct SkillStore {
    /// Loaded skills: name -> SkillDef
    skills: HashMap<String, SkillDef>,
    /// The directory where skills are stored
    skills_dir: std::path::PathBuf,
}

impl std::fmt::Debug for SkillStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkillStore")
            .field("skills_count", &self.skills.len())
            .field("enabled_count", &self.skills.values().filter(|s| s.enabled).count())
            .finish()
    }
}

impl SkillStore {
    /// Create a new SkillStore, loading from the given directory.
    pub fn load(skills_dir: &std::path::Path) -> Self {
        let mut store = Self {
            skills: HashMap::new(),
            skills_dir: skills_dir.to_path_buf(),
        };
        store.reload();
        store
    }

    /// Reload all skills from disk.
    pub fn reload(&mut self) {
        self.skills.clear();
        let _ = std::fs::create_dir_all(&self.skills_dir);

        if let Ok(entries) = std::fs::read_dir(&self.skills_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(true, |e| e != "yaml" && e != "yml" && e != "json") {
                    continue;
                }
                match std::fs::read_to_string(&path) {
                    Ok(contents) => {
                        let skill: SkillDef = if path.extension().map_or(false, |e| e == "json") {
                            serde_json::from_str(&contents)
                        } else {
                            serde_yaml::from_str(&contents)
                        }.map_err(|e| tracing::warn!("Failed to parse skill {}: {}", path.display(), e)).ok();

                        if let Some(skill) = skill {
                            tracing::info!("Loaded skill: {} v{}", skill.name, skill.version);
                            self.skills.insert(skill.name.clone(), skill);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to read skill {}: {}", path.display(), e);
                    }
                }
            }
        }
        tracing::info!("SkillStore: {} skills loaded from {}", self.skills.len(), self.skills_dir.display());
    }

    /// List all skills.
    pub fn list(&self) -> Vec<&SkillDef> {
        let mut skills: Vec<&SkillDef> = self.skills.values().collect();
        skills.sort_by(|a, b| a.name.cmp(&b.name));
        skills
    }

    /// Get a specific skill by name.
    pub fn get(&self, name: &str) -> Option<&SkillDef> {
        self.skills.get(name)
    }

    /// Enable or disable a skill.
    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> bool {
        if let Some(skill) = self.skills.get_mut(name) {
            skill.enabled = enabled;
            // Persist the change
            let path = self.skills_dir.join(format!("{}.yaml", name));
            if let Ok(contents) = serde_yaml::to_string(&skill) {
                let _ = std::fs::write(&path, contents);
            }
            true
        } else {
            false
        }
    }

    /// Remove a skill file.
    pub fn remove(&mut self, name: &str) -> bool {
        if self.skills.remove(name).is_some() {
            let path = self.skills_dir.join(format!("{}.yaml", name));
            let _ = std::fs::remove_file(&path);
            true
        } else {
            false
        }
    }

    /// Install a skill from source (GitHub repo URL or local path).
    pub async fn install(&mut self, source: &str) -> Result<String, String> {
        // Handle GitHub repos
        if source.starts_with("https://github.com/") || source.starts_with("git@github.com:") {
            let name = source.split('/').last()
                .unwrap_or(source)
                .trim_end_matches(".git")
                .to_string();

            let clone_dir = self.skills_dir.join("_tmp");
            let _ = std::fs::remove_dir_all(&clone_dir);

            let status = std::process::Command::new("git")
                .args(["clone", "--depth", "1", source, clone_dir.to_str().unwrap_or("_tmp")])
                .status()
                .map_err(|e| format!("Git clone failed: {}", e))?;

            if !status.success() {
                return Err("Git clone failed".to_string());
            }

            // Copy skill files from cloned repo
            if let Ok(entries) = std::fs::read_dir(&clone_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(true, |e| e != "yaml" && e != "yml" && e != "json") {
                        continue;
                    }
                    let dest = self.skills_dir.join(path.file_name().unwrap());
                    let _ = std::fs::copy(&path, &dest);
                }
            }

            let _ = std::fs::remove_dir_all(&clone_dir);
            self.reload();
            return Ok(format!("Installed skill from {}", source));
        }

        // Handle direct file path
        let src = std::path::Path::new(source);
        if src.exists() && src.is_file() {
            let dest = self.skills_dir.join(src.file_name().unwrap());
            std::fs::copy(src, &dest).map_err(|e| format!("Copy failed: {}", e))?;
            self.reload();
            return Ok(format!("Installed skill from {}", source));
        }

        Err(format!("Unknown source: {}. Use a GitHub URL or local file path.", source))
    }

    /// Build the system prompt additions from all enabled skills.
    pub fn build_system_prompt(&self) -> String {
        let mut prompt = String::new();
        for skill in self.skills.values() {
            if skill.enabled && !skill.system_prompt.is_empty() {
                prompt.push_str(&skill.system_prompt);
                prompt.push('\n');
            }
        }
        prompt
    }

    /// Get the skills directory path.
    pub fn dir(&self) -> &std::path::Path {
        &self.skills_dir
    }
}