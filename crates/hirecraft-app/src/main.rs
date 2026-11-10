#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use clap::Parser;
use eframe::egui::{self, Color32, RichText};
use hirecraft_core::*;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "hirecraft-app", about = "Native egui desktop host for HireCraft")]
struct Args {
    #[arg(long)]
    smoke: bool,
    #[arg(long)]
    db: Option<PathBuf>,
}

fn resolve_db(path: Option<PathBuf>) -> PathBuf {
    if let Some(p) = path {
        return p;
    }
    if let Ok(p) = std::env::var("HIRECRAFT_DB") {
        return PathBuf::from(p);
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let dir = PathBuf::from(local).join("hirecraft");
        let _ = std::fs::create_dir_all(&dir);
        return dir.join("hirecraft.db");
    }
    PathBuf::from("hirecraft.db")
}

fn run_smoke_checks() -> Result<(), Box<dyn std::error::Error>> {
    let store = HireStore::open_in_memory()?;
    let req = store.create_req("REQ-SMOKE", "Smoke Engineer", "QA", 1, 60000, 80000)?;
    let cand = store.create_candidate("CAND-SMOKE", &req.id, "Smoke Candidate", "smoke@test.at", None)?;
    store.add_scorecard(&cand.id, "Interviewer", 5, 5, "strong_hire", "Perfect fit")?;
    store.update_candidate_stage(&cand.id, CandidateStage::Hired, Some(75000))?;
    let hired = store.get_candidate(&cand.id)?;
    if hired.stage != CandidateStage::Hired {
        return Err("Candidate stage update failed".into());
    }
    println!("smoke: ok (hirecraft-app)");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.smoke {
        return run_smoke_checks();
    }

    let db_path = resolve_db(args.db);
    let store = HireStore::open(&db_path)?;
    let app = HireApp::new(store);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1040.0, 680.0])
            .with_title("HireCraft — Sovereign ATS & Recruiting Pipeline"),
        ..Default::default()
    };

    eframe::run_native(
        "HireCraft",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .map_err(|e| format!("Eframe runtime error: {e}").into())
}

struct HireApp {
    store: HireStore,
    candidates: Vec<Candidate>,
    stats: HireStats,
    selected_candidate: Option<String>,
    hire_salary_input: i64,
}

impl HireApp {
    fn new(store: HireStore) -> Self {
        let candidates = store.list_candidates(None).unwrap_or_default();
        let stats = store.stats().unwrap_or(HireStats {
            open_reqs: 0,
            total_candidates: 0,
            interviewing_count: 0,
            offer_count: 0,
            hired_count: 0,
        });
        Self {
            store,
            candidates,
            stats,
            selected_candidate: None,
            hire_salary_input: 75000,
        }
    }

    fn refresh(&mut self) {
        if let Ok(cs) = self.store.list_candidates(None) {
            self.candidates = cs;
        }
        if let Ok(st) = self.store.stats() {
            self.stats = st;
        }
    }
}

