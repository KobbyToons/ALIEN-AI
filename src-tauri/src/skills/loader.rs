use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::RwLock;

use super::manifest::{LoadedSkill, SkillManifest};

pub struct SkillLoader {
    skills_dir: PathBuf,
    loaded_skills: RwLock<Vec<LoadedSkill>>,
}

impl SkillLoader {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let skills_dir = app_data_dir.join("skills");
        if !skills_dir.exists() {
            let _ = fs::create_dir_all(&skills_dir);
        }

        Self {
            skills_dir,
            loaded_skills: RwLock::new(Vec::new()),
        }
    }

    /// Scan skills directory and load all manifests
    pub fn reload(&self) -> Result<Vec<LoadedSkill>, String> {
        let mut results = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        // Comprehensive search directories across app_data, binary location, and workspace
        let mut search_dirs = vec![self.skills_dir.clone(), PathBuf::from("./skills")];

        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                search_dirs.push(exe_dir.join("skills"));
                if let Some(p1) = exe_dir.parent() {
                    search_dirs.push(p1.join("skills"));
                    if let Some(p2) = p1.parent() {
                        search_dirs.push(p2.join("skills"));
                        if let Some(p3) = p2.parent() {
                            search_dirs.push(p3.join("skills"));
                            if let Some(p4) = p3.parent() {
                                search_dirs.push(p4.join("skills"));
                            }
                        }
                    }
                }
            }
        }

        // Also check legacy com.buddy.app skills folder
        if let Some(legacy) = dirs::data_dir().map(|d| d.join("com.buddy.app").join("skills")) {
            search_dirs.push(legacy);
        }

        for dir in search_dirs {
            if !dir.exists() {
                continue;
            }
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let manifest_path = path.join("manifest.json");
                        if manifest_path.exists() {
                            if let Ok(content) = fs::read_to_string(&manifest_path) {
                                if let Ok(manifest) = serde_json::from_str::<SkillManifest>(&content) {
                                    if !seen_ids.contains(&manifest.id) {
                                        seen_ids.insert(manifest.id.clone());
                                        results.push(LoadedSkill {
                                            manifest,
                                            directory: path.to_string_lossy().to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Ok(mut lock) = self.loaded_skills.write() {
            *lock = results.clone();
        }

        Ok(results)
    }

    pub fn get_skills(&self) -> Vec<LoadedSkill> {
        self.loaded_skills.read().map(|l| l.clone()).unwrap_or_default()
    }

    /// Execute a skill's custom script handler
    #[allow(dead_code)]
    pub fn execute_handler(
        &self,
        skill_id: &str,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> Result<String, String> {
        let skills = self.get_skills();
        let skill = skills
            .iter()
            .find(|s| s.manifest.id == skill_id)
            .ok_or_else(|| format!("Skill not found: {skill_id}"))?;

        let tool = skill
            .manifest
            .tools
            .iter()
            .find(|t| t.name == tool_name)
            .ok_or_else(|| format!("Tool {tool_name} not found in skill {skill_id}"))?;

        let handler_path = Path::new(&skill.directory).join(&tool.handler);
        if !handler_path.exists() {
            return Err(format!("Handler script does not exist: {:?}", handler_path));
        }

        let arg_str = serde_json::to_string(arguments).unwrap_or_default();

        let output = if cfg!(target_os = "windows") {
            let ext = handler_path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if ext == "ps1" {
                Command::new("powershell")
                    .arg("-ExecutionPolicy")
                    .arg("Bypass")
                    .arg("-File")
                    .arg(&handler_path)
                    .arg(&arg_str)
                    .output()
            } else {
                Command::new(&handler_path).arg(&arg_str).output()
            }
        } else {
            Command::new(&handler_path).arg(&arg_str).output()
        }
        .map_err(|e| format!("Failed to run skill handler: {e}"))?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

