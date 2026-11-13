#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use clap::{Parser, Subcommand};
use craft_biz::bus::Bus;
use hirecraft_core::*;
use serde_json::json;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "hirecraft",
    author,
    version,
    about = "Local-first ATS & recruiting pipeline engine. CLI + MCP."
)]
struct Cli {
    #[arg(long)]
    db: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Status,
    ReqCreate {
        id: String,
        title: String,
        department: String,
        #[arg(long, default_value = "1")]
        headcount: usize,
        #[arg(long, default_value = "60000")]
        min_salary: i64,
        #[arg(long, default_value = "85000")]
        max_salary: i64,
    },
    ReqList,
    CandCreate {
        id: String,
        req_id: String,
        name: String,
        email: String,
        #[arg(long)]
        notes: Option<String>,
    },
    CandShow {
        id: String,
    },
    CandList {
        #[arg(long)]
        stage: Option<String>,
    },
    CandStage {
        id: String,
        stage: String,
        #[arg(long)]
        salary: Option<i64>,
    },
    CandHire {
        id: String,
        salary: i64,
    },
    ScorecardAdd {
        candidate_id: String,
        interviewer: String,
        technical: i32,
        culture: i32,
        recommendation: String,
        notes: String,
    },
    Stats,
    Tools,
    Mcp,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let db_path = resolve_db(cli.db);
    let store = HireStore::open(&db_path)?;
    let bus = resolve_bus();
    let service = HireService::new(store, bus);

    let Some(cmd) = cli.command else {
        println!(
            "HireCraft — Local-first ATS & recruiting pipeline engine. Use --help for subcommands."
        );
        return Ok(());
    };

    match cmd {
        Commands::Status => {
            let s = service.store().stats()?;
            println!("Open Requisitions: {}", s.open_reqs);
            println!("Total Candidates: {}", s.total_candidates);
            println!("In Interview: {}", s.interviewing_count);
            println!("Offers Out: {}", s.offer_count);
            println!("Hired: {}", s.hired_count);
        }
        Commands::ReqCreate {
            id,
            title,
            department,
            headcount,
            min_salary,
            max_salary,
        } => {
            let req = service.store().create_req(
                &id,
                &title,
                &department,
                headcount,
                min_salary,
                max_salary,
            )?;
            println!(
                "Created job requisition {} ({}) in {}",
                req.id, req.title, req.department
            );
        }
        Commands::ReqList => {
            let reqs = service.store().list_reqs()?;
            for r in reqs {
                println!(
                    "{:12} {:30} {:14} €{}-€{}",
                    r.id, r.title, r.department, r.salary_min_eur, r.salary_max_eur
                );
            }
        }
        Commands::CandCreate {
            id,
            req_id,
            name,
            email,
            notes,
        } => {
            let cand =
                service
                    .store()
                    .create_candidate(&id, &req_id, &name, &email, notes.as_deref())?;
            println!(
                "Created candidate {} ({}) for req {}",
                cand.id, cand.name, cand.req_id
            );
        }
        Commands::CandShow { id } => {
            let c = service.store().get_candidate(&id)?;
            println!("Candidate: {} ({})", c.name, c.id);
            println!("Email: {}", c.email);
            println!("Req: {}", c.req_id);
            println!("Stage: {:?}", c.stage);
            println!("Scorecard Avg: {:.1}/5.0", c.scorecard_avg);
            if let Some(sal) = c.offered_salary_eur {
                println!("Offered Salary: €{}", sal);
            }
        }
        Commands::CandList { stage } => {
            let filter = stage.as_deref().and_then(CandidateStage::from_str_opt);
            let candidates = service.store().list_candidates(filter)?;
            for c in candidates {
                println!(
                    "{:10} {:20} {:12} Score: {:.1} Stage: {}",
                    c.id,
                    c.name,
                    c.req_id,
                    c.scorecard_avg,
                    c.stage.as_str()
                );
            }
        }
        Commands::CandStage { id, stage, salary } => {
            let st = CandidateStage::from_str_opt(&stage).ok_or("Invalid stage")?;
            service.store().update_candidate_stage(&id, st, salary)?;
            println!("Updated candidate {} to stage {}", id, st.as_str());
        }
        Commands::CandHire { id, salary } => {
            let cand = service.hire_candidate(&id, salary)?;
            println!(
                "Hired candidate {} ({}) at €{}. Automated onboarding cascade emitted!",
                cand.id, cand.name, salary
            );
        }
        Commands::ScorecardAdd {
            candidate_id,
            interviewer,
            technical,
            culture,
            recommendation,
            notes,
        } => {
            let sc = service.store().add_scorecard(
                &candidate_id,
                &interviewer,
                technical,
                culture,
                &recommendation,
                &notes,
            )?;
            println!(
                "Added scorecard {} for candidate {}. Tech: {}/5, Culture: {}/5",
                sc.id, candidate_id, sc.technical_score, sc.culture_score
            );
        }
        Commands::Stats => {
            let s = service.store().stats()?;
            println!("{}", serde_json::to_string_pretty(&s)?);
        }
        Commands::Tools => {
            print_tools_catalog();
        }
        Commands::Mcp => {
            run_mcp_server(service)?;
        }
    }

    Ok(())
}

fn print_tools_catalog() {
    let catalog = json!([
        { "name": "hire_stats", "description": "Get recruiting funnel totals" },
        { "name": "req_create", "description": "Create a new job requisition" },
        { "name": "req_list", "description": "List open job requisitions" },
        { "name": "candidate_create", "description": "Add candidate to recruitment pipeline" },
        { "name": "candidate_get", "description": "Get candidate details and scorecard average" },
        { "name": "candidate_list", "description": "List candidates filtered by stage" },
        { "name": "candidate_advance", "description": "Advance candidate along recruiting pipeline" },
        { "name": "candidate_hire", "description": "Hire candidate and trigger cross-fleet onboarding cascade" },
        { "name": "scorecard_add", "description": "Add interviewer evaluation scorecard" }
    ]);
    println!(
        "{}",
        serde_json::to_string_pretty(&catalog).unwrap_or_default()
    );
}

fn run_mcp_server(service: HireService) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let Ok(req) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };

        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = req.get("id").cloned();

        match method {
            "initialize" => {
                let resp = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "serverInfo": { "name": "hirecraft-mcp", "version": "0.1.0" },
                        "capabilities": { "tools": {} }
                    }
                });
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                stdout.flush()?;
            }
            "tools/list" => {
                let resp = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": [
                            { "name": "hire_stats", "description": "Get recruiting totals", "inputSchema": { "type": "object" } },
                            { "name": "candidate_list", "description": "List candidates", "inputSchema": { "type": "object" } }
                        ]
                    }
                });
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                stdout.flush()?;
            }
            "tools/call" => {
                let tool_name = req
                    .get("params")
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("");
                let result_json = match tool_name {
                    "hire_stats" => {
                        let st = service.store().stats().map_err(|e| e.to_string())?;
                        json!(st)
                    }
                    "candidate_list" => {
                        let cands = service
                            .store()
                            .list_candidates(None)
                            .map_err(|e| e.to_string())?;
                        json!(cands)
                    }
                    _ => json!({ "status": "unknown tool" }),
                };
                let resp = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&result_json)? }]
                    }
                });
                writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                stdout.flush()?;
            }
            _ => {
                if let Some(id) = id {
                    let resp = json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": "Method not found" } });
                    writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
                    stdout.flush()?;
                }
            }
        }
    }
    Ok(())
}