impl eframe::App for HireApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("👥 HireCraft").strong().color(Color32::from_rgb(155, 89, 182)));
            ui.separator();
            ui.label(RichText::new("Sovereign ATS, Candidate Pipeline & Onboarding Handshake").weak());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄 Refresh").clicked() {
                    self.refresh();
                }
            });
        });
        ui.add_space(4.0);

        // Live KPI HUD
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Open Reqs: {}", self.stats.open_reqs)).strong());
            ui.separator();
            ui.label(RichText::new(format!("Total Pipeline: {}", self.stats.total_candidates)).color(Color32::from_rgb(52, 152, 219)));
            ui.separator();
            ui.label(RichText::new(format!("Interviewing: {}", self.stats.interviewing_count)).color(Color32::from_rgb(241, 196, 15)));
            ui.separator();
            ui.label(RichText::new(format!("Offers Out: {}", self.stats.offer_count)).color(Color32::from_rgb(230, 126, 34)));
            ui.separator();
            ui.label(RichText::new(format!("Hired: {}", self.stats.hired_count)).color(Color32::from_rgb(46, 204, 113)));
        });
        ui.separator();

        ui.horizontal(|ui| {
            // Left main candidate pipeline table
            ui.vertical(|ui| {
                ui.heading("Candidates Pipeline");
                egui::ScrollArea::vertical().max_width(620.0).show(ui, |ui| {
                    egui::Grid::new("cand_grid").striped(true).min_col_width(90.0).show(ui, |ui| {
                        ui.label(RichText::new("ID").strong());
                        ui.label(RichText::new("Name").strong());
                        ui.label(RichText::new("Req").strong());
                        ui.label(RichText::new("Stage").strong());
                        ui.label(RichText::new("Score").strong());
                        ui.label(RichText::new("Action").strong());
                        ui.end_row();

                        let mut select_cand = None;
                        for c in &self.candidates {
                            ui.label(&c.id);
                            ui.label(&c.name);
                            ui.label(&c.req_id);
                            let stage_color = match c.stage {
                                CandidateStage::Hired => Color32::from_rgb(46, 204, 113),
                                CandidateStage::Offer => Color32::from_rgb(230, 126, 34),
                                CandidateStage::Interview => Color32::from_rgb(241, 196, 15),
                                CandidateStage::Screening => Color32::from_rgb(52, 152, 219),
                                CandidateStage::Rejected => Color32::from_rgb(231, 76, 60),
                                CandidateStage::Applied => Color32::GRAY,
                            };
                            ui.label(RichText::new(c.stage.as_str()).color(stage_color));
                            ui.label(format!("{:.1}/5", c.scorecard_avg));
                            if ui.button("Review").clicked() {
                                select_cand = Some(c.id.clone());
                            }
                            ui.end_row();
                        }

                        if let Some(id) = select_cand {
                            self.selected_candidate = Some(id);
                            self.refresh();
                        }
                    });
                });
            });

            ui.separator();

            // Right inspector & pipeline advancement
            ui.vertical(|ui| {
                if let Some(id) = &self.selected_candidate.clone() {
                    ui.heading(format!("Candidate: {id}"));
                    ui.add_space(4.0);

                    if let Ok(c) = self.store.get_candidate(id) {
                        ui.label(format!("Name: {}", c.name));
                        ui.label(format!("Email: {}", c.email));
                        ui.label(format!("Req: {}", c.req_id));
                        ui.label(format!("Stage: {:?}", c.stage));
                        ui.label(RichText::new(format!("Scorecard Rating: {:.1} / 5.0", c.scorecard_avg)).strong());
                        if let Some(sal) = c.offered_salary_eur {
                            ui.label(format!("Offered Salary: €{sal}/yr"));
                        }
                        ui.separator();

                        ui.label(RichText::new("Advance Pipeline Stage:").strong());
                        ui.horizontal(|ui| {
                            if ui.button("Screening").clicked() {
                                let _ = self.store.update_candidate_stage(id, CandidateStage::Screening, None);
                                self.refresh();
                            }
                            if ui.button("Interview").clicked() {
                                let _ = self.store.update_candidate_stage(id, CandidateStage::Interview, None);
                                self.refresh();
                            }
                            if ui.button("Offer").clicked() {
                                let _ = self.store.update_candidate_stage(id, CandidateStage::Offer, Some(self.hire_salary_input));
                                self.refresh();
                            }
                        });

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label("Salary (€):");
                            ui.add(egui::DragValue::new(&mut self.hire_salary_input).range(30000..=300000).speed(1000.0));
                            if ui.button("🤝 HIRE CANDIDATE").clicked() {
                                let _ = self.store.update_candidate_stage(id, CandidateStage::Hired, Some(self.hire_salary_input));
                                self.refresh();
                            }
                        });

                        ui.add_space(6.0);
                        if ui.button("Reject Candidate").clicked() {
                            let _ = self.store.update_candidate_stage(id, CandidateStage::Rejected, None);
                            self.refresh();
                        }
                    }
                } else {
                    ui.label(RichText::new("Select a candidate on the left to evaluate scorecards and advance pipeline stages.").weak());
                }
            });
        });
    }
}
