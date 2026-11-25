use eframe::egui::{self, Color32, RichText};
use serde_json::{json, Value};
use std::{
    io::{BufRead, BufReader, Write},
    os::unix::fs::MetadataExt,
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

#[derive(Clone, Default)]
struct Disk {
    path: String,
    model: String,
    serial: String,
    capacity: String,
    protocol: String,
    temp: String,
    hours: String,
    health: String,
    life: Option<u8>,
    attrs: Vec<(String, String, String, String, String)>,
}

struct SmartSession {
    _child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}

impl SmartSession {
    fn start() -> Result<Self, String> {
        let helper = Path::new("/usr/local/libexec/rustdiskinfo/smart-helper");
        if !trusted_root_file(helper) {
            return Err("Privileged helper is not installed. Run sudo ./scripts/install.sh.".into());
        }
        let mut child = Command::new("/usr/bin/pkexec")
            .arg(helper)
            .arg("--smart-helper")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Could not start the authorization helper: {e}"))?;
        let input = child.stdin.take().ok_or("Could not open helper input")?;
        let output = child.stdout.take().ok_or("Could not open helper output")?;
        Ok(Self {
            _child: child,
            input,
            output: BufReader::new(output),
        })
    }

    fn scan(&mut self) -> Result<Vec<Disk>, String> {
        writeln!(self.input, "scan").map_err(|_| "Authorization helper has exited".to_string())?;
        self.input.flush().map_err(|_| "Could not contact authorization helper".to_string())?;
        let mut line = String::new();
        self.output
            .read_line(&mut line)
            .map_err(|e| format!("Could not read SMART data: {e}"))?;
        if line.is_empty() {
            return Err("Authorization was cancelled or the helper exited. Restart the app to try again.".into());
        }
        let response: Value = serde_json::from_str(&line).map_err(|e| format!("Invalid helper response: {e}"))?;
        Ok(parse_disks(&response))
    }
}

struct App {
    disks: Vec<Disk>,
    selected: usize,
    message: String,
    session: Option<SmartSession>,
}

impl Default for App {
    fn default() -> Self {
        let mut session = SmartSession::start().ok();
        let (disks, message) = if let Some(session) = session.as_mut() {
            match session.scan() {
                Ok(disks) => {
                    let message = if disks.is_empty() {
                        "No drives found. Check the connected devices.".into()
                    } else {
                        drive_count_label(disks.len())
                    };
                    (disks, message)
                }
                Err(error) => (Vec::new(), error),
            }
        } else {
            (Vec::new(), "Authorization helper unavailable. Reinstall with sudo ./scripts/install.sh.".into())
        };
        Self { disks, selected: 0, message, session }
    }
}

impl App {
    fn demo() -> Self {
        Self {
            disks: vec![
                Disk {
                    path: "/dev/nvme0".into(), model: "Samsung SSD 990 PRO 2TB".into(),
                    serial: "S6Z2NX0W123456".into(), capacity: "2.00 TB".into(), protocol: "NVMe".into(),
                    temp: "39 °C".into(), hours: "1,284 h".into(), health: "Healthy".into(), life: Some(96),
                    attrs: vec![
                        ("—".into(), "Critical warnings".into(), "None".into(), "—".into(), "0x00".into()),
                        ("—".into(), "Available spare".into(), "100%".into(), "—".into(), "100".into()),
                        ("—".into(), "Percentage used".into(), "4%".into(), "—".into(), "4".into()),
                        ("—".into(), "Data units read".into(), "6.6 TB".into(), "—".into(), "12,841,920 units".into()),
                        ("—".into(), "Data units written".into(), "4.3 TB".into(), "—".into(), "8,412,605 units".into()),
                        ("—".into(), "Power-on hours".into(), "1,284 h".into(), "—".into(), "1,284 h".into()),
                        ("—".into(), "Power cycles".into(), "246 cycles".into(), "—".into(), "246 cycles".into()),
                    ],
                },
                Disk {
                    path: "/dev/sda".into(), model: "Crucial MX500 1TB".into(), serial: "2219E6A01234".into(),
                    capacity: "1.00 TB".into(), protocol: "SATA".into(), temp: "32 °C".into(), hours: "4,812 h".into(),
                    health: "Healthy".into(), life: Some(88), attrs: vec![],
                },
            ],
            selected: 0,
            message: "2 drives detected".into(),
            session: None,
        }
    }
}

impl App {
    fn refresh(&mut self) {
        if let Some(session) = self.session.as_mut() {
            match session.scan() {
                Ok(disks) => {
                    self.disks = disks;
                    self.selected = self.selected.min(self.disks.len().saturating_sub(1));
                    self.message = if self.disks.is_empty() {
                        "No drives found. Check the connected devices.".into()
                    } else {
                        drive_count_label(self.disks.len())
                    };
                }
                Err(error) => self.message = error,
            }
        }
    }
}

fn drive_count_label(count: usize) -> String {
    if count == 1 { "1 drive detected".into() } else { format!("{count} drives detected") }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let bg = Color32::from_rgb(18, 22, 29);
        let card = Color32::from_rgb(27, 33, 42);
        let muted = Color32::from_rgb(145, 158, 174);
        let green = Color32::from_rgb(65, 190, 152);
        ctx.set_visuals(egui::Visuals::dark());
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(bg).inner_margin(egui::Margin::same(18)))
            .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(Color32::from_rgb(35, 42, 52))
                .inner_margin(egui::Margin::symmetric(18, 10))
                .show(ui, |ui| {
                    ui.set_min_height(38.0);
                    ui.horizontal(|ui| {
                        let (logo_rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());
                        ui.painter().circle_filled(logo_rect.center(), 8.0, green);
                        ui.painter().circle_filled(logo_rect.center(), 4.0, Color32::from_rgb(23, 33, 42));
                        ui.label(RichText::new("RustDiskInfo").size(16.0).strong());
                        ui.label(RichText::new("S.M.A.R.T. MONITOR").size(10.0).color(muted));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("⟳  Refresh").clicked() { self.refresh(); }
                            ui.label(RichText::new(&self.message).size(11.0).color(muted));
                        });
                    });
                });
            ui.add_space(8.0);

            ui.horizontal_wrapped(|ui| {
                for (i, disk) in self.disks.iter().enumerate() {
                    let selected = self.selected == i;
                    let health_color = status_color(&disk.health, green);
                    let (rect, response) = ui.allocate_exact_size(egui::vec2(420.0, 66.0), egui::Sense::click());
                    let fill = if selected { Color32::from_rgb(41, 63, 67) } else { card };
                    ui.painter().rect_filled(rect, 10.0, fill);
                    if selected {
                        ui.painter().rect_stroke(rect, 10.0, egui::Stroke::new(1.0_f32, green), egui::StrokeKind::Inside);
                    }
                    let center = rect.left_center() + egui::vec2(24.0, 0.0);
                    ui.painter().circle_filled(center, 6.0, health_color);
                    ui.painter().text(rect.left_top() + egui::vec2(43.0, 13.0), egui::Align2::LEFT_TOP,
                        &disk.path, egui::FontId::proportional(14.0), Color32::from_rgb(231, 237, 245));
                    ui.painter().text(rect.left_top() + egui::vec2(43.0, 36.0), egui::Align2::LEFT_TOP,
                        format!("{}  •  {}", disk.health, disk.temp), egui::FontId::proportional(12.0), muted);
                    if response.clicked() { self.selected = i; }
                }
                if self.disks.is_empty() {
                    ui.label(RichText::new("No drives detected").color(muted));
                }
            });
            ui.add_space(12.0);

            if let Some(disk) = self.disks.get(self.selected) {
                drive_view(ui, disk, card, muted, green);
            } else {
                egui::Frame::NONE.fill(card).corner_radius(10.0).inner_margin(24.0).show(ui, |ui| {
                    ui.set_min_height(180.0);
                    ui.vertical_centered(|ui| {
                        ui.add_space(24.0);
                        ui.label(RichText::new("No storage devices available").size(21.0).strong());
                        ui.add_space(8.0);
                        ui.label(RichText::new(&self.message).color(muted));
                        ui.add_space(18.0);
                        if ui.button("⟳  Retry scan").clicked() { self.refresh(); }
                    });
                });
            }
        });
    }
}

