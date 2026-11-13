#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use clap::Parser;
use craft_biz::bus::Bus;
use craft_biz_ui::bookend::{self, Bar, Spec};
use craft_biz_ui::menu::Item;
use craft_biz_ui::theme::{self, tokens};
use eframe::egui::{self, RichText};
use hirecraft_core::*;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "hirecraft-app",
    about = "Native egui desktop host for HireCraft"
)]
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

fn resolve_bus() -> Option<Bus> {
    if let Ok(dir) = std::env::var("CRAFT_BUS_DIR") {
        return Some(Bus::new(PathBuf::from(dir)));
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        return Some(Bus::new(PathBuf::from(local).join("craft-biz-bus")));
    }
    None
}

fn run_smoke_checks() -> Result<(), Box<dyn std::error::Error>> {
    let store = HireStore::open_in_memory()?;
    let req = store.create_req("REQ-SMOKE", "Smoke Engineer", "QA", 1, 60000, 80000)?;
    let cand = store.create_candidate(
        "CAND-SMOKE",
        &req.id,
        "Smoke Candidate",
        "smoke@test.at",
        None,
    )?;
    store.add_scorecard(&cand.id, "Interviewer", 5, 5, "strong_hire", "Perfect fit")?;
    let service = HireService::new(store, None);
    service.hire_candidate(&cand.id, 75000)?;
    let hired = service.store().get_candidate(&cand.id)?;
    if hired.stage != CandidateStage::Hired {
        return Err("Candidate stage update failed".into());
    }
    bookend::smoke("HireCraft", env!("CARGO_PKG_VERSION"), "hiring")?;
    println!("smoke: ok (hirecraft-app)");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.smoke {
        return run_smoke_checks();
    }
    let store = HireStore::open(resolve_db(args.db))?;
    let app = HireApp::new(HireService::new(store, resolve_bus()));
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 760.0])
            .with_title("HireCraft - Hiring desk"),
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
    service: HireService,
    reqs: Vec<JobRequisition>,
    candidates: Vec<Candidate>,
    stats: HireStats,
    selected: Option<String>,
    notice: String,
    bar: Bar,
    dark: bool,
    zoom: f32,
    req_id: String,
    req_title: String,
    req_dept: String,
    req_headcount: String,
    req_min: String,
    req_max: String,
    cand_id: String,
    cand_req: String,
    cand_name: String,
    cand_email: String,
    sc_who: String,
    sc_tech: String,
    sc_culture: String,
    sc_rec: String,
    sc_notes: String,
    salary: String,
}

impl HireApp {
    fn new(service: HireService) -> Self {
        let mut app = Self {
            service,
            reqs: Vec::new(),
            candidates: Vec::new(),
            stats: empty_stats(),
            selected: None,
            notice: String::new(),
            bar: Bar::new(),
            dark: true,
            zoom: 1.0,
            req_id: String::new(),
            req_title: String::new(),
            req_dept: String::new(),
            req_headcount: "1".into(),
            req_min: String::new(),
            req_max: String::new(),
            cand_id: String::new(),
            cand_req: String::new(),
            cand_name: String::new(),
            cand_email: String::new(),
            sc_who: String::new(),
            sc_tech: "4".into(),
            sc_culture: "4".into(),
            sc_rec: "hire".into(),
            sc_notes: String::new(),
            salary: "42000".into(),
        };
        app.refresh();
        app
    }

    fn paint_menu(&mut self, ui: &mut egui::Ui) {
        let docs = bookend::repo_docs(env!("CARGO_MANIFEST_DIR"));
        let spec = Spec {
            name: "HireCraft",
            version: env!("CARGO_PKG_VERSION"),
            description: "Requisitions, candidates, scorecards, and the hire cascade.",
            repository: "https://github.com/sandraschi/hirecraft",
            docs: &docs,
            dark: self.dark,
            zoom: self.zoom,
            file: vec![
                Item::action("Load Northwind demo", "app.demo"),
                Item::action("Refresh", "app.refresh"),
            ],
            view: Vec::new(),
            middle: Vec::new(),
        };
        let change = self.bar.show(ui, &spec);
        if let Some(dark) = change.dark {
            self.dark = dark;
        }
        if let Some(zoom) = change.zoom {
            self.zoom = zoom;
        }
        match change.action.as_deref() {
            Some("app.refresh") => {
                self.refresh();
                self.notice = "Refreshed.".into();
            }
            Some("app.demo") => self.load_demo(),
            _ => {}
        }
    }

