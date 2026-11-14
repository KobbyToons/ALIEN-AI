use std::collections::HashSet;
use std::sync::RwLock;

use crate::services::obsidian::ObsidianVault;
use super::schema::{LearningConfig, ScenarioQuestion, ThinkingProfile};

pub struct LearningService {
    config: RwLock<LearningConfig>,
    questions: Vec<ScenarioQuestion>,
    answered: RwLock<HashSet<String>>,
}

impl LearningService {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(LearningConfig::default()),
            answered: RwLock::new(HashSet::new()),
            questions: vec![
                ScenarioQuestion {
                    id: "scarlet_crystal_narrative".into(),
                    category: "Webtoon".into(),
                    question: "When plotting story arcs for 'The SCARLET Crystal', which narrative driver matters most to you?".into(),
                    options: vec![
                        "High-stakes psychological character conflicts".into(),
                        "Fast-paced combat, dynamic powers & explosive climaxes".into(),
                        "Deep ancient lore, hidden conspiracies & mystery revelations".into(),
                    ],
                },
                ScenarioQuestion {
                    id: "csp_art_process".into(),
                    category: "DigitalArt".into(),
                    question: "In Clip Studio Paint, what visual stage should I prioritize assisting you with?".into(),
                    options: vec![
                        "Dynamic panel compositions & perspective storyboarding".into(),
                        "Color keys, atmospheric lighting & explosive VFX effects".into(),
                        "Polished lineart cleanups & anatomically tight character inks".into(),
                    ],
                },
                ScenarioQuestion {
                    id: "blender_staging".into(),
                    category: "3DScene".into(),
                    question: "When constructing 3D scene backgrounds in Blender, what's your preferred approach?".into(),
                    options: vec![
                        "Cinematic wide lenses with heavy atmospheric volumetric fog".into(),
                        "Modular layout props focused on rapid Webtoon perspective tracing".into(),
                        "Photorealistic lighting and material textures ready to render".into(),
                    ],
                },
                ScenarioQuestion {
                    id: "music_scoring".into(),
                    category: "AudioMusic".into(),
                    question: "When producing music and theme tracks for your projects, what energy level fits best?".into(),
                    options: vec![
                        "Aggressive cyber-electronic beats with heavy bass & synths".into(),
                        "Orchestral cinematic emotion that builds toward heroic peaks".into(),
                        "Moody lo-fi ambient tones that set a tense mystery atmosphere".into(),
                    ],
                },
                ScenarioQuestion {
                    id: "discord_autonomy".into(),
                    category: "AutonomousDiscord".into(),
                    question: "For autonomous Discord operations, how should I handle messaging collaborators?".into(),
                    options: vec![
                        "Draft the message and wait for your one-click approval".into(),
                        "Send routine status updates automatically, ask for sensitive ones".into(),
                        "Act fully autonomously under your guidelines".into(),
                    ],
                },
                ScenarioQuestion {
                    id: "research_depth".into(),
                    category: "DeepResearch".into(),
                    question: "When you ask me to do a deep research dive, how should findings be formatted in Obsidian?".into(),
                    options: vec![
                        "Structured wiki note with bi-directional links to existing lore".into(),
                        "Executive bullet points, timeline comparisons & actionable takeaways".into(),
                        "Full comprehensive dossier with source references and citations".into(),
                    ],
                },
                ScenarioQuestion {
                    id: "creative_feedback".into(),
                    category: "CreativePartner".into(),
                    question: "When brainstorming creative concepts with you, what feedback style do you value most?".into(),
                    options: vec![
                        "Brutally honest critique focused on pacing flaws & plot holes".into(),
                        "Enthusiastic expansion with wild 'what-if' twists".into(),
                        "Balanced analytical critique with three alternate directions".into(),
                    ],
                },
                ScenarioQuestion {
                    id: "character_dialogue".into(),
                    category: "WritingStyle".into(),
                    question: "How should dialogue sound in your story scripts?".into(),
                    options: vec![
                        "Punchy, sarcastic & sharp with quick banter".into(),
                        "Epic, intense & emotionally resonant".into(),
                        "Grounded, authentic & subtext-heavy".into(),
                    ],
                },
                ScenarioQuestion {
                    id: "obsidian_linking".into(),
                    category: "VaultOrganization".into(),
                    question: "How should our Obsidian Vault evolve as we work together?".into(),
                    options: vec![
                        "Atomic interconnected concept cards with lots of tags and links".into(),
                        "Comprehensive master binders grouped by project (Art, Story, Music)".into(),
                        "Chronological journal logs that summarize daily breakthroughs".into(),
                    ],
                },
                ScenarioQuestion {
                    id: "autonomy_boundary".into(),
                    category: "SystemAutonomy".into(),
                    question: "When executing multi-step complex tasks, how much creative freedom should I take?".into(),
                    options: vec![
                        "Take proactive initiative and execute the best route autonomously".into(),
                        "Follow your explicit steps exactly without adding unasked variations".into(),
                        "Execute the plan but offer creative alternative proposals at each checkpoint".into(),
                    ],
                },
                ScenarioQuestion {
                    id: "response_brevity".into(),
                    category: "Brevity".into(),
                    question: "For everyday quick voice and chat replies, what pacing feels most natural?".into(),
                    options: vec![
                        "Crisp, ultra-fast & straight to the core point".into(),
                        "Casual, conversational & human-speed with concise reasoning".into(),
                        "Detailed, thorough & layered with strategic context".into(),
                    ],
                },
                ScenarioQuestion {
                    id: "personality_tone".into(),
                    category: "IdentityTone".into(),
                    question: "What personality tone fits your creative companion best?".into(),
                    options: vec![
                        "Sharp, high-tech cybernetic ally who thinks two steps ahead".into(),
                        "Calm, razor-focused creative specialist".into(),
                        "Direct, energetic and witty creative partner".into(),
                    ],
                },
            ],
        }
    }

    pub fn get_config(&self) -> LearningConfig {
        self.config.read().map(|c| c.clone()).unwrap_or_default()
    }

    pub fn update_config(&self, cfg: LearningConfig) {
        if let Ok(mut lock) = self.config.write() {
            *lock = cfg;
        }
    }

    /// Retrieve the next unanswered question if eligible (enforcing 2-hour cooldown)
    pub fn get_next_question(&self) -> Option<ScenarioQuestion> {
        let cfg = self.get_config();
        if !cfg.scenario_questions {
            return None;
        }

        let now_sec = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // 2 hours = 7200 seconds cooldown between questions
        if let Some(last_time) = cfg.last_question_time {
            if now_sec < last_time + 7200 {
                return None;
            }
        }

        let answered_lock = self.answered.read().ok()?;
        let unanswered: Vec<ScenarioQuestion> = self
            .questions
            .iter()
            .filter(|q| !answered_lock.contains(&q.id))
            .cloned()
            .collect();

        if unanswered.is_empty() {
            return None;
        }

        // Randomize selection so questions don't appear in predictable hardcoded order
        let index = (now_sec as usize) % unanswered.len();
        let selected = unanswered[index].clone();

        // Update last question timestamp so cooldown starts immediately
        if let Ok(mut lock) = self.config.write() {
            lock.last_question_time = Some(now_sec);
        }

        Some(selected)
    }

    /// Save the user's answer into memory, reset cooldown, and write to the Obsidian vault
    pub fn answer_question(
        &self,
        vault: &ObsidianVault,
        question_id: &str,
        answer: &str,
    ) -> Result<(), String> {
        let q = self
            .questions
            .iter()
            .find(|q| q.id == question_id)
            .ok_or_else(|| "Question not found".to_string())?;

        let now_sec = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // 1. Mark as answered in memory and record cooldown timestamp
        if let Ok(mut lock) = self.answered.write() {
            lock.insert(question_id.to_string());
        }
        if let Ok(mut lock) = self.config.write() {
            lock.last_question_time = Some(now_sec);
        }

        // 2. Persist answer into Obsidian vault if connected
        let note_content = format!(
            "- **Topic**: {}\n- **Question**: {}\n- **Your Choice**: {}\n- **Mindset Rule**: User prefers {} style decisions.\n- **Recorded**: {}",
            q.category,
            q.question,
            answer,
            answer,
            chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
        );

        let _ = vault.write_note("Preferences", &q.category, &note_content);
        let _ = vault.write_note("Preferences", "User Mindset & Decision Rules", &note_content);

        Ok(())
    }

    /// Synthesize an active "How the user thinks" profile
    pub fn get_thinking_profile(&self, vault: &ObsidianVault) -> ThinkingProfile {
        let mut mindset_points = Vec::new();

        // Query preferences from vault
        if let Ok(notes) = vault.search("Preferences") {
            for note in notes.iter().take(6) {
                let first_line = note.content.lines().find(|l| l.starts_with('-')).unwrap_or("");
                if !first_line.is_empty() {
                    mindset_points.push(first_line.to_string());
                }
            }
        }

        if mindset_points.is_empty() {
            mindset_points.push("- Values dynamic visual pacing, Webtoon cinematic storytelling & non-destructive operations.".into());
            mindset_points.push("- Thinks through iterative creative sprints with fast autonomous execution.".into());
        }

        ThinkingProfile {
            summary: mindset_points.join("\n"),
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Format for system prompt injection
    pub fn format_prompt_block(&self, vault: &ObsidianVault) -> String {
        let cfg = self.get_config();
        if !cfg.passive_learning {
            return String::new();
        }

        let profile = self.get_thinking_profile(vault);
        if profile.summary.is_empty() {
            return String::new();
        }

        format!(
            "\n[User Mindset & Preferred Decision Style]:\n{}\n(Always align autonomous proposals and creative outputs to these preferences)\n",
            profile.summary
        )
    }
}
