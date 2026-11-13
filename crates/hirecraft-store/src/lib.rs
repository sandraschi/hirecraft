#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HireStoreError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateStage {
    Applied,
    Screening,
    Interview,
    Offer,
    Hired,
    Rejected,
}

impl CandidateStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::Screening => "screening",
            Self::Interview => "interview",
            Self::Offer => "offer",
            Self::Hired => "hired",
            Self::Rejected => "rejected",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "applied" => Some(Self::Applied),
            "screening" => Some(Self::Screening),
            "interview" => Some(Self::Interview),
            "offer" => Some(Self::Offer),
            "hired" => Some(Self::Hired),
            "rejected" => Some(Self::Rejected),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRequisition {
    pub id: String,
    pub title: String,
    pub department: String,
    pub status: String,
    pub target_headcount: usize,
    pub salary_min_eur: i64,
    pub salary_max_eur: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub id: String,
    pub req_id: String,
    pub name: String,
    pub email: String,
    pub stage: CandidateStage,
    pub scorecard_avg: f64,
    pub offered_salary_eur: Option<i64>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scorecard {
    pub id: String,
    pub candidate_id: String,
    pub interviewer: String,
    pub technical_score: i32,
    pub culture_score: i32,
    pub recommendation: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HireStats {
    pub open_reqs: usize,
    pub total_candidates: usize,
    pub interviewing_count: usize,
    pub offer_count: usize,
    pub hired_count: usize,
}

pub struct HireStore {
    conn: Connection,
}

impl HireStore {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, HireStoreError> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self, HireStoreError> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<(), HireStoreError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS job_requisitions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                department TEXT NOT NULL,
                status TEXT NOT NULL,
                target_headcount INTEGER NOT NULL,
                salary_min_eur INTEGER NOT NULL,
                salary_max_eur INTEGER NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS candidates (
                id TEXT PRIMARY KEY,
                req_id TEXT NOT NULL,
                name TEXT NOT NULL,
                email TEXT NOT NULL,
                stage TEXT NOT NULL,
                scorecard_avg REAL NOT NULL,
                offered_salary_eur INTEGER,
                notes TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (req_id) REFERENCES job_requisitions(id)
            );
            CREATE INDEX IF NOT EXISTS idx_candidates_stage ON candidates(stage);
            CREATE INDEX IF NOT EXISTS idx_candidates_req ON candidates(req_id);

            CREATE TABLE IF NOT EXISTS scorecards (
                id TEXT PRIMARY KEY,
                candidate_id TEXT NOT NULL,
                interviewer TEXT NOT NULL,
                technical_score INTEGER NOT NULL,
                culture_score INTEGER NOT NULL,
                recommendation TEXT NOT NULL,
                notes TEXT NOT NULL,
                FOREIGN KEY (candidate_id) REFERENCES candidates(id)
            );
            CREATE INDEX IF NOT EXISTS idx_scorecards_candidate ON scorecards(candidate_id);",
        )?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_req(
        &self,
        id: &str,
        title: &str,
        department: &str,
        target_headcount: usize,
        salary_min_eur: i64,
        salary_max_eur: i64,
    ) -> Result<JobRequisition, HireStoreError> {
        let now = Utc::now();
        let req = JobRequisition {
            id: id.to_string(),
            title: title.to_string(),
            department: department.to_string(),
            status: "open".to_string(),
            target_headcount,
            salary_min_eur,
            salary_max_eur,
            created_at: now,
        };

        self.conn.execute(
            "INSERT INTO job_requisitions (id, title, department, status, target_headcount, salary_min_eur, salary_max_eur, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                req.id,
                req.title,
                req.department,
                req.status,
                req.target_headcount as i64,
                req.salary_min_eur,
                req.salary_max_eur,
                req.created_at.to_rfc3339()
            ],
        )?;

        Ok(req)
    }

    pub fn list_reqs(&self) -> Result<Vec<JobRequisition>, HireStoreError> {
        let mut stmt = self.conn.prepare("SELECT id, title, department, status, target_headcount, salary_min_eur, salary_max_eur, created_at FROM job_requisitions ORDER BY id ASC")?;
        let rows = stmt.query_map([], |row| {
            let created_str: String = row.get(7)?;
            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            Ok(JobRequisition {
                id: row.get(0)?,
                title: row.get(1)?,
                department: row.get(2)?,
                status: row.get(3)?,
                target_headcount: row.get::<_, i64>(4)? as usize,
                salary_min_eur: row.get(5)?,
                salary_max_eur: row.get(6)?,
                created_at,
            })
        })?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn create_candidate(
        &self,
        id: &str,
        req_id: &str,
        name: &str,
        email: &str,
        notes: Option<&str>,
    ) -> Result<Candidate, HireStoreError> {
        let now = Utc::now();
        let cand = Candidate {
            id: id.to_string(),
            req_id: req_id.to_string(),
            name: name.to_string(),
            email: email.to_string(),
            stage: CandidateStage::Applied,
            scorecard_avg: 0.0,
            offered_salary_eur: None,
            notes: notes.map(|s| s.to_string()),
            created_at: now,
        };

        self.conn.execute(
            "INSERT INTO candidates (id, req_id, name, email, stage, scorecard_avg, offered_salary_eur, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                cand.id,
                cand.req_id,
                cand.name,
                cand.email,
                cand.stage.as_str(),
                cand.scorecard_avg,
                cand.offered_salary_eur,
                cand.notes,
                cand.created_at.to_rfc3339()
            ],
        )?;

        Ok(cand)
    }

    pub fn get_candidate(&self, id: &str) -> Result<Candidate, HireStoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, req_id, name, email, stage, scorecard_avg, offered_salary_eur, notes, created_at
             FROM candidates WHERE id = ?1",
        )?;

        stmt.query_row(params![id], |row| Self::row_to_candidate(row))
            .optional()?
            .ok_or_else(|| HireStoreError::NotFound(id.to_string()))
    }

    pub fn list_candidates(
        &self,
        stage: Option<CandidateStage>,
    ) -> Result<Vec<Candidate>, HireStoreError> {
        let mut sql = "SELECT id, req_id, name, email, stage, scorecard_avg, offered_salary_eur, notes, created_at FROM candidates".to_string();
        if stage.is_some() {
            sql.push_str(" WHERE stage = ?1");
        }
        sql.push_str(" ORDER BY created_at DESC");

        let mut stmt = self.conn.prepare(&sql)?;
        let mut out = Vec::new();
        if let Some(st) = stage {
            let rows = stmt.query_map(params![st.as_str()], Self::row_to_candidate)?;
            for r in rows {
                out.push(r?);
            }
        } else {
            let rows = stmt.query_map([], Self::row_to_candidate)?;
            for r in rows {
                out.push(r?);
            }
        }
        Ok(out)
    }

    pub fn update_candidate_stage(
        &self,
        id: &str,
        stage: CandidateStage,
        offered_salary_eur: Option<i64>,
    ) -> Result<(), HireStoreError> {
        let rows = if let Some(sal) = offered_salary_eur {
            self.conn.execute(
                "UPDATE candidates SET stage = ?1, offered_salary_eur = ?2 WHERE id = ?3",
                params![stage.as_str(), sal, id],
            )?
        } else {
            self.conn.execute(
                "UPDATE candidates SET stage = ?1 WHERE id = ?2",
                params![stage.as_str(), id],
            )?
        };

        if rows == 0 {
            return Err(HireStoreError::NotFound(id.to_string()));
        }
        Ok(())
    }

    fn row_to_candidate(row: &rusqlite::Row) -> Result<Candidate, rusqlite::Error> {
        let stage_str: String = row.get(4)?;
        let stage = CandidateStage::from_str_opt(&stage_str).unwrap_or(CandidateStage::Applied);
        let created_str: String = row.get(8)?;
        let created_at = DateTime::parse_from_rfc3339(&created_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(Candidate {
            id: row.get(0)?,
            req_id: row.get(1)?,
            name: row.get(2)?,
            email: row.get(3)?,
            stage,
            scorecard_avg: row.get(5)?,
            offered_salary_eur: row.get(6)?,
            notes: row.get(7)?,
            created_at,
        })
    }

    pub fn add_scorecard(
        &self,
        candidate_id: &str,
        interviewer: &str,
        technical_score: i32,
        culture_score: i32,
        recommendation: &str,
        notes: &str,
    ) -> Result<Scorecard, HireStoreError> {
        if !(1..=5).contains(&technical_score) || !(1..=5).contains(&culture_score) {
            return Err(HireStoreError::Validation(
                "scorecard scores must be in 1..=5".into(),
            ));
        }
        let sc_id = format!("SC-{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));
        let sc = Scorecard {
            id: sc_id,
            candidate_id: candidate_id.to_string(),
            interviewer: interviewer.to_string(),
            technical_score,
            culture_score,
            recommendation: recommendation.to_string(),
            notes: notes.to_string(),
        };

        self.conn.execute(
            "INSERT INTO scorecards (id, candidate_id, interviewer, technical_score, culture_score, recommendation, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![sc.id, sc.candidate_id, sc.interviewer, sc.technical_score, sc.culture_score, sc.recommendation, sc.notes],
        )?;

        // Update candidate average
        let mut avg_stmt = self.conn.prepare("SELECT AVG((technical_score + culture_score) / 2.0) FROM scorecards WHERE candidate_id = ?1")?;
        let avg: f64 = avg_stmt.query_row(params![candidate_id], |row| row.get(0))?;

        self.conn.execute(
            "UPDATE candidates SET scorecard_avg = ?1 WHERE id = ?2",
            params![avg, candidate_id],
        )?;

        Ok(sc)
    }

    pub fn stats(&self) -> Result<HireStats, HireStoreError> {
        let mut req_stmt = self
            .conn
            .prepare("SELECT COUNT(*) FROM job_requisitions WHERE status = 'open'")?;
        let open_reqs: usize = req_stmt
            .query_row([], |row| row.get::<_, i64>(0))
            .unwrap_or(0) as usize;

        let mut cand_stmt = self.conn.prepare(
            "SELECT 
                COUNT(*),
                COALESCE(SUM(CASE WHEN stage IN ('screening', 'interview') THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN stage = 'offer' THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN stage = 'hired' THEN 1 ELSE 0 END), 0)
             FROM candidates",
        )?;

        let (total_candidates, interviewing_count, offer_count, hired_count) = cand_stmt
            .query_row([], |row| {
                Ok((
                    row.get::<_, i64>(0)? as usize,
                    row.get::<_, i64>(1)? as usize,
                    row.get::<_, i64>(2)? as usize,
                    row.get::<_, i64>(3)? as usize,
                ))
            })?;

        Ok(HireStats {
            open_reqs,
            total_candidates,
            interviewing_count,
            offer_count,
            hired_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recruiting_pipeline() {
        let store = HireStore::open_in_memory().unwrap();
        let req = store
            .create_req("REQ-01", "Rust Engineer", "Core", 1, 75000, 95000)
            .unwrap();
        assert_eq!(req.status, "open");

        let cand = store
            .create_candidate("CAND-01", "REQ-01", "Jane Doe", "jane@example.com", None)
            .unwrap();
        assert_eq!(cand.stage, CandidateStage::Applied);

        store
            .add_scorecard(
                "CAND-01",
                "Lead Dev",
                5,
                4,
                "strong_hire",
                "Excellent architecture knowledge",
            )
            .unwrap();
        let updated = store.get_candidate("CAND-01").unwrap();
        assert_eq!(updated.scorecard_avg, 4.5);

        store
            .update_candidate_stage("CAND-01", CandidateStage::Hired, Some(88000))
            .unwrap();
        let hired = store.get_candidate("CAND-01").unwrap();
        assert_eq!(hired.stage, CandidateStage::Hired);
        assert_eq!(hired.offered_salary_eur, Some(88000));
    }

    #[test]
    fn scorecard_refuses_scores_outside_one_to_five() -> Result<(), String> {
        let store = HireStore::open_in_memory().map_err(|e| e.to_string())?;
        store
            .create_req("REQ-01", "Rust Engineer", "Core", 1, 75000, 95000)
            .map_err(|e| e.to_string())?;
        store
            .create_candidate("CAND-01", "REQ-01", "Jane Doe", "jane@example.com", None)
            .map_err(|e| e.to_string())?;
        match store.add_scorecard("CAND-01", "Lead Dev", 6, 4, "strong_hire", "too high") {
            Err(HireStoreError::Validation(_)) => {}
            Ok(_) => return Err("score 6 was stored".into()),
            Err(other) => return Err(other.to_string()),
        }
        match store.add_scorecard("CAND-01", "Lead Dev", 0, 4, "no_hire", "too low") {
            Err(HireStoreError::Validation(_)) => {}
            Ok(_) => return Err("score 0 was stored".into()),
            Err(other) => return Err(other.to_string()),
        }
        let candidate = store.get_candidate("CAND-01").map_err(|e| e.to_string())?;
        if candidate.scorecard_avg != 0.0 {
            return Err(format!("average moved to {}", candidate.scorecard_avg));
        }
        Ok(())
    }
}
