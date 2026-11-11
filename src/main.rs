#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod i18n;

use eframe::egui;
use i18n::{Lang, Strings};
use serde::Deserialize;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

#[derive(Deserialize, Debug, Clone)]
struct Acc {
    name: String,
    domain: String,
    contact: String,
    reliability: String,
    justdeleteme: String,
    draft: String,
    #[serde(default)] method: String,
    #[serde(default)] action_url: String,
    #[serde(default)] instructions: String,
    #[serde(default)] requires_login: bool,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Msg {
    Log { msg: String },
    Account(Acc),
    Done { dir: String, count: usize },
    Error { msg: String },
}

enum Event {
    Engine(Msg),
    Exit(Option<i32>),
}

#[cfg(windows)]
fn hide_console(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x0800_0000);
}
#[cfg(not(windows))]
fn hide_console(_cmd: &mut Command) {}

fn open_path(p: &Path) {
    #[cfg(target_os = "windows")] let prog = "explorer";
    #[cfg(target_os = "macos")] let prog = "open";
    #[cfg(all(unix, not(target_os = "macos")))] let prog = "xdg-open";
    let _ = Command::new(prog).arg(p).spawn();
}

fn open_url(url: &str) {
    #[cfg(target_os = "windows")] let prog = "explorer";
    #[cfg(target_os = "macos")] let prog = "open";
    #[cfg(all(unix, not(target_os = "macos")))] let prog = "xdg-open";
    let _ = Command::new(prog).arg(url).spawn();
}

fn download_draft(source: &Path, domain: &str, s: &Strings) {
    let default = format!("{}{}.txt", s.file_prefix, domain);
    if let Some(path) = rfd::FileDialog::new()
        .set_title(s.save_dialog_title)
        .set_file_name(&default)
        .save_file()
    {
        let _ = fs::copy(source, path);
    }
}

fn find_engine() -> PathBuf {
    #[cfg(target_os = "windows")] let engine_name = "engine.exe";
    #[cfg(not(target_os = "windows"))] let engine_name = "engine.py";

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join(engine_name);
            if p.exists() { return p; }
            #[cfg(target_os = "windows")]
            {
                let p_py = dir.join("engine.py");
                if p_py.exists() { return p_py; }
            }
        }
    }
    PathBuf::from(engine_name)
}

fn valid_email(s: &str) -> bool {
    let s = s.trim();
    match s.split_once('@') {
        Some((l, d)) => !l.is_empty() && d.contains('.') && !d.ends_with('.'),
        None => false,
    }
}

fn send(tx: &Sender<Event>, ctx: &egui::Context, ev: Event) {
    let _ = tx.send(ev);
    ctx.request_repaint();
}

fn run_engine(
    python: String, engine: PathBuf, email: String, name: String, out: PathBuf,
    lang_code: String, child_slot: Arc<Mutex<Option<Child>>>, tx: Sender<Event>,
    ctx: egui::Context, s: &'static Strings,
) {
    let is_exe = engine.extension().map_or(false, |e| e.eq_ignore_ascii_case("exe"));

    let mut cmd;
    if is_exe {
        cmd = Command::new(&engine);
        cmd.args(["--email", &email, "--name", &name, "--out"]).arg(&out)
            .args(["--lang", &lang_code]);
    } else {
        cmd = Command::new(&python);
        cmd.args(["-u", "-X", "utf8"]).arg(&engine)
            .args(["--email", &email, "--name", &name, "--out"]).arg(&out)
            .args(["--lang", &lang_code])
            .env("PYTHONIOENCODING", "utf-8");
    }

    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    hide_console(&mut cmd);

    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let target = engine.file_name().unwrap_or_default().to_string_lossy();
            let msg = s.launching_error.replacen("{}", &target, 1).replacen("{}", &e.to_string(), 1);
            send(&tx, &ctx, Event::Engine(Msg::Error { msg }));
            send(&tx, &ctx, Event::Exit(None));
            return;
        }
    };

    *child_slot.lock().unwrap() = Some(child);
    let mut child = child_slot.lock().unwrap().take().unwrap();
    let stderr = child.stderr.take();
    
    let stderr_thread = stderr.map(|se| {
        let tx2 = tx.clone();
        let ctx2 = ctx.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(se).lines().map_while(Result::ok) {
                if !line.trim().is_empty() {
                    send(&tx2, &ctx2, Event::Engine(Msg::Log { msg: format!("[stderr] {line}") }));
                }
            }
        })
    });

    if let Some(stdout) = child.stdout.take() {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let line = line.trim().to_string();
            if line.is_empty() { continue; }
            let msg = serde_json::from_str::<Msg>(&line).unwrap_or(Msg::Log { msg: line });
            send(&tx, &ctx, Event::Engine(msg));
        }
    }
    
    if let Some(h) = stderr_thread { let _ = h.join(); }
    let code = child.wait().ok().and_then(|s| s.code());
    *child_slot.lock().unwrap() = None;
    send(&tx, &ctx, Event::Exit(code));
}