fn drive_view(ui: &mut egui::Ui, d: &Disk, card: Color32, muted: Color32, green: Color32) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(&d.model).size(27.0).strong());
            ui.label(RichText::new(format!("{}  •  {}  •  Serial {}", d.path, d.protocol, d.serial)).size(12.0).color(muted));
        });
    });
    ui.add_space(12.0);

    ui.columns(3, |columns| {
        columns[0].set_min_width(190.0);
        columns[1].set_min_width(190.0);
        columns[2].set_min_width(190.0);
        let health_color = status_color(&d.health, green);
        gauge(&mut columns[0], "HEALTH STATUS", &d.health, health_color, card, 0.96);
        let life_value = d.life.map(|n| format!("{n}%")).unwrap_or_else(|| "—".into());
        let life_color = d.life.map(life_color).unwrap_or(muted);
        gauge(&mut columns[1], "SSD LIFE", &life_value, life_color, card, d.life.unwrap_or(0) as f32 / 100.0);
        gauge(&mut columns[2], "TEMPERATURE", &d.temp, Color32::from_rgb(246, 177, 78), card, 0.48);
    });
    ui.add_space(12.0);

    let panel_width = ui.max_rect().width();
    // Keep the dashboard compact enough to fit in the initial window without scrolling.
    let panel_height = ui.available_height().clamp(300.0, 360.0);
    let details_width = (panel_width * 0.32).clamp(320.0, 440.0);
    let table_width = (panel_width - details_width - 16.0).max(320.0);
    ui.horizontal_top(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(details_width, panel_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| egui::Frame::NONE.fill(card).corner_radius(9.0).inner_margin(14.0).show(ui, |ui| {
            ui.set_min_width((details_width - 28.0).max(280.0));
            ui.set_min_height((panel_height - 28.0).max(180.0));
            ui.label(RichText::new("DRIVE DETAILS").strong().size(12.0).color(muted));
            ui.add_space(8.0);
            detail_row(ui, "Capacity", &d.capacity);
            detail_row(ui, "Power-on time", &d.hours);
            detail_row(ui, "Interface", &d.protocol);
            detail_row(ui, "Serial number", &d.serial);
            ui.add_space(8.0);
            ui.label(RichText::new("SSD life is estimated from the drive's SMART endurance data.").size(12.0).color(muted));
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                ui.horizontal(|ui| {
                    let (mark, _) = ui.allocate_exact_size(egui::vec2(62.0, 62.0), egui::Sense::hover());
                    ui.painter().circle_stroke(mark.center(), 30.0, egui::Stroke::new(2.0_f32, green));
                    ui.painter().line_segment([mark.center() - egui::vec2(0.0, 16.0), mark.center() + egui::vec2(0.0, 16.0)], egui::Stroke::new(3.0_f32, green));
                    ui.painter().line_segment([mark.center() - egui::vec2(16.0, 0.0), mark.center() + egui::vec2(16.0, 0.0)], egui::Stroke::new(3.0_f32, green));
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Bundled smartctl").size(13.0).strong());
                        ui.label(RichText::new("Local SMART data • no cloud upload").size(11.0).color(muted));
                    });
                });
            });
        }),
        );
        ui.add_space(16.0);
        ui.allocate_ui_with_layout(
            egui::vec2(table_width, panel_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| egui::Frame::NONE.fill(card).corner_radius(9.0).inner_margin(14.0).show(ui, |ui| {
            ui.set_min_width((table_width - 28.0).max(280.0));
            ui.set_min_height((panel_height - 28.0).max(180.0));
            ui.label(RichText::new("S.M.A.R.T. ATTRIBUTES").strong().size(12.0).color(muted));
            egui::ScrollArea::vertical().auto_shrink([false, false]).max_height((panel_height - 58.0).max(140.0)).show(ui, |ui| {
                // ScrollArea sizes its contents to their natural width by default; keep the
                // table aligned with the full SMART card so columns use the available space.
                ui.set_min_width((table_width - 56.0).max(300.0));
                let grid_width = (table_width - 56.0).max(300.0);
                let data_width = grid_width - 48.0;
                    let widths = [48.0, data_width * 0.52, data_width * 0.15, data_width * 0.14, data_width * 0.19];
                    egui::Grid::new("smart_attributes").striped(true).num_columns(5).show(ui, |ui| {
                    for (index, heading) in ["ID", "Attribute", "Current", "Worst", "Raw value"].iter().enumerate() {
                        ui.add_sized([widths[index], 22.0], egui::Label::new(RichText::new(*heading).size(12.0).strong().color(muted)).halign(egui::Align::Min));
                    }
                    ui.end_row();
                    for (id, name, value, worst, raw) in &d.attrs {
                        for (index, cell) in [id, name, value, worst, raw].iter().enumerate() {
                            ui.add_sized([widths[index], 23.0], egui::Label::new(RichText::new(*cell).size(13.0)).halign(egui::Align::Min));
                        }
                        ui.end_row();
                    }
                    if d.attrs.is_empty() { ui.label("No SMART attributes reported."); }
                });
            });
        }),
        );
    });
}