    fn refresh(&mut self) {
        let store = self.service.store();
        self.reqs = store.list_reqs().unwrap_or_default();
        self.candidates = store.list_candidates(None).unwrap_or_default();
        self.stats = store.stats().unwrap_or_else(|_| empty_stats());
    }

    fn load_demo(&mut self) {
        match self.service.store().seed_northwind_demo() {
            Ok(n) => {
                self.notice =
                    format!("Loaded {n} labelled Northwind Traders candidates. Nobody is hired.");
                self.refresh();
                if self.selected.is_none() {
                    self.selected = self.candidates.first().map(|c| c.id.clone());
                }
            }
            Err(e) => self.notice = e.to_string(),
        }
    }

    fn create_req(&mut self) {
        let id = self.req_id.trim().to_string();
        let title = self.req_title.trim().to_string();
        let dept = self.req_dept.trim().to_string();
        if id.is_empty() || title.is_empty() || dept.is_empty() {
            self.notice = "Requisition id, title, and department are required.".into();
            return;
        }
        let head = match parse_positive(&self.req_headcount, "headcount") {
            Ok(n) => n,
            Err(e) => {
                self.notice = e;
                return;
            }
        };
        let min = match parse_euros(&self.req_min, "salary min") {
            Ok(n) => n,
            Err(e) => {
                self.notice = e;
                return;
            }
        };
        let max = match parse_euros(&self.req_max, "salary max") {
            Ok(n) => n,
            Err(e) => {
                self.notice = e;
                return;
            }
        };
        if min > max {
            self.notice = "Salary min cannot be above salary max.".into();
            return;
        }
        match self
            .service
            .store()
            .create_req(&id, &title, &dept, head, min, max)
        {
            Ok(_) => {
                self.notice = format!("Requisition {id} opened.");
                self.cand_req = id;
                self.req_id.clear();
                self.req_title.clear();
                self.req_dept.clear();
                self.refresh();
            }
            Err(e) => self.notice = e.to_string(),
        }
    }

    fn create_candidate(&mut self) {
        let id = self.cand_id.trim().to_string();
        let req = self.cand_req.trim().to_string();
        let name = self.cand_name.trim().to_string();
        let email = self.cand_email.trim().to_string();
        if id.is_empty() || req.is_empty() || name.is_empty() || email.is_empty() {
            self.notice = "Candidate id, requisition, name, and email are required.".into();
            return;
        }
        match self
            .service
            .store()
            .create_candidate(&id, &req, &name, &email, None)
        {
            Ok(_) => {
                self.notice = format!("Candidate {id} added at applied.");
                self.cand_id.clear();
                self.cand_name.clear();
                self.cand_email.clear();
                self.selected = Some(id);
                self.refresh();
            }
            Err(e) => self.notice = e.to_string(),
        }
    }

    fn add_scorecard(&mut self) {
        let Some(id) = self.selected.clone() else {
            self.notice = "Select a candidate first.".into();
            return;
        };
        let who = self.sc_who.trim().to_string();
        if who.is_empty() {
            self.notice = "Scorecard needs an interviewer name.".into();
            return;
        }
        let tech = match parse_score(&self.sc_tech) {
            Ok(n) => n,
            Err(e) => {
                self.notice = e;
                return;
            }
        };
        let culture = match parse_score(&self.sc_culture) {
            Ok(n) => n,
            Err(e) => {
                self.notice = e;
                return;
            }
        };
        let rec = self.sc_rec.trim().to_string();
        let notes = self.sc_notes.trim().to_string();
        match self
            .service
            .store()
            .add_scorecard(&id, &who, tech, culture, &rec, &notes)
        {
            Ok(_) => {
                self.notice = format!("Scorecard saved for {id}.");
                self.sc_notes.clear();
                self.refresh();
            }
            Err(e) => self.notice = e.to_string(),
        }
    }

    fn set_stage(&mut self, stage: CandidateStage, salary: Option<i64>) {
        let Some(id) = self.selected.clone() else {
            self.notice = "Select a candidate first.".into();
            return;
        };
        match self
            .service
            .store()
            .update_candidate_stage(&id, stage, salary)
        {
            Ok(()) => {
                self.notice = format!("{id} moved to {}.", stage.as_str());
                self.refresh();
            }
            Err(e) => self.notice = e.to_string(),
        }
    }