struct App {
    email: String, name: String, python: String, certified: bool, running: bool,
    logs: Vec<String>, accounts: Vec<Acc>, out_dir: Option<PathBuf>,
    got_done: bool, rx: Option<Receiver<Event>>, method_filter: String,
    lang: Lang, child_slot: Arc<Mutex<Option<Child>>>, selected_account: Option<Acc>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            email: String::new(), name: String::new(), python: "python".to_string(),
            certified: false, running: false, logs: Vec::new(), accounts: Vec::new(),
            out_dir: None, got_done: false, rx: None, method_filter: String::new(),
            lang: Lang::detect(), child_slot: Arc::new(Mutex::new(None)), selected_account: None,
        }
    }
}

impl App {
    fn can_start(&self) -> bool {
        self.certified && !self.running && valid_email(&self.email)
            && !self.name.trim().is_empty() && !self.python.trim().is_empty()
    }

    fn start(&mut self, ctx: &egui::Context, s: &'static Strings) {
        let local: String = self.email.trim().split('@').next().unwrap_or("rgpd")
            .chars().map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' }).collect();
        let out = std::env::current_dir().unwrap_or_default().join(local);

        self.logs.clear(); self.accounts.clear(); self.out_dir = None;
        self.got_done = false; self.running = true; self.method_filter.clear();
        self.selected_account = None;

        let (tx, rx) = channel();
        self.rx = Some(rx);
        
        let lang_code = match self.lang { Lang::Fr => "fr", Lang::En => "en" }.to_string();
        let ctx = ctx.clone(); 
        let slot = self.child_slot.clone();

        let python = self.python.trim().to_string();
        let email = self.email.trim().to_string();
        let name = self.name.trim().to_string();
        let engine = find_engine();
        
        std::thread::spawn(move || run_engine(
            python, engine, email, name, out, lang_code, slot, tx, ctx, s
        ));
    }

    fn stop(&mut self, s: &'static Strings) {
        if let Some(child) = self.child_slot.lock().unwrap().as_mut() {
            let _ = child.kill();
            self.logs.push(s.stopped_message.to_string());
        }
    }

    fn poll(&mut self, s: &'static Strings) {
        let Some(rx) = self.rx.take() else { return };
        let mut keep = true; let mut budget = 200;
        while let Ok(ev) = rx.try_recv() {
            match ev {
                Event::Engine(Msg::Log { msg }) => self.logs.push(msg),
                Event::Engine(Msg::Account(a)) => self.accounts.push(a),
                Event::Engine(Msg::Done { dir, count }) => {
                    self.out_dir = Some(PathBuf::from(dir));
                    self.got_done = true;
                    self.logs.push(s.done_message.replace("{}", &count.to_string()));
                }
                Event::Engine(Msg::Error { msg }) => self.logs.push(format!("ERROR: {msg}")),
                Event::Exit(code) => {
                    if !self.got_done {
                        if code == Some(0) {
                            self.logs.push(s.stopped_message.to_string());
                        } else {
                            self.logs.push(s.engine_stopped_no_result.replace("{:?}", &format!("{:?}", code)));
                        }
                    }
                    self.running = false; keep = false;
                }
            }
            budget -= 1; if budget == 0 { break; }
        }
        if keep && self.running { self.rx = Some(rx); }
    }

    fn method_color(m: &str) -> egui::Color32 {
        match m {
            "direct" => egui::Color32::from_rgb(60, 180, 90),
            "form" => egui::Color32::from_rgb(220, 180, 60),
            "email" => egui::Color32::from_rgb(80, 140, 220),
            "ticket" => egui::Color32::from_rgb(220, 120, 60),
            "impossible" => egui::Color32::from_rgb(200, 60, 60),
            _ => egui::Color32::GRAY,
        }
    }