fn gauge(ui: &mut egui::Ui, title: &str, value: &str, color: Color32, fill: Color32, fraction: f32) {
    egui::Frame::NONE.fill(fill).corner_radius(9.0).inner_margin(14.0).show(ui, |ui| {
        ui.set_min_size(egui::vec2(0.0, 106.0));
        ui.label(RichText::new(title).size(12.0).strong());
        ui.add_space(6.0);
        ui.label(RichText::new(value).size(27.0).strong().color(color));
        ui.add_space(5.0);
        let fraction = fraction.clamp(0.0, 1.0);
        ui.add(egui::ProgressBar::new(fraction).desired_height(7.0).fill(color).corner_radius(4.0));
    });
}

fn detail_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).color(Color32::from_rgb(145, 158, 174)));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(value); });
    });
    ui.separator();
}

fn status_color(status: &str, green: Color32) -> Color32 {
    if status == "FAILED" || status == "Unavailable" { Color32::from_rgb(240, 85, 85) } else { green }
}

fn life_color(life: u8) -> Color32 {
    if life <= 10 { Color32::from_rgb(240, 85, 85) }
    else if life <= 25 { Color32::from_rgb(246, 177, 78) }
    else { Color32::from_rgb(65, 190, 152) }
}

fn trusted_root_file(path: &Path) -> bool {
    let mut current = path;
    loop {
        let Ok(metadata) = current.metadata() else { return false };
        if metadata.uid() != 0 || metadata.mode() & 0o022 != 0 { return false; }
        let Some(parent) = current.parent() else { return true };
        if parent == current { return true; }
        current = parent;
    }
}

