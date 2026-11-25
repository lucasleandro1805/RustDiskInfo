use eframe::egui::{self,Color32,RichText};
use serde_json::Value;
use std::{os::unix::fs::MetadataExt,path::Path,process::Command};
#[derive(Clone,Default)]
struct Disk{path:String,model:String,serial:String,capacity:String,protocol:String,temp:String,hours:String,health:String,attrs:Vec<(String,String,String,String,String)>}
struct App{disks:Vec<Disk>,selected:usize,message:String,started:bool}
fn smartctl()->Command{
let mut bundled=std::env::current_exe().unwrap_or_default();bundled.set_file_name("smartctl");
let executable=if bundled.is_file(){Some(bundled)}else{std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()).map(|p|p.join("smartctl")).find(|p|p.is_file())};
let mut command=if let Some(path)=executable{Command::new(path)}else{Command::new("smartctl")};
let mut database=std::env::current_exe().unwrap_or_default();database.set_file_name("drivedb.h");
if database.is_file(){command.arg("-B").arg(database);}
command
}
fn trusted_root_file(path:&Path)->bool{let mut current=path;loop{let Ok(metadata)=current.metadata()else{return false};if metadata.uid()!=0||metadata.mode()&0o022!=0{return false;}let Some(parent)=current.parent()else{return true};if parent==current{return true;}current=parent;}}
fn privileged_smartctl()->Option<Command>{
let directory=Path::new("/usr/local/libexec/rustdiskinfo");let executable=directory.join("smartctl");let database=directory.join("drivedb.h");
if !trusted_root_file(&executable)||!trusted_root_file(&database){return None;}
let mut command=Command::new("/usr/bin/pkexec");command.arg(executable).arg("-B").arg(database);Some(command)
}
impl Default for App{fn default()->Self{Self{disks:vec![],selected:0,message:"Use Abrir monitor para consultar as unidades.".into(),started:false}}}
impl App{fn refresh(&mut self){self.disks=scan();self.selected=self.selected.min(self.disks.len().saturating_sub(1));self.message=if self.disks.is_empty(){"Nenhum disco encontrado. Verifique os dispositivos conectados.".into()}else if self.disks.iter().any(|d|d.health=="Acesso negado"){if privileged_smartctl().is_some(){"Discos detectados; autentique no aviso do sistema para ler o S.M.A.R.T.".into()}else{"Discos detectados, mas sem acesso. Reinstale com sudo ./install.sh para habilitar a autenticação.".into()}}else{format!("{} disco(s) • dados via smartctl",self.disks.len())};}}
impl eframe::App for App{fn update(&mut self,ctx:&egui::Context,_:&mut eframe::Frame){
let bg=Color32::from_rgb(18,22,29);let card=Color32::from_rgb(27,33,42);let muted=Color32::from_rgb(145,158,174);let green=Color32::from_rgb(65,190,152);ctx.set_visuals(egui::Visuals::dark());
if !self.started { welcome(ctx,self,bg,card,muted,green); return; }
egui::CentralPanel::default().frame(egui::Frame::NONE.fill(bg)).show(ctx,|ui|{
ui.add_space(12.);ui.horizontal(|ui|{ui.label(RichText::new("◉  RUSTDISKINFO").size(17.).strong());ui.label(RichText::new("S.M.A.R.T. MONITOR").size(10.).color(muted));ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{if ui.button("⌂ Início").clicked(){self.started=false}if ui.button("⟳ Atualizar").clicked(){self.refresh()}ui.label(RichText::new(&self.message).size(11.).color(muted));});});ui.add_space(12.);
ui.columns(2,|c|{c[0].set_min_width(230.);egui::Frame::NONE.fill(card).corner_radius(10.).inner_margin(14.).show(&mut c[0],|ui|{ui.label(RichText::new("UNIDADES").size(10.).strong().color(muted));ui.add_space(8.);if self.disks.is_empty(){ui.label("Nenhuma unidade disponível");}for(i,d)in self.disks.iter().enumerate(){let active=self.selected==i;let t=format!("●  {}\n    {} • {}",d.path,d.model,d.capacity);if ui.add_sized([ui.available_width(),56.],egui::Button::new(RichText::new(t).size(12.)).fill(if active{Color32::from_rgb(39,61,65)}else{card})).clicked(){self.selected=i;}}ui.add_space(12.);ui.label(RichText::new("Sobre SMART").strong().size(11.).color(muted));ui.label(RichText::new("A leitura usa smartctl, empacotado junto ao programa. Alguns dispositivos exigem permissões elevadas.").size(10.).color(muted));});
egui::Frame::NONE.fill(bg).inner_margin(egui::Margin::symmetric(14,0)).show(&mut c[1],|ui|{if let Some(d)=self.disks.get(self.selected){view(ui,d,card,muted,green)}else{ui.centered_and_justified(|ui|{ui.label("Conecte um disco para começar");});}});
});});}}
fn welcome(ctx:&egui::Context,app:&mut App,bg:Color32,card:Color32,muted:Color32,green:Color32){
egui::CentralPanel::default().frame(egui::Frame::NONE.fill(bg)).show(ctx,|ui|{
ui.vertical_centered(|ui|{
ui.add_space(56.);
ui.label(RichText::new("◉").size(64.).strong().color(green));
ui.add_space(8.);
ui.label(RichText::new("RustDiskInfo").size(34.).strong());
ui.label(RichText::new("Saúde dos seus discos, de relance.").size(16.).color(muted));
ui.add_space(26.);
egui::Frame::NONE.fill(card).corner_radius(14.).inner_margin(24.).show(ui,|ui|{
ui.set_max_width(560.);
ui.label(RichText::new("MONITORAMENTO S.M.A.R.T.").size(11.).strong().color(green));
ui.add_space(8.);
ui.label(RichText::new("Acompanhe a saúde, temperatura e atributos dos seus discos em um só lugar.").size(15.));
ui.add_space(16.);
ui.horizontal(|ui|{
ui.label(RichText::new(format!("{} unidade(s) detectada(s)",app.disks.len())).color(muted));
ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
if ui.add(egui::Button::new(RichText::new("Abrir monitor  →").strong()).fill(green)).clicked(){app.refresh();app.started=true;}
if ui.button("⟳ Atualizar").clicked(){app.refresh();}
});
});
if app.disks.is_empty(){ui.add_space(10.);ui.label(RichText::new(&app.message).size(11.).color(muted));}
});
ui.add_space(20.);
ui.label(RichText::new("Leitura fornecida pelo smartctl • Os dados permanecem no seu computador").size(11.).color(muted));
});
});
}
fn view(ui:&mut egui::Ui,d:&Disk,card:Color32,muted:Color32,green:Color32){ui.label(RichText::new(&d.model).size(22.).strong());ui.label(RichText::new(format!("{} • {} • SN {}",d.path,d.protocol,d.serial)).size(11.).color(muted));ui.add_space(12.);ui.horizontal(|ui|{let health_color=if d.health=="Acesso negado"||d.health=="FAILED"{Color32::from_rgb(240,85,85)}else{green};metric(ui,"SAÚDE",&d.health,health_color,card);metric(ui,"TEMPERATURA",&d.temp,Color32::from_rgb(246,177,78),card);metric(ui,"TEMPO LIGADO",&d.hours,Color32::from_rgb(106,173,236),card);});ui.add_space(12.);egui::Frame::NONE.fill(card).corner_radius(9.).inner_margin(12.).show(ui,|ui|{ui.label(RichText::new("ATRIBUTOS S.M.A.R.T.").strong().size(10.).color(muted));egui::ScrollArea::vertical().max_height(460.).show(ui,|ui|{egui::Grid::new("smart").striped(true).show(ui,|ui|{for h in ["ID","Atributo","Valor","Pior","Bruto"]{ui.label(RichText::new(h).strong().color(muted));}ui.end_row();for(id,n,v,w,r)in &d.attrs{ui.label(id);ui.label(n);ui.label(v);ui.label(w);ui.label(r);ui.end_row();}if d.attrs.is_empty(){ui.label("Sem atributos SMART informados.");}});});});}
fn metric(ui:&mut egui::Ui,title:&str,value:&str,color:Color32,fill:Color32){egui::Frame::NONE.fill(fill).corner_radius(9.).inner_margin(12.).show(ui,|ui|{ui.set_min_size(egui::vec2(126.,67.));ui.label(RichText::new(title).size(9.).strong());ui.label(RichText::new(value).size(20.).strong().color(color));});}
fn permission_error(j:&Value)->bool{let messages=j["messages"].as_array().into_iter().flatten().filter_map(|m|m["string"].as_str()).collect::<Vec<_>>().join(" ").to_lowercase();messages.contains("permission denied")||messages.contains("operation not permitted")}
fn json_text(v:&Value)->String{v.as_str().map(str::to_owned).or_else(||v.as_i64().map(|n|n.to_string())).or_else(||v.as_u64().map(|n|n.to_string())).unwrap_or_else(||"—".into())}
fn scan()->Vec<Disk>{let Ok(o)=smartctl().args(["--scan-open","-j"]).output()else{return vec![]};let scan:Value=serde_json::from_slice(&o.stdout).unwrap_or(Value::Null);scan["devices"].as_array().into_iter().flatten().filter_map(|dev|{
let path=dev["name"].as_str()?.to_string();let dtype=dev["type"].as_str().unwrap_or("");let mut args=vec!["-a".to_string(),"-j".to_string(),path.clone()];if !dtype.is_empty(){args.push("-d".into());args.push(dtype.into());}
let direct=smartctl().args(&args).output().ok().and_then(|o|serde_json::from_slice::<Value>(&o.stdout).ok());
let scan_error=dev["open_error"].as_str().unwrap_or("").to_lowercase();let needs_auth=scan_error.contains("permission denied")||scan_error.contains("operation not permitted")||direct.as_ref().is_some_and(permission_error);
let elevated=if needs_auth{privileged_smartctl().and_then(|mut c|c.args(&args).output().ok()).and_then(|o|serde_json::from_slice::<Value>(&o.stdout).ok()).filter(|j|!permission_error(j))}else{None};let j=elevated.or(direct)?;
let mut attrs:Vec<(String,String,String,String,String)>=j["ata_smart_attributes"]["table"].as_array().into_iter().flatten().map(|a|{let raw=a["raw"]["string"].as_str().map(str::to_owned).or_else(||a["raw"]["value"].as_i64().map(|n|n.to_string())).unwrap_or_else(||"—".into());(a["id"].to_string().trim_matches('"').into(),a["name"].as_str().unwrap_or("Atributo").replace('_'," "),a["value"].to_string().trim_matches('"').into(),a["worst"].to_string().trim_matches('"').into(),raw)}).collect();
let nvme=&j["nvme_smart_health_information_log"];if attrs.is_empty(){for (id,name,key) in [("—","Avisos críticos","critical_warning"),("—","Reserva disponível","available_spare"),("—","Limite de reserva","available_spare_threshold"),("—","Desgaste estimado","percentage_used"),("—","Unidades de dados lidas","data_units_read"),("—","Unidades de dados gravadas","data_units_written"),("—","Horas ligado","power_on_hours"),("—","Ciclos de energia","power_cycles"),("—","Desligamentos inseguros","unsafe_shutdowns"),("—","Erros de mídia","media_errors"),("—","Entradas no log de erros","num_err_log_entries")] {if !nvme[key].is_null(){let value=json_text(&nvme[key]);attrs.push((id.into(),name.into(),value.clone(),"—".into(),value));}}}
let bytes=j["user_capacity"]["bytes"].as_u64().unwrap_or(0);let serial=j["serial_number"].as_str().or(j["device"]["serial_number"].as_str()).or(j["nvme_id_ctrl"]["serial_number"].as_str()).unwrap_or("—");let model=j["model_name"].as_str().or(j["device"]["model_name"].as_str()).or(j["nvme_id_ctrl"]["model_number"].as_str()).unwrap_or(if permission_error(&j){"Acesso negado"}else{"Disco"});
let passed=j["smart_status"]["passed"].as_bool().or_else(||nvme["critical_warning"].as_u64().map(|warning|warning==0));let health=if permission_error(&j){"Acesso negado"}else{match passed{Some(true)=>"Saudável",Some(false)=>"FAILED",None=>"Indisponível"}};
Some(Disk{path,model:model.into(),serial:serial.into(),capacity:if bytes>0{format!("{:.0} GB",bytes as f64/1e9)}else{"—".into()},protocol:j["device"]["protocol"].as_str().or(j["device"]["type"].as_str()).unwrap_or(dtype).into(),temp:j["temperature"]["current"].as_i64().or(nvme["temperature"].as_i64()).map(|n|format!("{n} °C")).unwrap_or_else(||"-- °C".into()),hours:j["power_on_time"]["hours"].as_u64().or(nvme["power_on_hours"].as_u64()).map(|n|format!("{n} h")).unwrap_or_else(||"—".into()),health:health.into(),attrs})
}).collect()}
fn main()->eframe::Result<()>{let opts=eframe::NativeOptions{viewport:egui::ViewportBuilder::default().with_inner_size([1000.,680.]).with_min_inner_size([740.,520.]),..Default::default()};eframe::run_native("RustDiskInfo",opts,Box::new(|_|Ok(Box::<App>::default())))}