    fn method_label(m: &str) -> &'static str {
        match m {
            "direct" => "🟢 Direct", "form" => "🟡 Form", "email" => "🔵 Email",
            "ticket" => "🟠 Ticket", "impossible" => "🔴 Impossible", _ => "⚪ ?",
        }
    }

    fn count_by(&self, m: &str) -> usize {
        if m.is_empty() { self.accounts.len() }
        else { self.accounts.iter().filter(|a| a.method == m).count() }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let s: &'static Strings = Strings::for_lang(self.lang);
        self.poll(s);

        if let Some(acc) = self.selected_account.clone() {
            let mut is_open = true;
            egui::Window::new(format!("Détails : {}", acc.domain))
                .open(&mut is_open).collapsible(false).resizable(true).default_width(400.0)
                .show(ctx, |ui| {
                    ui.heading(&acc.domain);
                    ui.horizontal(|ui| {
                        ui.label(format!("{} :", s.col_method));
                        ui.colored_label(Self::method_color(&acc.method), Self::method_label(&acc.method));
                    });
                    ui.horizontal(|ui| { ui.label(format!("{} :", s.col_contact)); ui.monospace(&acc.contact); });
                    
                    ui.separator();
                    ui.label(egui::RichText::new("Instructions :").strong());
                    ui.label(&acc.instructions);
                    if acc.requires_login { ui.colored_label(egui::Color32::from_rgb(220, 100, 50), s.requires_login_tip); }
                    
                    ui.add_space(15.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        if !acc.action_url.is_empty() {
                            if ui.button("🌐 Ouvrir le site").clicked() { open_url(&acc.action_url); }
                        }
                        if ui.button("📄 Télécharger le courrier").clicked() { download_draft(Path::new(&acc.draft), &acc.domain, s); }
                        if ui.button("🔗 Voir sur JustDeleteMe").clicked() && !acc.justdeleteme.is_empty() { open_url(&acc.justdeleteme); }
                    });
                });
            if !is_open { self.selected_account = None; }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(s.app_title);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("lang_selector").selected_text(self.lang.label())
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(self.lang == Lang::Fr, "🇫🇷 Français").clicked() { self.lang = Lang::Fr; }
                            if ui.selectable_label(self.lang == Lang::En, "🇬🇧 English").clicked() { self.lang = Lang::En; }
                        });
                    ui.label(s.language_label);
                });
            });

            ui.label(s.tagline);
            ui.colored_label(egui::Color32::from_rgb(220, 100, 50), s.reminder);
            ui.separator();

            egui::Grid::new("inputs").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                ui.label(s.email_label); ui.add(egui::TextEdit::singleline(&mut self.email).desired_width(340.0)); ui.end_row();
                ui.label(s.name_label); ui.add(egui::TextEdit::singleline(&mut self.name).desired_width(340.0)); ui.end_row();
            });

            let engine_path = find_engine();
            if engine_path.extension().map_or(false, |e| e.eq_ignore_ascii_case("py")) {
                ui.add_space(4.0);
                ui.collapsing(s.advanced_settings, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(s.python_label);
                        ui.add(egui::TextEdit::singleline(&mut self.python).desired_width(240.0)).on_hover_text(s.python_help);
                    });
                });
                ui.add_space(4.0);
            }

            ui.checkbox(&mut self.certified, s.certify);

            ui.horizontal(|ui| {
                if ui.add_enabled(self.can_start(), egui::Button::new(s.scan_button)).clicked() { self.start(ctx, s); }
                if self.running {
                    ui.spinner(); ui.label(s.scanning);
                    if ui.button("⏹ Stop").clicked() { self.stop(s); }
                }
                if let Some(dir) = self.out_dir.clone() {
                    if !self.running { if ui.button(s.open_folder).clicked() { open_path(&dir); } }
                }
            });

            ui.separator();
            ui.strong(s.log_title);
            egui::ScrollArea::vertical().id_salt("log").max_height(100.0).stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
                for l in &self.logs { ui.monospace(l); }
            });

            ui.separator();
            ui.horizontal(|ui| {
                ui.strong(format!("{} ({})", s.accounts_title, self.accounts.len()));
                ui.separator(); ui.label(s.filter_label);
                let filters = [
                    ("", s.filter_all), ("direct", s.filter_direct), ("form", s.filter_form),
                    ("email", s.filter_email), ("ticket", s.filter_ticket), ("impossible", s.filter_impossible),
                ];
                for (val, lbl) in filters {
                    let text = format!("{lbl} ({})", self.count_by(val));
                    if ui.selectable_label(self.method_filter == val, text).clicked() { self.method_filter = val.to_string(); }
                }
            });

            let filter = self.method_filter.clone();
            egui::ScrollArea::both().id_salt("accounts").auto_shrink([false, false]).show(ui, |ui| {
                egui::Grid::new("table").striped(true).num_columns(6).spacing([12.0, 6.0]).show(ui, |ui| {
                    // 👇 UTILISATION DE s.col_action ICI
                    for h in [s.col_site, s.col_method, s.col_contact, s.col_reliability, s.col_action, "Détails"] { 
                        ui.strong(h); 
                    }
                    ui.end_row();

                    for a in &self.accounts {
                        if !filter.is_empty() && a.method != filter { continue; }
                        ui.label(&a.domain).on_hover_text(&a.name);
                        ui.colored_label(Self::method_color(&a.method), Self::method_label(&a.method));
                        ui.label(&a.contact);
                        ui.label(&a.reliability);

                        match a.method.as_str() {
                            "direct" => { if ui.button("🗑️ Supprimer").clicked() && !a.action_url.is_empty() { open_url(&a.action_url); } }
                            "form" => { if ui.button("📝 Formulaire").clicked() && !a.action_url.is_empty() { open_url(&a.action_url); } }
                            "ticket" => { if ui.button("🎫 Ticket").clicked() && !a.action_url.is_empty() { open_url(&a.action_url); } }
                            "email" => { if ui.button("✉️ Télécharger").clicked() { download_draft(Path::new(&a.draft), &a.domain, s); } }
                            _ => { ui.label("❌ Impossible"); }
                        }

                        if ui.button("ℹ️").clicked() {
                            self.selected_account = Some(a.clone());
                        }
                        ui.end_row();
                    }
                });
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 720.0]).with_min_inner_size([720.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native("RGPD Cleaner", options, Box::new(|_cc| Ok(Box::new(App::default()))))
}