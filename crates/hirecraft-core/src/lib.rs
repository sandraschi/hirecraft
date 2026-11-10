#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

pub use hirecraft_store::*;
use craft_biz::bus::Bus;
use craft_biz::BizError;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HireCoreError {
    #[error("Store error: {0}")]
    Store(#[from] HireStoreError),
    #[error("Bus error: {0}")]
    Bus(#[from] BizError),
    #[error("Validation error: {0}")]
    Validation(String),
}

pub struct HireService {
    store: HireStore,
    bus: Option<Bus>,
}

impl HireService {
    pub fn new(store: HireStore, bus: Option<Bus>) -> Self {
        Self { store, bus }
    }

    pub fn store(&self) -> &HireStore {
        &self.store
    }

    pub fn bus(&self) -> Option<&Bus> {
        self.bus.as_ref()
    }

    /// Advances candidate to Hired and triggers the 3-app onboarding cascade across the fleet
    pub fn hire_candidate(&self, id: &str, offered_salary_eur: i64) -> Result<Candidate, HireCoreError> {
        self.store.update_candidate_stage(id, CandidateStage::Hired, Some(offered_salary_eur))?;
        let cand = self.store.get_candidate(id)?;

        if let Some(bus) = &self.bus {
            let payload = json!({
                "candidate_id": cand.id,
                "name": cand.name,
                "email": cand.email,
                "req_id": cand.req_id,
                "salary_eur": offered_salary_eur,
                "azg_weekly_cap": 38.5,
                "annual_leave_days": 25,
            });

            // 1. Trigger employment contract in contractcraft
            let _ = bus.emit("hirecraft", "contractcraft", "candidate.hired", payload.clone());

            // 2. Trigger employee profile & timesheet in teamcraft
            let _ = bus.emit("hirecraft", "teamcraft", "candidate.hired", payload.clone());

            // 3. Trigger hardware reservation in assetcraft
            let _ = bus.emit("hirecraft", "assetcraft", "candidate.hired", payload);
        }

        Ok(cand)
    }
}