fn smartctl() -> Command {
    let mut executable = std::env::current_exe().unwrap_or_default();
    executable.set_file_name("smartctl");
    let mut command = Command::new(executable);
    let mut database = std::env::current_exe().unwrap_or_default();
    database.set_file_name("drivedb.h");
    if database.is_file() { command.arg("-B").arg(database); }
    command
}

fn read_raw_devices() -> Value {
    let Ok(scan_output) = smartctl().args(["--scan-open", "-j"]).output() else { return json!({"devices": []}) };
    let scan: Value = serde_json::from_slice(&scan_output.stdout).unwrap_or(Value::Null);
    let mut devices = Vec::new();
    for device in scan["devices"].as_array().into_iter().flatten() {
        let Some(path) = device["name"].as_str() else { continue };
        let dtype = device["type"].as_str().unwrap_or("");
        let mut args = vec!["-a".to_string(), "-j".to_string(), path.to_string()];
        if !dtype.is_empty() { args.extend(["-d".into(), dtype.into()]); }
        let Ok(output) = smartctl().args(&args).output() else { continue };
        if let Ok(data) = serde_json::from_slice::<Value>(&output.stdout) {
            devices.push(json!({"path": path, "type": dtype, "data": data}));
        }
    }
    json!({"devices": devices})
}

