use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A single arm in the multi-armed bandit: a provider+model combo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanditArm {
    pub provider: String,
    pub model: String,
    pub trials: u32,
    pub successes: u32,
    pub total_latency_ms: f64,
    pub total_cost: f64,
    pub last_used: i64, // unix timestamp
}

impl BanditArm {
    pub fn new(provider: &str, model: &str) -> Self {
        Self {
            provider: provider.to_string(),
            model: model.to_string(),
            trials: 0,
            successes: 0,
            total_latency_ms: 0.0,
            total_cost: 0.0,
            last_used: 0,
        }
    }

    /// UCB1 score: exploitation + exploration
    pub fn ucb1_score(&self, total_trials: u32) -> f64 {
        if self.trials == 0 {
            return f64::MAX; // Untried arms get maximum exploration bonus
        }
        let exploitation = self.successes as f64 / self.trials as f64;
        let exploration = (2.0 * (total_trials as f64).ln() / self.trials as f64).sqrt();
        exploitation + exploration
    }

    /// Average latency in ms
    pub fn avg_latency(&self) -> f64 {
        if self.trials == 0 { 0.0 } else { self.total_latency_ms / self.trials as f64 }
    }

    /// Average cost per call
    pub fn avg_cost(&self) -> f64 {
        if self.trials == 0 { 0.0 } else { self.total_cost / self.trials as f64 }
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.trials == 0 { 0.0 } else { self.successes as f64 / self.trials as f64 }
    }
}

/// Multi-armed bandit selector for automatic provider/model routing.
pub struct BanditSelector {
    arms: HashMap<String, BanditArm>,
    pub total_trials: u32,
    db: Option<std::sync::Mutex<rusqlite::Connection>>,
}

impl std::fmt::Debug for BanditSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BanditSelector")
            .field("arms", &self.arms)
            .field("total_trials", &self.total_trials)
            .finish()
    }
}

impl BanditSelector {
    pub fn new(db_path: &str) -> Self {
        let db = rusqlite::Connection::open(db_path).ok();
        if let Some(ref conn) = db {
            let _ = conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS bandit_arms (
                    provider TEXT NOT NULL,
                    model TEXT NOT NULL,
                    trials INTEGER NOT NULL DEFAULT 0,
                    successes INTEGER NOT NULL DEFAULT 0,
                    total_latency_ms REAL NOT NULL DEFAULT 0.0,
                    total_cost REAL NOT NULL DEFAULT 0.0,
                    last_used INTEGER NOT NULL DEFAULT 0,
                    PRIMARY KEY (provider, model)
                );"
            );
        }

        let mut selector = Self {
            arms: HashMap::new(),
            total_trials: 0,
            db: db.map(|d| std::sync::Mutex::new(d)),
        };