    fn paint_detail(&mut self, ui: &mut egui::Ui) {
        let Some(id) = self.selected.clone() else {
            ui.heading("Candidate");
            ui.label("Add or open a candidate. Hire writes the cascade to contract, team, and asset inboxes.");
            return;
        };
        let Ok(c) = self.service.store().get_candidate(&id) else {
            ui.label("That candidate is no longer in the file.");
            return;
        };
        ui.heading(&c.name);
        ui.label(&c.email);
        ui.label(format!("{}  req {}", c.stage.as_str(), c.req_id));
        ui.label(format!("Scorecard average {:.1} / 5", c.scorecard_avg));
        if let Some(note) = &c.notes {
            ui.label(note);
        }
        if let Some(sal) = c.offered_salary_eur {
            ui.label(format!("Offered {sal} EUR"));
        }
        ui.separator();
        ui.label(RichText::new("Stage").strong());
        ui.horizontal(|ui| {
            if ui.button("Screening").clicked() {
                self.set_stage(CandidateStage::Screening, None);
            }
            if ui.button("Interview").clicked() {
                self.set_stage(CandidateStage::Interview, None);
            }
            if ui.button("Offer").clicked() {
                let salary = parse_euros(&self.salary, "salary").ok();
                self.set_stage(CandidateStage::Offer, salary);
            }
            if ui.button("Reject").clicked() {
                self.set_stage(CandidateStage::Rejected, None);
            }
        });
        ui.separator();
        ui.label(RichText::new("Scorecard (1 to 5)").strong());
        ui.horizontal(|ui| {
            field(ui, "Interviewer", &mut self.sc_who, 110.0);
            field(ui, "Tech", &mut self.sc_tech, 28.0);
            field(ui, "Culture", &mut self.sc_culture, 28.0);
        });
        ui.horizontal(|ui| {
            field(ui, "Recommendation", &mut self.sc_rec, 90.0);
            field(ui, "Notes", &mut self.sc_notes, 140.0);
            if ui.button("Save scorecard").clicked() {
                self.add_scorecard();
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            field(ui, "Salary EUR", &mut self.salary, 80.0);
            if ui.button("Hire").clicked() {
                self.hire();
            }
        });
        ui.label("Hire notifies contractcraft, teamcraft, and assetcraft. Stage buttons do not.");
    }

    fn hire(&mut self) {
        let Some(id) = self.selected.clone() else {
            self.notice = "Select a candidate first.".into();
            return;
        };
        let salary = match parse_euros(&self.salary, "salary") {
            Ok(n) => n,
            Err(e) => {
                self.notice = e;
                return;
            }
        };
        match self.service.hire_candidate(&id, salary) {
            Ok(_) => {
                self.notice = format!("{id} hired at {salary} EUR. Contract, team, and asset inboxes were notified when the bus is available.");
                self.refresh();
            }
            Err(e) => self.notice = e.to_string(),
        }
    }
}

impl eframe::App for HireApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        theme::apply(ui.ctx(), self.dark);
        self.paint_menu(ui);
        let accent = tokens(self.dark).accent;
        ui.horizontal(|ui| {
            ui.heading(RichText::new("HireCraft").strong().color(accent));
            ui.separator();
            ui.label("Open a requisition, add a candidate, score them, then hire. Demo people are labelled DEMO.");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = if self.dark { "Light theme" } else { "Dark theme" };
                if ui.button(label).clicked() {
                    self.dark = !self.dark;
                    theme::apply(ui.ctx(), self.dark);
                }
            });
        });
        if !self.notice.is_empty() {
            ui.label(RichText::new(&self.notice).color(accent));
        }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            kpi(ui, "Open reqs", self.stats.open_reqs);
            ui.separator();
            kpi(ui, "Pipeline", self.stats.total_candidates);
            ui.separator();
            kpi(ui, "Interviewing", self.stats.interviewing_count);
            ui.separator();
            kpi(ui, "Offers", self.stats.offer_count);
            ui.separator();
            kpi(ui, "Hired", self.stats.hired_count);
        });
        ui.separator();
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_min_width(560.0);
                ui.heading("New requisition");
                ui.horizontal(|ui| {
                    field(ui, "Id", &mut self.req_id, 80.0);
                    field(ui, "Title", &mut self.req_title, 140.0);
                    field(ui, "Department", &mut self.req_dept, 100.0);
                });
                ui.horizontal(|ui| {
                    field(ui, "Headcount", &mut self.req_headcount, 40.0);
                    field(ui, "Min EUR", &mut self.req_min, 70.0);
                    field(ui, "Max EUR", &mut self.req_max, 70.0);
                    if ui.button("Open requisition").clicked() {
                        self.create_req();
                    }
                });
                ui.add_space(4.0);
                ui.heading("New candidate");
                ui.horizontal(|ui| {
                    field(ui, "Id", &mut self.cand_id, 80.0);
                    field(ui, "Req", &mut self.cand_req, 80.0);
                    field(ui, "Name", &mut self.cand_name, 140.0);
                    field(ui, "Email", &mut self.cand_email, 150.0);
                });
                ui.horizontal(|ui| {
                    if ui.button("Add candidate").clicked() {
                        self.create_candidate();
                    }
                    if ui.button("Load Northwind demo").clicked() {
                        self.load_demo();
                    }
                });
                ui.add_space(6.0);
                ui.heading("Candidates");
                if self.candidates.is_empty() {
                    ui.label("No candidates yet. Add one, or load the Northwind Traders desk.");
                }
                egui::ScrollArea::vertical()
                    .max_height(240.0)
                    .show(ui, |ui| {
                        egui::Grid::new("cand_grid")
                            .striped(true)
                            .min_col_width(70.0)
                            .show(ui, |ui| {
                                ui.label(RichText::new("Id").strong());
                                ui.label(RichText::new("Name").strong());
                                ui.label(RichText::new("Req").strong());
                                ui.label(RichText::new("Stage").strong());
                                ui.label(RichText::new("Score").strong());
                                ui.label("");
                                ui.end_row();
                                let mut pick = None;
                                for c in &self.candidates {
                                    ui.label(&c.id);
                                    ui.label(&c.name);
                                    ui.label(&c.req_id);
                                    ui.label(c.stage.as_str());
                                    ui.label(format!("{:.1}", c.scorecard_avg));
                                    if ui.button("Open").clicked() {
                                        pick = Some(c.id.clone());
                                    }
                                    ui.end_row();
                                }
                                if let Some(id) = pick {
                                    self.selected = Some(id);
                                }
                            });
                    });
                ui.heading("Open requisitions");
                for r in &self.reqs {
                    ui.label(format!(
                        "{}  {}  {}  {} seats  {}-{} EUR",
                        r.id,
                        r.title,
                        r.department,
                        r.target_headcount,
                        r.salary_min_eur,
                        r.salary_max_eur
                    ));
                }
            });
            ui.separator();
            ui.vertical(|ui| {
                ui.set_min_width(420.0);
                self.paint_detail(ui);
            });
        });
    }
}