fn helper_main() -> std::io::Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for command in stdin.lock().lines() {
        let Ok(command) = command else { break };
        if command.trim() == "scan" {
            serde_json::to_writer(&mut stdout, &read_raw_devices())?;
            writeln!(stdout)?;
            stdout.flush()?;
        }
    }
    Ok(())
}

fn json_text(value: &Value) -> String {
    value.as_str().map(str::to_owned)
        .or_else(|| value.as_i64().map(|n| n.to_string()))
        .or_else(|| value.as_u64().map(|n| n.to_string()))
        .unwrap_or_else(|| "—".into())
}

fn format_integer(value: u64) -> String {
    let digits = value.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 { grouped.push(','); }
        grouped.push(digit);
    }
    grouped
}

fn format_bytes(bytes: u128) -> String {
    if bytes >= 1_000_000_000_000 {
        format!("{:.1} TB", bytes as f64 / 1_000_000_000_000.0)
    } else if bytes >= 1_000_000_000 {
        format!("{:.1} GB", bytes as f64 / 1_000_000_000.0)
    } else {
        format!("{:.0} MB", bytes as f64 / 1_000_000.0)
    }
}

fn format_capacity(bytes: u64) -> String {
    let (value, unit) = if bytes >= 1_000_000_000_000 {
        (bytes as f64 / 1_000_000_000_000.0, "TB")
    } else if bytes >= 1_000_000_000 {
        (bytes as f64 / 1_000_000_000.0, "GB")
    } else if bytes >= 1_000_000 {
        (bytes as f64 / 1_000_000.0, "MB")
    } else if bytes >= 1_000 {
        (bytes as f64 / 1_000.0, "KB")
    } else {
        return format!("{bytes} B");
    };
    if value >= 100.0 { format!("{value:.0} {unit}") }
    else if value >= 10.0 { format!("{value:.1} {unit}") }
    else { format!("{value:.2} {unit}") }
}

fn nvme_attribute_value(key: &str, value: &Value) -> (String, String) {
    let Some(number) = value.as_u64() else {
        let value = json_text(value);
        return (value.clone(), value);
    };
    let number_text = format_integer(number);
    match key {
        "critical_warning" => {
            let status = if number == 0 { "None".into() } else { format!("0x{number:02X}") };
            (status, format!("0x{number:02X}"))
        }
        "available_spare" | "available_spare_threshold" => (format!("{number}%"), format!("{number}%")),
        "percentage_used" => (format!("{number}%"), format!("{number}%")),
        "data_units_read" | "data_units_written" => {
            (format_bytes(number as u128 * 512_000), format!("{number_text} units"))
        }
        "power_on_hours" => (format!("{number_text} h"), format!("{number_text} h")),
        "power_cycles" => (format!("{number_text} cycles"), format!("{number_text} cycles")),
        "unsafe_shutdowns" => (format!("{number_text} events"), format!("{number_text} events")),
        "media_errors" => (format!("{number_text} errors"), format!("{number_text} errors")),
        "num_err_log_entries" => (format!("{number_text} entries"), format!("{number_text} entries")),
        _ => (number_text.clone(), number_text),
    }
}