        // Load existing arms from DB
        selector.load_from_db();
        selector
    }

    fn load_from_db(&mut self) {
        if let Some(ref db) = self.db {
            if let Ok(db) = db.lock() {
                let mut stmt = db.prepare(
                    "SELECT provider, model, trials, successes, total_latency_ms, total_cost, last_used FROM bandit_arms"
                ).ok();
                if let Some(ref mut stmt) = stmt {
                    if let Ok(rows) = stmt.query_map([], |row| {
                        Ok(BanditArm {
                            provider: row.get(0)?,
                            model: row.get(1)?,
                            trials: row.get(2)?,
                            successes: row.get(3)?,
                            total_latency_ms: row.get(4)?,
                            total_cost: row.get(5)?,
                            last_used: row.get(6)?,
                        })
                    }) {
                        for row in rows.flatten() {
                            let key = format!("{}/{}", row.provider, row.model);
                            self.total_trials += row.trials;
                            self.arms.insert(key, row);
                        }
                    }
                }
            }
        }
    }

    /// Register a new arm (provider+model combination).
    pub fn register_arm(&mut self, provider: &str, model: &str) {
        let key = format!("{}/{}", provider, model);
        self.arms.entry(key).or_insert_with(|| {
            let arm = BanditArm::new(provider, model);
            if let Some(ref db) = self.db {
                if let Ok(db) = db.lock() {
                    let _ = db.execute(
                        "INSERT OR IGNORE INTO bandit_arms (provider, model, trials, successes, total_latency_ms, total_cost, last_used) VALUES (?1, ?2, 0, 0, 0.0, 0.0, 0)",
                        rusqlite::params![provider, model],
                    );
                }
            }
            arm
        });
    }

    /// Select the best arm using UCB1.
    /// Returns (provider, model).
    pub fn select(&self, preferred_provider: Option<&str>) -> Option<(String, String)> {
        if self.arms.is_empty() {
            return None;
        }

        // If preferred provider is set, use it
        if let Some(provider) = preferred_provider {
            let candidates: Vec<&BanditArm> = self.arms.values()
                .filter(|a| a.provider == provider)
                .collect();
            if !candidates.is_empty() {
                let best = candidates.iter()
                    .max_by(|a, b| a.ucb1_score(self.total_trials).partial_cmp(&b.ucb1_score(self.total_trials)).unwrap_or(std::cmp::Ordering::Equal))
                    .unwrap();
                return Some((best.provider.clone(), best.model.clone()));
            }
        }

        // Otherwise pick the global best arm (UCB1)
        let best = self.arms.values()
            .max_by(|a, b| a.ucb1_score(self.total_trials).partial_cmp(&b.ucb1_score(self.total_trials)).unwrap_or(std::cmp::Ordering::Equal));

        best.map(|b| (b.provider.clone(), b.model.clone()))
    }

    /// Record a successful completion.
    pub fn record_success(&mut self, provider: &str, model: &str, latency_ms: f64, cost: f64) {
        let key = format!("{}/{}", provider, model);
        if let Some(arm) = self.arms.get_mut(&key) {
            arm.trials += 1;
            arm.successes += 1;
            arm.total_latency_ms += latency_ms;
            arm.total_cost += cost;
            arm.last_used = chrono::Utc::now().timestamp();
            self.total_trials += 1;
            self.persist_arm(arm);
        }
    }

    /// Record a failed attempt.
    pub fn record_failure(&mut self, provider: &str, model: &str, latency_ms: f64, cost: f64) {
        let key = format!("{}/{}", provider, model);
        if let Some(arm) = self.arms.get_mut(&key) {
            arm.trials += 1;
            arm.total_latency_ms += latency_ms;
            arm.total_cost += cost;
            arm.last_used = chrono::Utc::now().timestamp();
            self.total_trials += 1;
            self.persist_arm(arm);
        }
    }

    fn persist_arm(&self, arm: &BanditArm) {
        if let Some(ref db) = self.db {
            if let Ok(db) = db.lock() {
                let _ = db.execute(
                    "UPDATE bandit_arms SET trials=?1, successes=?2, total_latency_ms=?3, total_cost=?4, last_used=?5 WHERE provider=?6 AND model=?7",
                    rusqlite::params![arm.trials, arm.successes, arm.total_latency_ms, arm.total_cost, arm.last_used, arm.provider, arm.model],
                );
            }
        }
    }

    /// Get stats summary for all arms.
    pub fn summary(&self) -> Vec<BanditArm> {
        let mut arms: Vec<BanditArm> = self.arms.values().cloned().collect();
        arms.sort_by(|a, b| b.ucb1_score(self.total_trials).partial_cmp(&a.ucb1_score(self.total_trials)).unwrap_or(std::cmp::Ordering::Equal));
        arms
    }
}

/// Pricing model for common providers (approximate per 1K tokens).
pub fn estimate_cost(provider: &str, model: &str, input_tokens: u32, output_tokens: u32) -> f64 {
    let (input_price, output_price) = match provider {
        "openai" => match model {
            "gpt-4o" => (0.0025, 0.01),
            "gpt-4o-mini" => (0.00015, 0.0006),
            "o3-mini" => (0.0011, 0.0044),
            _ => (0.001, 0.002),
        },
        "anthropic" => match model {
            "claude-sonnet-4" => (0.003, 0.015),
            "claude-3.5-haiku" => (0.0008, 0.004),
            _ => (0.002, 0.01),
        },
        "deepseek" => match model {
            "deepseek-chat" => (0.00014, 0.00028),
            "deepseek-reasoner" => (0.00055, 0.00219),
            _ => (0.00014, 0.00028),
        },
        "openrouter" => (0.001, 0.002), // varies widely
        "google" => match model {
            "gemini-2.0-flash" => (0.0001, 0.0004),
            "gemini-2.5-pro" => (0.00125, 0.005),
            _ => (0.0005, 0.0015),
        },
        _ => (0.001, 0.002),
    };
    (input_tokens as f64 / 1000.0) * input_price + (output_tokens as f64 / 1000.0) * output_price
}