fn empty_stats() -> HireStats {
    HireStats {
        open_reqs: 0,
        total_candidates: 0,
        interviewing_count: 0,
        offer_count: 0,
        hired_count: 0,
    }
}

fn kpi(ui: &mut egui::Ui, name: &str, n: usize) {
    ui.label(RichText::new(format!("{name}: {n}")).strong());
}

fn field(ui: &mut egui::Ui, name: &str, value: &mut String, width: f32) {
    ui.label(name);
    ui.add(egui::TextEdit::singleline(value).desired_width(width));
}

fn parse_positive(raw: &str, label: &str) -> Result<usize, String> {
    let n: usize = raw
        .trim()
        .parse()
        .map_err(|_| format!("{label} must be a whole number"))?;
    if n == 0 {
        return Err(format!("{label} must be at least 1"));
    }
    Ok(n)
}

fn parse_euros(raw: &str, label: &str) -> Result<i64, String> {
    let n: i64 = raw
        .trim()
        .parse()
        .map_err(|_| format!("{label} must be a whole number of euros"))?;
    if n <= 0 {
        return Err(format!("{label} must be greater than zero"));
    }
    Ok(n)
}

fn parse_score(raw: &str) -> Result<i32, String> {
    let n: i32 = raw
        .trim()
        .parse()
        .map_err(|_| "score must be a whole number".to_string())?;
    if !(1..=5).contains(&n) {
        return Err("score must be from 1 to 5".into());
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn salary_and_score_parse() -> Result<(), String> {
        if parse_euros("42000", "salary")? != 42000 {
            return Err("euros".into());
        }
        if parse_score("4")? != 4 || parse_score("6").is_ok() {
            return Err("score".into());
        }
        if parse_positive("2", "headcount")? != 2 || parse_positive("0", "headcount").is_ok() {
            return Err("headcount".into());
        }
        Ok(())
    }
}