fn parse_disks(response: &Value) -> Vec<Disk> {
    response["devices"].as_array().into_iter().flatten().filter_map(|entry| {
        let path = entry["path"].as_str()?.to_string();
        let dtype = entry["type"].as_str().unwrap_or("");
        let j = &entry["data"];
        let nvme = &j["nvme_smart_health_information_log"];
        let attrs: Vec<(String, String, String, String, String)> = j["ata_smart_attributes"]["table"]
            .as_array().into_iter().flatten().map(|a| {
                let raw = a["raw"]["string"].as_str().map(str::to_owned)
                    .or_else(|| a["raw"]["value"].as_i64().map(|n| n.to_string())).unwrap_or_else(|| "—".into());
                (json_text(&a["id"]), a["name"].as_str().unwrap_or("Attribute").replace('_', " "),
                 json_text(&a["value"]), json_text(&a["worst"]), raw)
            }).collect();
        let life = nvme["percentage_used"].as_u64().map(|used| 100u64.saturating_sub(used).min(100) as u8)
            .or_else(|| ata_life(j));
        let mut attrs = attrs;
        if attrs.is_empty() {
            for (name, key) in [("Critical warnings", "critical_warning"), ("Available spare", "available_spare"),
                ("Spare threshold", "available_spare_threshold"), ("Percentage used", "percentage_used"),
                ("Data units read", "data_units_read"), ("Data units written", "data_units_written"),
                ("Power-on hours", "power_on_hours"), ("Power cycles", "power_cycles"),
                ("Unsafe shutdowns", "unsafe_shutdowns"), ("Media errors", "media_errors"),
                ("Error log entries", "num_err_log_entries")] {
                if !nvme[key].is_null() {
                    let (value, raw) = nvme_attribute_value(key, &nvme[key]);
                    attrs.push(("—".into(), name.into(), value, "—".into(), raw));
                }
            }
        }
        let bytes = j["user_capacity"]["bytes"].as_u64().unwrap_or(0);
        let serial = j["serial_number"].as_str().or(j["device"]["serial_number"].as_str()).or(j["nvme_id_ctrl"]["serial_number"].as_str()).unwrap_or("—");
        let model = j["model_name"].as_str().or(j["device"]["model_name"].as_str()).or(j["nvme_id_ctrl"]["model_number"].as_str()).unwrap_or("Drive");
        let passed = j["smart_status"]["passed"].as_bool().or_else(|| nvme["critical_warning"].as_u64().map(|warning| warning == 0));
        let health = match passed { Some(true) => "Healthy", Some(false) => "FAILED", None => "Unavailable" };
        Some(Disk {
            path, model: model.into(), serial: serial.into(),
            capacity: if bytes > 0 { format_capacity(bytes) } else { "—".into() },
            protocol: j["device"]["protocol"].as_str().or(j["device"]["type"].as_str()).unwrap_or(dtype).into(),
            temp: j["temperature"]["current"].as_i64().or(nvme["temperature"].as_i64()).map(|n| format!("{n} °C")).unwrap_or_else(|| "— °C".into()),
            hours: j["power_on_time"]["hours"].as_u64().or(nvme["power_on_hours"].as_u64()).map(|n| format!("{} h", format_integer(n))).unwrap_or_else(|| "—".into()),
            health: health.into(), life, attrs,
        })
    }).collect()
}

fn ata_life(data: &Value) -> Option<u8> {
    data["ata_smart_attributes"]["table"].as_array()?.iter().find_map(|attr| {
        let name = attr["name"].as_str()?.to_ascii_lowercase().replace([' ', '_', '-'], "");
        let value = attr["value"].as_u64().or_else(|| attr["raw"]["value"].as_u64())?;
        let remaining = if ["percentlifetimeremain", "percentliferemain", "percentlifeleft", "ssdlifeleft", "remaininglifetime"]
            .iter().any(|key| name == *key) || name == "mediawearoutindicator" {
            value.min(100)
        } else if name == "percentlifetimeused" || name == "percentageused" {
            100u64.saturating_sub(value)
        } else { return None };
        Some(remaining as u8)
    })
}

fn main() -> eframe::Result<()> {
    let arg = std::env::args().nth(1);
    if arg.as_deref() == Some("--smart-helper") {
        return helper_main().map_err(|e| eframe::Error::AppCreation(Box::new(e)));
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 820.0]).with_min_inner_size([1040.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native("RustDiskInfo", options, Box::new(move |_| {
        if arg.as_deref() == Some("--demo") { Ok(Box::new(App::demo())) }
        else { Ok(Box::<App>::default()) }
    }))
}
