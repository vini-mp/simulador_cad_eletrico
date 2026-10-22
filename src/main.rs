use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// Paleta escura inspirada na referência enviada pelo usuário.
const BG: Color32 = Color32::from_rgb(32, 36, 42);
const RIBBON: Color32 = Color32::from_rgb(42, 46, 53);
const RIBBON_DARK: Color32 = Color32::from_rgb(27, 30, 35);
const PANEL: Color32 = Color32::from_rgb(37, 41, 47);
const BUTTON: Color32 = Color32::from_rgb(52, 58, 66);
const BUTTON_HOVER: Color32 = Color32::from_rgb(68, 76, 86);
const BUTTON_ACTIVE: Color32 = Color32::from_rgb(48, 86, 116);
const CANVAS: Color32 = Color32::from_rgb(29, 33, 38);
const GRID: Color32 = Color32::from_rgb(43, 48, 55);
const GRID_MAJOR: Color32 = Color32::from_rgb(54, 60, 68);
const TEXT: Color32 = Color32::from_rgb(218, 222, 227);
const MUTED: Color32 = Color32::from_rgb(145, 154, 164);
const ACCENT: Color32 = Color32::from_rgb(67, 169, 225);
const WIRE_COLOR: Color32 = Color32::from_rgb(211, 217, 223);
const PENDING_COLOR: Color32 = Color32::from_rgb(246, 190, 77);
const TERMINAL: Color32 = Color32::from_rgb(104, 197, 150);
const DANGER: Color32 = Color32::from_rgb(210, 94, 94);
const GRID_STEP: f32 = 20.0;
const NODE_RADIUS: f32 = 17.0;
const PORT_OFFSET: f32 = 28.0;

fn dark_visuals() -> egui::Visuals {
    // Tema definido pelo próprio programa. Não depende do tema do Windows.
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = PANEL;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = RIBBON_DARK;
    visuals.faint_bg_color = BUTTON;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    visuals.widgets.inactive.bg_fill = BUTTON;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT);
    visuals.widgets.hovered.bg_fill = BUTTON_HOVER;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.widgets.active.bg_fill = BUTTON_ACTIVE;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.widgets.open.bg_fill = BUTTON_ACTIVE;
    visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.selection.bg_fill = BUTTON_ACTIVE;
    visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);
    visuals
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Simulador")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1050.0, 680.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Simulador",
        options,
        Box::new(|cc| Ok(Box::new(CadApp::new(cc)))),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum RibbonTab {
    Home,
    Insert,
    Parameters,
    Annotations,
    Simulation,
    View,
    Manage,
    Help,
}

impl RibbonTab {
    const ALL: [Self; 8] = [
        Self::Home,
        Self::Insert,
        Self::Parameters,
        Self::Annotations,
        Self::Simulation,
        Self::View,
        Self::Manage,
        Self::Help,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Insert => "Inserir",
            Self::Parameters => "Parâmetros",
            Self::Annotations => "Anotações",
            Self::Simulation => "Simulação",
            Self::View => "Ver",
            Self::Manage => "Gerenciar",
            Self::Help => "Ajuda",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum ComponentKind {
    Source,
    Breaker,
    Generator,
    Transformer,
    Busbar,
    Load,
}

impl ComponentKind {
    const ALL: [Self; 6] = [
        Self::Source, Self::Breaker, Self::Generator,
        Self::Transformer, Self::Busbar, Self::Load,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Source => "Fonte",
            Self::Breaker => "Disjuntor",
            Self::Generator => "Gerador",
            Self::Transformer => "Transformador",
            Self::Busbar => "Barramento",
            Self::Load => "Carga",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Source => "◉",
            Self::Breaker => "─╱─",
            Self::Generator => "Ⓖ",
            Self::Transformer => ")))‖((",
            Self::Busbar => "┃",
            Self::Load => "▷",
        }
    }

    fn reference_prefix(self) -> &'static str {
        match self {
            Self::Source => "F",
            Self::Breaker => "QF",
            Self::Generator => "G",
            Self::Transformer => "T",
            Self::Busbar => "BB",
            Self::Load => "C",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tool {
    Select,
    Place(ComponentKind),
    Wire,
    Annotation,
}

impl Tool {
    fn label(self) -> &'static str {
        match self {
            Self::Select => "Selecionar",
            Self::Place(kind) => match kind {
                ComponentKind::Source => "Inserir fonte",
                ComponentKind::Breaker => "Inserir disjuntor",
                ComponentKind::Generator => "Inserir gerador",
                ComponentKind::Transformer => "Inserir transformador",
                ComponentKind::Busbar => "Inserir barramento",
                ComponentKind::Load => "Inserir carga",
            },
            Self::Wire => "Condutores",
            Self::Annotation => "Inserir anotação",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Node {
    id: u64,
    kind: ComponentKind,
    x: f32,
    y: f32,
    reference: String,
    /// Observações e dados livres do componente, editáveis pela janela de propriedades.
    #[serde(default)]
    details: String,
    /// Altura do barramento em unidades do desenho; usada somente nesse tipo de elemento.
    #[serde(default = "default_busbar_height")]
    height: f32,
}

fn default_busbar_height() -> f32 { 160.0 }

impl Node {
    fn pos(&self) -> Pos2 {
        Pos2::new(self.x, self.y)
    }

    fn set_pos(&mut self, pos: Pos2) {
        self.x = pos.x;
        self.y = pos.y;
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct Wire {
    from: u64,
    from_port: u8,
    to: u64,
    to_port: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Annotation {
    id: u64,
    x: f32,
    y: f32,
    text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ProjectFile {
    format_version: u32,
    project_name: String,
    nodes: Vec<Node>,
    wires: Vec<Wire>,
    annotations: Vec<Annotation>,
    next_id: u64,
}

#[derive(Clone)]
struct Snapshot {
    nodes: Vec<Node>,
    wires: Vec<Wire>,
    annotations: Vec<Annotation>,
    next_id: u64,
    project_name: String,
}

#[derive(Clone, Copy)]
struct PortRef {
    node_id: u64,
    port: u8, // 0 = terminal esquerdo; 1 = terminal direito
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DragMode {
    None,
    MoveSelection,
    MoveAnnotation,
    Pan,
    BoxSelect,
}

struct CadApp {
    current_tab: RibbonTab,
    tool: Tool,
    nodes: Vec<Node>,
    wires: Vec<Wire>,
    annotations: Vec<Annotation>,
    selected_nodes: Vec<u64>,
    selected_annotation: Option<u64>,
    pending_wire_from: Option<PortRef>,
    next_id: u64,
    show_grid: bool,
    snap_to_grid: bool,
    show_library: bool,
    show_properties: bool,
    zoom: f32,
    pan: Vec2,
    drag_mode: DragMode,
    drag_undo_saved_flag: bool,
    selection_start: Option<Pos2>,
    selection_rect: Option<Rect>,
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
    clipboard_nodes: Vec<Node>,
    clipboard_wires: Vec<Wire>,
    active_file: Option<PathBuf>,
    project_name: String,
    dirty: bool,
    notice: String,
    show_help: bool,
    grid_step: f32,
}

impl CadApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let visuals = dark_visuals();
        let mut style = (*cc.egui_ctx.style()).clone();
        // Importante: definir os visuais dentro do Style impede que os widgets
        // mantenham cores claras herdadas do tema inicial da integração nativa.
        style.visuals = visuals.clone();
        style.spacing.item_spacing = Vec2::new(5.0, 5.0);
        style.spacing.button_padding = Vec2::new(8.0, 5.0);
        style.text_styles.insert(egui::TextStyle::Body, FontId::proportional(13.0));
        style.text_styles.insert(egui::TextStyle::Button, FontId::proportional(12.5));
        cc.egui_ctx.set_style(style);
        cc.egui_ctx.set_visuals(visuals);

        Self {
            current_tab: RibbonTab::Home,
            tool: Tool::Select,
            nodes: Vec::new(),
            wires: Vec::new(),
            annotations: Vec::new(),
            selected_nodes: Vec::new(),
            selected_annotation: None,
            pending_wire_from: None,
            next_id: 1,
            show_grid: true,
            snap_to_grid: true,
            show_library: true,
            show_properties: true,
            zoom: 1.0,
            pan: Vec2::new(35.0, 35.0),
            drag_mode: DragMode::None,
            drag_undo_saved_flag: false,
            selection_start: None,
            selection_rect: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            clipboard_nodes: Vec::new(),
            clipboard_wires: Vec::new(),
            active_file: None,
            project_name: "Simulador sem título".to_owned(),
            dirty: false,
            notice: "Pronto".to_owned(),
            show_help: false,
            grid_step: GRID_STEP,
        }
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            nodes: self.nodes.clone(),
            wires: self.wires.clone(),
            annotations: self.annotations.clone(),
            next_id: self.next_id,
            project_name: self.project_name.clone(),
        }
    }

    fn push_undo(&mut self) {
        self.undo_stack.push(self.snapshot());
        self.redo_stack.clear();
        self.dirty = true;
    }

    fn restore_snapshot(&mut self, snapshot: Snapshot) {
        self.nodes = snapshot.nodes;
        self.wires = snapshot.wires;
        self.annotations = snapshot.annotations;
        self.next_id = snapshot.next_id;
        self.project_name = snapshot.project_name;
        self.selected_nodes.clear();
        self.selected_annotation = None;
        self.pending_wire_from = None;
        self.dirty = true;
    }

    fn undo(&mut self) {
        if let Some(previous) = self.undo_stack.pop() {
            self.redo_stack.push(self.snapshot());
            self.restore_snapshot(previous);
            self.notice = "Ação desfeita".to_owned();
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.snapshot());
            self.restore_snapshot(next);
            self.notice = "Ação refeita".to_owned();
        }
    }

    fn set_tool(&mut self, tool: Tool) {
        self.tool = tool;
        self.pending_wire_from = None;
        self.drag_mode = DragMode::None;
        self.notice = match tool {
            Tool::Select => "Selecione ou arraste elementos; use Ctrl para seleção múltipla".to_owned(),
            Tool::Wire => "Condutores: clique em um terminal e depois no terminal de destino".to_owned(),
            Tool::Annotation => "Clique na área de desenho para posicionar uma anotação".to_owned(),
            Tool::Place(kind) => format!("{}: clique na área de desenho para inserir", kind.label()),
        };
    }

    fn snap_value(&self, value: f32) -> f32 {
        if self.snap_to_grid {
            (value / self.grid_step).round() * self.grid_step
        } else {
            value
        }
    }

    fn world_from_screen(&self, rect: Rect, screen: Pos2) -> Pos2 {
        Pos2::new(
            (screen.x - rect.min.x - self.pan.x) / self.zoom,
            (screen.y - rect.min.y - self.pan.y) / self.zoom,
        )
    }

    fn screen_from_world(&self, rect: Rect, world: Pos2) -> Pos2 {
        Pos2::new(
            rect.min.x + self.pan.x + world.x * self.zoom,
            rect.min.y + self.pan.y + world.y * self.zoom,
        )
    }

    fn node_by_id(&self, id: u64) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    fn port_world(node: &Node, port: u8) -> Pos2 {
        let effective_port = if node.kind == ComponentKind::Source { 1 } else { port };
        let offset = if effective_port == 0 { -PORT_OFFSET } else { PORT_OFFSET };
        if node.kind == ComponentKind::Busbar {
            Pos2::new(node.x + offset, node.y)
        } else {
            Pos2::new(node.x + offset, node.y)
        }
    }

    fn port_screen(&self, rect: Rect, port: PortRef) -> Option<Pos2> {
        let node = self.node_by_id(port.node_id)?;
        Some(self.screen_from_world(rect, Self::port_world(node, port.port)))
    }

    fn port_at_screen(&self, rect: Rect, screen: Pos2) -> Option<PortRef> {
        let exact_limit = 12.0_f32.max(11.0 * self.zoom);
        for node in self.nodes.iter().rev() {
            let ports: &[u8] = if node.kind == ComponentKind::Source { &[1] } else { &[0, 1] };
            for &port in ports {
                let candidate = self.screen_from_world(rect, Self::port_world(node, port));
                if candidate.distance(screen) <= exact_limit {
                    return Some(PortRef { node_id: node.id, port });
                }
            }
        }
        // Facilita a operação: clicar no corpo do símbolo também escolhe o terminal mais próximo.
        for node in self.nodes.iter().rev() {
            let center = self.screen_from_world(rect, node.pos());
            if center.distance(screen) <= (NODE_RADIUS * self.zoom).max(18.0) {
                return Some(PortRef {
                    node_id: node.id,
                    port: if screen.x < center.x { 0 } else { 1 },
                });
            }
        }
        None
    }

    fn hit_node_screen(&self, rect: Rect, screen: Pos2) -> Option<u64> {
        self.nodes
            .iter()
            .rev()
            .find(|node| self.screen_from_world(rect, node.pos()).distance(screen) <= (26.0 * self.zoom).max(18.0))
            .map(|node| node.id)
    }

    fn add_node(&mut self, kind: ComponentKind, world: Pos2) {
        self.push_undo();
        let count = self.nodes.iter().filter(|n| n.kind == kind).count() + 1;
        let node = Node {
            id: self.next_id,
            kind,
            x: self.snap_value(world.x),
            y: self.snap_value(world.y),
            reference: format!("{}{}", kind.reference_prefix(), count),
            details: String::new(),
            height: default_busbar_height(),
        };
        self.next_id += 1;
        self.nodes.push(node);
        self.selected_nodes = vec![self.next_id - 1];
        self.selected_annotation = None;
        self.notice = format!("{} inserido", kind.label());
    }

    fn add_annotation(&mut self, world: Pos2) {
        self.push_undo();
        let id = self.next_id;
        self.next_id += 1;
        self.annotations.push(Annotation {
            id,
            x: self.snap_value(world.x),
            y: self.snap_value(world.y),
            text: "Anotação".to_owned(),
        });
        self.selected_nodes.clear();
        self.selected_annotation = Some(id);
        self.notice = "Anotação adicionada; edite o texto no painel Propriedades".to_owned();
    }

    fn connect_ports(&mut self, from: PortRef, to: PortRef) {
        if from.node_id == to.node_id {
            self.pending_wire_from = None;
            self.notice = "Escolha dois componentes diferentes para conectar".to_owned();
            return;
        }
        let exists = self.wires.iter().any(|w| {
            (w.from == from.node_id && w.from_port == from.port && w.to == to.node_id && w.to_port == to.port)
                || (w.from == to.node_id && w.from_port == to.port && w.to == from.node_id && w.to_port == from.port)
        });
        if !exists {
            self.push_undo();
            self.wires.push(Wire {
                from: from.node_id,
                from_port: from.port,
                to: to.node_id,
                to_port: to.port,
            });
            self.notice = "Condutor conectado".to_owned();
        } else {
            self.notice = "Essa ligação já existe".to_owned();
        }
        self.pending_wire_from = None;
    }

    fn delete_selection(&mut self) {
        let selected = self.selected_nodes.clone();
        let selected_annotation = self.selected_annotation;
        if selected.is_empty() && selected_annotation.is_none() {
            self.notice = "Nenhum elemento selecionado".to_owned();
            return;
        }
        self.push_undo();
        self.nodes.retain(|node| !selected.contains(&node.id));
        self.wires.retain(|wire| !selected.contains(&wire.from) && !selected.contains(&wire.to));
        if let Some(id) = selected_annotation {
            self.annotations.retain(|annotation| annotation.id != id);
        }
        self.selected_nodes.clear();
        self.selected_annotation = None;
        self.notice = "Seleção excluída".to_owned();
    }

    fn new_project(&mut self) {
        self.push_undo();
        self.nodes.clear();
        self.wires.clear();
        self.annotations.clear();
        self.selected_nodes.clear();
        self.selected_annotation = None;
        self.pending_wire_from = None;
        self.next_id = 1;
        self.project_name = "Simulador sem título".to_owned();
        self.active_file = None;
        self.dirty = true;
        self.notice = "Novo projeto criado".to_owned();
    }

    fn project_data(&self) -> ProjectFile {
        ProjectFile {
            format_version: 1,
            project_name: self.project_name.clone(),
            nodes: self.nodes.clone(),
            wires: self.wires.clone(),
            annotations: self.annotations.clone(),
            next_id: self.next_id,
        }
    }

    fn save_project(&mut self, choose_path: bool) {
        let path = if !choose_path {
            self.active_file.clone()
        } else {
            None
        };
        let destination = path.or_else(|| {
            let filename = format!("{}.cad.json", sanitize_filename(&self.project_name));
            rfd::FileDialog::new()
                .add_filter("Projeto Simulador", &["json"])
                .set_file_name(&filename)
                .save_file()
        });
        let Some(destination) = destination else {
            self.notice = "Salvamento cancelado".to_owned();
            return;
        };
        let saved_name = destination.file_stem().and_then(|s| s.to_str())
            .map(|stem| stem.trim_end_matches(".cad").to_owned())
            .unwrap_or_else(|| self.project_name.clone());
        let mut project = self.project_data();
        project.project_name = saved_name.clone();
        match serde_json::to_string_pretty(&project) {
            Ok(json) => match fs::write(&destination, json) {
                Ok(()) => {
                    self.active_file = Some(destination.clone());
                    self.project_name = saved_name;
                    self.dirty = false;
                    self.notice = format!("Projeto salvo: {}", destination.display());
                }
                Err(error) => self.notice = format!("Falha ao salvar arquivo: {error}"),
            },
            Err(error) => self.notice = format!("Falha ao preparar o arquivo: {error}"),
        }
    }

    fn open_project(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Projeto Simulador", &["json"])
            .pick_file()
        else {
            self.notice = "Abertura cancelada".to_owned();
            return;
        };
        match fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<ProjectFile>(&contents) {
                Ok(project) if project.format_version == 1 => {
                    self.push_undo();
                    self.nodes = project.nodes;
                    self.wires = project.wires;
                    self.annotations = project.annotations;
                    self.next_id = project.next_id.max(
                        self.nodes.iter().map(|n| n.id).max().unwrap_or(0).saturating_add(1),
                    );
                    self.project_name = project.project_name;
                    self.active_file = Some(path.clone());
                    self.selected_nodes.clear();
                    self.selected_annotation = None;
                    self.pending_wire_from = None;
                    self.dirty = false;
                    self.zoom = 1.0;
                    self.pan = Vec2::new(35.0, 35.0);
                    self.notice = format!("Projeto aberto: {}", path.display());
                }
                Ok(_) => self.notice = "Versão de arquivo não suportada".to_owned(),
                Err(error) => self.notice = format!("Arquivo de projeto inválido: {error}"),
            },
            Err(error) => self.notice = format!("Não foi possível ler o arquivo: {error}"),
        }
    }

    fn copy_selection(&mut self) {
        if self.selected_nodes.is_empty() {
            self.notice = "Selecione um ou mais componentes para copiar".to_owned();
            return;
        }
        self.clipboard_nodes = self.nodes.iter().filter(|n| self.selected_nodes.contains(&n.id)).cloned().collect();
        let ids: Vec<u64> = self.clipboard_nodes.iter().map(|n| n.id).collect();
        self.clipboard_wires = self.wires.iter().copied().filter(|w| ids.contains(&w.from) && ids.contains(&w.to)).collect();
        self.notice = format!("{} componente(s) copiado(s)", self.clipboard_nodes.len());
    }

    fn paste_selection(&mut self) {
        if self.clipboard_nodes.is_empty() {
            self.notice = "A área de transferência do CAD está vazia".to_owned();
            return;
        }
        self.push_undo();
        let mut map = std::collections::HashMap::new();
        let mut pasted = Vec::new();
        for old in &self.clipboard_nodes {
            let mut node = old.clone();
            let new_id = self.next_id;
            self.next_id += 1;
            map.insert(old.id, new_id);
            node.id = new_id;
            node.x = self.snap_value(node.x + self.grid_step * 2.0);
            node.y = self.snap_value(node.y + self.grid_step * 2.0);
            node.reference = format!("{}{}", node.kind.reference_prefix(), new_id);
            pasted.push(node);
        }
        self.selected_nodes.clear();
        for node in &pasted {
            self.selected_nodes.push(node.id);
        }
        self.nodes.extend(pasted);
        for old_wire in &self.clipboard_wires {
            if let (Some(from), Some(to)) = (map.get(&old_wire.from), map.get(&old_wire.to)) {
                self.wires.push(Wire { from: *from, from_port: old_wire.from_port, to: *to, to_port: old_wire.to_port });
            }
        }
        self.selected_annotation = None;
        self.notice = "Seleção colada".to_owned();
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let (command, ctrl_z, ctrl_y, ctrl_c, ctrl_v, ctrl_s, ctrl_o, ctrl_n, delete, escape) = ctx.input(|i| {
            let command = i.modifiers.command;
            (
                command,
                command && i.key_pressed(egui::Key::Z),
                command && i.key_pressed(egui::Key::Y),
                command && i.key_pressed(egui::Key::C),
                command && i.key_pressed(egui::Key::V),
                command && i.key_pressed(egui::Key::S),
                command && i.key_pressed(egui::Key::O),
                command && i.key_pressed(egui::Key::N),
                i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace),
                i.key_pressed(egui::Key::Escape),
            )
        });
        let _ = command;
        if ctrl_z { self.undo(); }
        if ctrl_y { self.redo(); }
        if ctrl_c { self.copy_selection(); }
        if ctrl_v { self.paste_selection(); }
        if ctrl_s { self.save_project(false); }
        if ctrl_o { self.open_project(); }
        if ctrl_n { self.new_project(); }
        if delete { self.delete_selection(); }
        if escape { self.set_tool(Tool::Select); }
    }

    fn draw_tabs(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none().fill(RIBBON_DARK).inner_margin(egui::Margin::symmetric(10.0, 3.0)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("⚡").size(20.0).color(ACCENT));
                ui.label(RichText::new("SIMULADOR").strong().size(13.0).color(TEXT));
                ui.separator();
                for tab in RibbonTab::ALL {
                    let active = self.current_tab == tab;
                    let button = egui::Button::new(RichText::new(tab.label()).size(13.0).color(if active { Color32::WHITE } else { TEXT }))
                        .fill(if active { RIBBON } else { RIBBON_DARK })
                        .stroke(if active { Stroke::new(1.0_f32, Color32::from_rgb(75, 86, 98)) } else { Stroke::NONE });
                    if ui.add_sized([92.0, 29.0], button).clicked() {
                        self.current_tab = tab;
                        if tab == RibbonTab::Help { self.show_help = true; }
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let dirty = if self.dirty { " ●" } else { "" };
                    ui.label(RichText::new(format!("{}{}", self.project_name, dirty)).size(12.0).color(if self.dirty { PENDING_COLOR } else { MUTED }));
                });
            });
        });
    }

    fn ribbon_button(ui: &mut egui::Ui, icon: &str, label: &str, active: bool, enabled: bool) -> bool {
        let text = format!("{icon}\n{label}");
        let fill = if active { BUTTON_ACTIVE } else { BUTTON };
        let response = ui.add_enabled(enabled, egui::Button::new(RichText::new(text).size(12.0).color(TEXT))
            .fill(fill).stroke(if active { Stroke::new(1.0_f32, ACCENT) } else { Stroke::NONE }));
        response.on_hover_text(label).clicked()
    }

    fn group_label(ui: &mut egui::Ui, label: &str) {
        ui.add_space(2.0);
        ui.separator();
        ui.vertical(|ui| {
            ui.add_space(37.0);
            ui.label(RichText::new(label).size(10.0).color(MUTED));
        });
    }

    fn draw_toolbar(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none().fill(RIBBON).inner_margin(egui::Margin::symmetric(10.0, 6.0)).show(ui, |ui| {
            ui.set_min_height(72.0);
            ui.horizontal(|ui| match self.current_tab {
                RibbonTab::Home => {
                    if Self::ribbon_button(ui, "↖", "Selecionar", self.tool == Tool::Select, true) { self.set_tool(Tool::Select); }
                    if Self::ribbon_button(ui, "↶", "Desfazer", false, !self.undo_stack.is_empty()) { self.undo(); }
                    if Self::ribbon_button(ui, "↷", "Refazer", false, !self.redo_stack.is_empty()) { self.redo(); }
                    if Self::ribbon_button(ui, "▣", "Copiar", false, !self.selected_nodes.is_empty()) { self.copy_selection(); }
                    if Self::ribbon_button(ui, "▢", "Colar", false, !self.clipboard_nodes.is_empty()) { self.paste_selection(); }
                    if Self::ribbon_button(ui, "⌫", "Excluir", false, !self.selected_nodes.is_empty() || self.selected_annotation.is_some()) { self.delete_selection(); }
                    Self::group_label(ui, "Área de transferência / Edição");
                }
                RibbonTab::Insert => {
                    for kind in ComponentKind::ALL {
                        if Self::ribbon_button(ui, kind.icon(), kind.label(), self.tool == Tool::Place(kind), true) { self.set_tool(Tool::Place(kind)); }
                    }
                    if Self::ribbon_button(ui, "╱", "Condutores", self.tool == Tool::Wire, true) { self.set_tool(Tool::Wire); }
                    if Self::ribbon_button(ui, "T", "Anotação", self.tool == Tool::Annotation, true) { self.set_tool(Tool::Annotation); }
                    Self::group_label(ui, "Elementos elétricos");
                }
                RibbonTab::Simulation => {
                    // Controles visuais de simulação; o motor de cálculo será integrado posteriormente.
                    if Self::ribbon_button(ui, "▶", "Play", false, true) { self.notice = "Simulação iniciada (modo demonstrativo)".to_owned(); }
                    if Self::ribbon_button(ui, "Ⅱ", "Pause", false, true) { self.notice = "Simulação pausada".to_owned(); }
                    if Self::ribbon_button(ui, "■", "Stop", false, true) { self.notice = "Simulação parada".to_owned(); }
                    Self::group_label(ui, "Controle da simulação");
                }
                RibbonTab::Parameters => {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Desenho").strong().color(TEXT));
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.snap_to_grid, "Encaixe na grade");
                            ui.label("Passo");
                            ui.add(egui::DragValue::new(&mut self.grid_step).speed(1.0).range(5.0..=100.0));
                        });
                        ui.label(RichText::new("Parâmetros elétricos serão adicionados em uma etapa futura.").size(11.0).color(MUTED));
                    });
                }
                RibbonTab::Annotations => {
                    if Self::ribbon_button(ui, "T", "Inserir texto", self.tool == Tool::Annotation, true) { self.set_tool(Tool::Annotation); }
                    if Self::ribbon_button(ui, "↖", "Selecionar", self.tool == Tool::Select, true) { self.set_tool(Tool::Select); }
                    ui.label(RichText::new("Selecione uma anotação para editar seu conteúdo no painel Propriedades.").color(MUTED).size(11.0));
                }
                RibbonTab::View => {
                    ui.checkbox(&mut self.show_grid, "Grade");
                    ui.checkbox(&mut self.snap_to_grid, "Encaixe");
                    ui.separator();
                    if ui.button("＋ Zoom").clicked() { self.zoom = (self.zoom * 1.15).min(3.0); }
                    if ui.button("－ Zoom").clicked() { self.zoom = (self.zoom / 1.15).max(0.25); }
                    if ui.button("1:1 / Enquadrar").clicked() { self.zoom = 1.0; self.pan = Vec2::new(35.0, 35.0); }
                    ui.separator();
                    ui.checkbox(&mut self.show_library, "Biblioteca");
                    ui.checkbox(&mut self.show_properties, "Propriedades");
                }
                RibbonTab::Manage => {
                    if Self::ribbon_button(ui, "▧", "Novo", false, true) { self.new_project(); }
                    if Self::ribbon_button(ui, "▱", "Abrir", false, true) { self.open_project(); }
                    if Self::ribbon_button(ui, "▣", "Salvar", false, true) { self.save_project(false); }
                    if Self::ribbon_button(ui, "▣+", "Salvar como", false, true) { self.save_project(true); }
                    if Self::ribbon_button(ui, "×", "Limpar desenho", false, !self.nodes.is_empty() || !self.annotations.is_empty()) {
                        self.push_undo(); self.nodes.clear(); self.wires.clear(); self.annotations.clear(); self.selected_nodes.clear(); self.selected_annotation = None; self.pending_wire_from = None; self.notice = "Desenho limpo".to_owned();
                    }
                    Self::group_label(ui, "Projeto");
                }
                RibbonTab::Help => {
                    if Self::ribbon_button(ui, "?", "Como usar", false, true) { self.show_help = true; }
                    ui.label(RichText::new("Atalhos: Ctrl+S salvar · Ctrl+O abrir · Ctrl+Z desfazer · Espaço+arrastar mover a tela").size(11.0).color(MUTED));
                }
            });
        });
    }

    fn draw_library(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("BIBLIOTECA").strong().size(12.0).color(TEXT));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(RichText::new("▦").color(MUTED)); });
        });
        ui.separator();
        ui.label(RichText::new("FERRAMENTAS").size(10.0).color(MUTED));
        if ui.selectable_label(self.tool == Tool::Select, "↖  Selecionar").clicked() { self.set_tool(Tool::Select); }
        if ui.selectable_label(self.tool == Tool::Wire, "╱  Condutores").clicked() { self.set_tool(Tool::Wire); }
        if ui.selectable_label(self.tool == Tool::Annotation, "T  Anotação").clicked() { self.set_tool(Tool::Annotation); }
        ui.add_space(9.0);
        ui.label(RichText::new("COMPONENTES ELÉTRICOS").size(10.0).color(MUTED));
        for kind in ComponentKind::ALL {
            let text = format!("{}   {}", kind.icon(), kind.label());
            if ui.selectable_label(self.tool == Tool::Place(kind), text).clicked() { self.set_tool(Tool::Place(kind)); }
        }
        ui.add_space(12.0);
        ui.separator();
        ui.label(RichText::new("DICAS").size(10.0).color(MUTED));
        ui.label(RichText::new("• Clique para inserir\n• Arraste para mover\n• Ctrl+clique: múltipla seleção\n• Espaço+arrastar: mover tela\n• Role o mouse: zoom").size(11.0).color(MUTED));
        ui.add_space(10.0);
        ui.separator();
        ui.label(RichText::new(format!("{} componentes", self.nodes.len())).color(TEXT));
        ui.label(RichText::new(format!("{} condutores", self.wires.len())).color(TEXT));
        ui.label(RichText::new(format!("{} anotações", self.annotations.len())).color(TEXT));
    }

    fn draw_properties(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.label(RichText::new("PROPRIEDADES").strong().size(12.0).color(TEXT));
        ui.separator();
        if let Some(id) = self.selected_annotation {
            if let Some(index) = self.annotations.iter().position(|a| a.id == id) {
                ui.label(RichText::new("Anotação").strong().color(ACCENT));
                ui.label(RichText::new(format!("ID: {id}")).size(11.0).color(MUTED));
                ui.label("Texto");
                let response = ui.text_edit_multiline(&mut self.annotations[index].text);
                if response.changed() { self.dirty = true; }
                ui.horizontal(|ui| {
                    ui.label("X"); ui.label(format!("{:.1}", self.annotations[index].x));
                    ui.label("Y"); ui.label(format!("{:.1}", self.annotations[index].y));
                });
                if ui.button("Excluir anotação").clicked() { self.delete_selection(); }
            }
        } else if self.selected_nodes.len() == 1 {
            let id = self.selected_nodes[0];
            if let Some(index) = self.nodes.iter().position(|n| n.id == id) {
                let mut changed_position = false;
                ui.label(RichText::new(self.nodes[index].kind.label()).strong().color(ACCENT));
                ui.label(RichText::new(format!("ID interno: {}", self.nodes[index].id)).size(11.0).color(MUTED));
                ui.add_space(4.0);
                ui.label("Referência / identificação");
                if ui.text_edit_singleline(&mut self.nodes[index].reference).changed() { self.dirty = true; }
                ui.add_space(4.0);
                ui.label("Posição no desenho");
                ui.horizontal(|ui| {
                    ui.label("X");
                    changed_position |= ui.add(egui::DragValue::new(&mut self.nodes[index].x).speed(1.0).range(-10000.0..=10000.0)).changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Y");
                    changed_position |= ui.add(egui::DragValue::new(&mut self.nodes[index].y).speed(1.0).range(-10000.0..=10000.0)).changed();
                });
                if changed_position { self.dirty = true; }
                ui.add_space(8.0);
                ui.label(RichText::new("Terminais").strong().color(TEXT));
                ui.label(RichText::new("1 — Esquerdo\n2 — Direito").size(11.0).color(MUTED));
                if ui.button("Excluir componente").clicked() { self.delete_selection(); }
                ui.add_space(8.0);
                ui.label(RichText::new("Os parâmetros elétricos ainda não são solicitados nesta versão.").size(11.0).color(MUTED));
            }
        } else if self.selected_nodes.len() > 1 {
            ui.label(RichText::new("Seleção múltipla").strong().color(ACCENT));
            ui.label(format!("{} componentes selecionados", self.selected_nodes.len()));
            if ui.button("Copiar seleção").clicked() { self.copy_selection(); }
            if ui.button("Excluir seleção").clicked() { self.delete_selection(); }
        } else {
            ui.label(RichText::new("Nenhum elemento selecionado").color(MUTED));
            ui.add_space(8.0);
            ui.label(RichText::new("PROJETO").strong().color(TEXT));
            ui.label(format!("Nome: {}", self.project_name));
            ui.label(format!("Componentes: {}", self.nodes.len()));
            ui.label(format!("Condutores: {}", self.wires.len()));
            ui.label(format!("Anotações: {}", self.annotations.len()));
            ui.add_space(8.0);
            if ui.button("Salvar projeto").clicked() { self.save_project(false); }
            if ui.button("Abrir projeto").clicked() { self.open_project(); }
        }
    }

    fn draw_statusbar(&self, ui: &mut egui::Ui) {
        egui::Frame::none().fill(RIBBON_DARK).inner_margin(egui::Margin::symmetric(9.0, 4.0)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{}  |  {}", self.tool.label(), self.notice)).size(11.0).color(MUTED));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("Zoom: {:.0}%   |   Encaixe: {}   |   Grade: {}   |   Componentes: {}   |   Condutores: {}", self.zoom * 100.0, if self.snap_to_grid { "ativo" } else { "livre" }, if self.show_grid { "visível" } else { "oculta" }, self.nodes.len(), self.wires.len())).size(11.0).color(TEXT));
                });
            });
        });
    }

    fn node_screen_pos(&self, rect: Rect, node: &Node) -> Pos2 {
        self.screen_from_world(rect, node.pos())
    }

    fn wire_path(&self, rect: Rect, wire: Wire) -> Option<[Pos2; 6]> {
        let from = self.node_by_id(wire.from)?;
        let to = self.node_by_id(wire.to)?;
        let start = self.screen_from_world(rect, Self::port_world(from, wire.from_port));
        let end = self.screen_from_world(rect, Self::port_world(to, wire.to_port));
        let stub = 14.0 * self.zoom.max(0.5);
        let start_stub = start + Vec2::new(if wire.from_port == 1 { stub } else { -stub }, 0.0);
        let end_stub = end + Vec2::new(if wire.to_port == 1 { stub } else { -stub }, 0.0);
        let mid_x = (start_stub.x + end_stub.x) * 0.5;
        Some([start, start_stub, Pos2::new(mid_x, start_stub.y), Pos2::new(mid_x, end_stub.y), end_stub, end])
    }

    fn paint_canvas(&self, painter: egui::Painter, rect: Rect, ctx: &egui::Context) {
        painter.rect_filled(rect, 0.0, CANVAS);
        if self.show_grid {
            let world_left = -self.pan.x / self.zoom;
            let world_top = -self.pan.y / self.zoom;
            let start_x = (world_left / self.grid_step).floor() * self.grid_step;
            let start_y = (world_top / self.grid_step).floor() * self.grid_step;
            let mut xw = start_x;
            let mut index = 0;
            while xw * self.zoom + self.pan.x <= rect.width() && index < 2000 {
                let x = rect.min.x + self.pan.x + xw * self.zoom;
                if x >= rect.left() && x <= rect.right() {
                    let major = ((xw / self.grid_step).round() as i32).rem_euclid(5) == 0;
                    painter.line_segment([Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())], Stroke::new(1.0_f32, if major { GRID_MAJOR } else { GRID }));
                }
                xw += self.grid_step;
                index += 1;
            }
            let mut yw = start_y;
            let mut index = 0;
            while yw * self.zoom + self.pan.y <= rect.height() && index < 2000 {
                let y = rect.min.y + self.pan.y + yw * self.zoom;
                if y >= rect.top() && y <= rect.bottom() {
                    let major = ((yw / self.grid_step).round() as i32).rem_euclid(5) == 0;
                    painter.line_segment([Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)], Stroke::new(1.0_f32, if major { GRID_MAJOR } else { GRID }));
                }
                yw += self.grid_step;
                index += 1;
            }
        }

        // Condutores atrás dos símbolos e sempre presos aos terminais dos componentes.
        for wire in &self.wires {
            if let Some(path) = self.wire_path(rect, *wire) {
                for pair in path.windows(2) {
                    painter.line_segment([pair[0], pair[1]], Stroke::new(2.0_f32.max(self.zoom), WIRE_COLOR));
                }
            }
        }
        if let Some(from) = self.pending_wire_from {
            if let Some(start) = self.port_screen(rect, from) {
                if let Some(pointer) = ctx.pointer_hover_pos().filter(|p| rect.contains(*p)) {
                    let middle = Pos2::new((start.x + pointer.x) * 0.5, start.y);
                    painter.line_segment([start, Pos2::new(start.x + if from.port == 1 { 14.0 } else { -14.0 }, start.y)], Stroke::new(2.0_f32, PENDING_COLOR));
                    painter.line_segment([Pos2::new(start.x + if from.port == 1 { 14.0 } else { -14.0 }, start.y), middle], Stroke::new(2.0_f32, PENDING_COLOR));
                    painter.line_segment([middle, Pos2::new(middle.x, pointer.y)], Stroke::new(2.0_f32, PENDING_COLOR));
                    painter.line_segment([Pos2::new(middle.x, pointer.y), pointer], Stroke::new(2.0_f32, PENDING_COLOR));
                }
                painter.circle_filled(start, 5.0, PENDING_COLOR);
            }
        }

        for node in &self.nodes {
            let pos = self.node_screen_pos(rect, node);
            let selected = self.selected_nodes.contains(&node.id);
            let symbol_color = if selected { ACCENT } else { TEXT };
            let scale = self.zoom;
            let stroke = Stroke::new(if selected { 2.4_f32 } else { 1.8_f32 } * scale.max(0.65), symbol_color);
            if selected { painter.circle_stroke(pos, (26.0 * scale).max(20.0), Stroke::new(1.0_f32, ACCENT)); }
            match node.kind {
                ComponentKind::Source => {
                    painter.line_segment([pos + Vec2::new(-NODE_RADIUS, 0.0) * scale, pos + Vec2::new(-PORT_OFFSET, 0.0) * scale], stroke);
                    painter.line_segment([pos + Vec2::new(NODE_RADIUS, 0.0) * scale, pos + Vec2::new(PORT_OFFSET, 0.0) * scale], stroke);
                    painter.circle_stroke(pos, NODE_RADIUS * scale, stroke);
                    let mut wave = Vec::with_capacity(17);
                    for i in 0..=16 {
                        let f = i as f32 / 16.0;
                        wave.push(Pos2::new(pos.x - 10.0 * scale + f * 20.0 * scale, pos.y - (f * std::f32::consts::TAU).sin() * 5.0 * scale));
                    }
                    painter.add(egui::Shape::line(wave, Stroke::new(1.7_f32 * scale.max(0.7), symbol_color)));
                }
                ComponentKind::Breaker => {
                    painter.line_segment([pos + Vec2::new(-PORT_OFFSET, 0.0) * scale, pos + Vec2::new(-25.0, 0.0) * scale], stroke);
                    painter.line_segment([pos + Vec2::new(25.0, 0.0) * scale, pos + Vec2::new(PORT_OFFSET, 0.0) * scale], stroke);
                    painter.line_segment([pos + Vec2::new(-25.0, 0.0) * scale, pos + Vec2::new(-9.0, 0.0) * scale], stroke);
                    painter.line_segment([pos + Vec2::new(9.0, 0.0) * scale, pos + Vec2::new(25.0, 0.0) * scale], stroke);
                    painter.circle_filled(pos + Vec2::new(-9.0, 0.0) * scale, 2.2 * scale.max(0.65), symbol_color);
                    painter.circle_filled(pos + Vec2::new(9.0, 0.0) * scale, 2.2 * scale.max(0.65), symbol_color);
                    painter.line_segment([pos + Vec2::new(-7.0, 1.0) * scale, pos + Vec2::new(8.0, -10.0) * scale], stroke);
                }
                ComponentKind::Generator => {
                    painter.line_segment([pos + Vec2::new(-NODE_RADIUS, 0.0) * scale, pos + Vec2::new(-PORT_OFFSET, 0.0) * scale], stroke);
                    painter.line_segment([pos + Vec2::new(NODE_RADIUS, 0.0) * scale, pos + Vec2::new(PORT_OFFSET, 0.0) * scale], stroke);
                    painter.circle_stroke(pos, NODE_RADIUS * scale, stroke);
                    painter.text(pos, Align2::CENTER_CENTER, "G", FontId::proportional(16.0 * scale.max(0.75)), symbol_color);
                }
                ComponentKind::Transformer => {
                    painter.line_segment([pos + Vec2::new(-PORT_OFFSET, 0.0) * scale, pos + Vec2::new(-17.0, 0.0) * scale], stroke);
                    painter.line_segment([pos + Vec2::new(17.0, 0.0) * scale, pos + Vec2::new(PORT_OFFSET, 0.0) * scale], stroke);
                    for side in [-1.0_f32, 1.0_f32] {
                        let cx = pos.x + side * 9.0 * scale;
                        for coil in 0..3 {
                            let cy = pos.y + (coil as f32 - 1.0) * 8.0 * scale;
                            let direction = if side < 0.0 { -1.0 } else { 1.0 };
                            painter.circle_stroke(Pos2::new(cx, cy), 5.5 * scale, stroke);
                            painter.line_segment([Pos2::new(cx, cy - 5.5 * scale), Pos2::new(cx, cy + 5.5 * scale)], Stroke::new(1.0_f32, CANVAS));
                            let _ = direction;
                        }
                    }
                    painter.line_segment([pos + Vec2::new(0.0, -18.0) * scale, pos + Vec2::new(0.0, 18.0) * scale], stroke);
                }
                ComponentKind::Busbar => {
                    let half = node.height.clamp(40.0, 1200.0) * 0.5 * scale;
                    painter.line_segment([Pos2::new(pos.x, pos.y - half), Pos2::new(pos.x, pos.y + half)], Stroke::new(6.0 * scale.max(0.65), symbol_color));
                }
                ComponentKind::Load => {
                    painter.line_segment([pos + Vec2::new(-PORT_OFFSET, 0.0) * scale, pos + Vec2::new(-13.0, 0.0) * scale], stroke);
                    let points = vec![pos + Vec2::new(-13.0, -13.0) * scale, pos + Vec2::new(13.0, 0.0) * scale, pos + Vec2::new(-13.0, 13.0) * scale];
                    painter.add(egui::Shape::convex_polygon(points, symbol_color, stroke));
                    painter.line_segment([pos + Vec2::new(13.0, 0.0) * scale, pos + Vec2::new(PORT_OFFSET, 0.0) * scale], stroke);
                }
            }
            let label = format!("{} · {}", node.reference, node.kind.label());
            painter.text(pos + Vec2::new(0.0, 24.0 * scale.max(0.75)), Align2::CENTER_TOP, label, FontId::proportional(11.5 * scale.max(0.75)), if selected { Color32::WHITE } else { TEXT });
            for port in [0_u8, 1_u8] {
                let port_pos = self.screen_from_world(rect, Self::port_world(node, port));
                painter.circle_filled(port_pos, (3.5 * scale).max(2.5), TERMINAL);
                painter.circle_stroke(port_pos, (4.8 * scale).max(3.5), Stroke::new(1.0_f32, CANVAS));
            }
        }

        for annotation in &self.annotations {
            let pos = self.screen_from_world(rect, Pos2::new(annotation.x, annotation.y));
            let selected = self.selected_annotation == Some(annotation.id);
            painter.text(pos, Align2::LEFT_TOP, &annotation.text, FontId::proportional(13.0 * self.zoom.max(0.8)), if selected { ACCENT } else { Color32::from_rgb(239, 209, 120) });
            if selected { painter.line_segment([pos, pos + Vec2::new(100.0, 0.0)], Stroke::new(1.0_f32, ACCENT)); }
        }

        if let Some(selection) = self.selection_rect {
            painter.rect_filled(selection, 0.0, Color32::from_rgba_premultiplied(65, 145, 205, 35));
            let tl = selection.left_top(); let tr = selection.right_top(); let bl = selection.left_bottom(); let br = selection.right_bottom();
            let stroke = Stroke::new(1.0_f32, ACCENT);
            painter.line_segment([tl, tr], stroke); painter.line_segment([tr, br], stroke); painter.line_segment([br, bl], stroke); painter.line_segment([bl, tl], stroke);
        }
    }

    fn handle_canvas_input(&mut self, ctx: &egui::Context, rect: Rect, response: &egui::Response) {
        let space_down = ctx.input(|i| i.key_down(egui::Key::Space));
        if response.hovered() {
            let scroll = ctx.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.1 {
                if let Some(pointer) = ctx.pointer_hover_pos() {
                    let before = self.world_from_screen(rect, pointer);
                    let factor = (1.0 + scroll * 0.0015).clamp(0.85, 1.15);
                    self.zoom = (self.zoom * factor).clamp(0.25, 3.0);
                    self.pan = Vec2::new(pointer.x - rect.min.x - before.x * self.zoom, pointer.y - rect.min.y - before.y * self.zoom);
                }
            }
        }

        if response.drag_started() {
            if let Some(pointer) = response.interact_pointer_pos() {
                if space_down {
                    self.drag_mode = DragMode::Pan;
                } else if self.tool == Tool::Select {
                    if let Some(id) = self.hit_node_screen(rect, pointer) {
                        let command = ctx.input(|i| i.modifiers.command || i.modifiers.shift);
                        if !self.selected_nodes.contains(&id) {
                            if !command { self.selected_nodes.clear(); }
                            self.selected_nodes.push(id);
                        }
                        self.selected_annotation = None;
                        self.drag_mode = DragMode::MoveSelection;
                    } else if let Some(id) = self.hit_annotation_screen(rect, pointer) {
                        self.selected_nodes.clear();
                        self.selected_annotation = Some(id);
                        self.drag_mode = DragMode::MoveAnnotation;
                    } else {
                        self.drag_mode = DragMode::BoxSelect;
                        self.selection_start = Some(pointer);
                        if !ctx.input(|i| i.modifiers.command || i.modifiers.shift) { self.selected_nodes.clear(); self.selected_annotation = None; }
                    }
                }
            }
        }

        if response.dragged() {
            let delta = ctx.input(|i| i.pointer.delta());
            match self.drag_mode {
                DragMode::Pan => { self.pan += delta; }
                DragMode::MoveSelection => {
                    if delta.length_sq() > 0.0 {
                        if !self.drag_undo_saved() { self.push_undo(); self.mark_drag_undo_saved(true); }
                        let step = delta / self.zoom;
                        let selected = self.selected_nodes.clone();
                        for node in &mut self.nodes {
                            if selected.contains(&node.id) {
                                node.x += step.x;
                                node.y += step.y;
                                if self.snap_to_grid {
                                    node.x = (node.x / self.grid_step).round() * self.grid_step;
                                    node.y = (node.y / self.grid_step).round() * self.grid_step;
                                }
                            }
                        }
                        self.dirty = true;
                    }
                }
                DragMode::MoveAnnotation => {
                    if delta.length_sq() > 0.0 {
                        if !self.drag_undo_saved() { self.push_undo(); self.mark_drag_undo_saved(true); }
                        if let Some(id) = self.selected_annotation {
                            if let Some(annotation) = self.annotations.iter_mut().find(|a| a.id == id) {
                                annotation.x += delta.x / self.zoom;
                                annotation.y += delta.y / self.zoom;
                                if self.snap_to_grid {
                                    annotation.x = (annotation.x / self.grid_step).round() * self.grid_step;
                                    annotation.y = (annotation.y / self.grid_step).round() * self.grid_step;
                                }
                                self.dirty = true;
                            }
                        }
                    }
                }
                DragMode::BoxSelect => {
                    if let (Some(start), Some(pointer)) = (self.selection_start, response.interact_pointer_pos()) {
                        self.selection_rect = Some(Rect::from_two_pos(start, pointer));
                    }
                }
                DragMode::None => {}
            }
        }

        if response.drag_stopped() {
            if self.drag_mode == DragMode::BoxSelect {
                if let Some(selection) = self.selection_rect {
                    let command = ctx.input(|i| i.modifiers.command || i.modifiers.shift);
                    if !command { self.selected_nodes.clear(); }
                    for node in &self.nodes {
                        let screen = self.node_screen_pos(rect, node);
                        if selection.contains(screen) && !self.selected_nodes.contains(&node.id) { self.selected_nodes.push(node.id); }
                    }
                    self.selected_annotation = None;
                }
            }
            self.drag_mode = DragMode::None;
            self.selection_start = None;
            self.selection_rect = None;
            self.reset_drag_undo_saved();
        }

        if !response.clicked() { return; }
        let Some(pointer) = response.interact_pointer_pos() else { return; };
        let world = self.world_from_screen(rect, pointer);
        match self.tool {
            Tool::Select => {
                if let Some(id) = self.hit_node_screen(rect, pointer) {
                    let command = ctx.input(|i| i.modifiers.command || i.modifiers.shift);
                    if command {
                        if self.selected_nodes.contains(&id) { self.selected_nodes.retain(|existing| *existing != id); }
                        else { self.selected_nodes.push(id); }
                    } else if !self.selected_nodes.contains(&id) || self.selected_nodes.len() != 1 {
                        self.selected_nodes = vec![id];
                    }
                    self.selected_annotation = None;
                } else if let Some(id) = self.hit_annotation_screen(rect, pointer) {
                    self.selected_nodes.clear();
                    self.selected_annotation = Some(id);
                } else if !ctx.input(|i| i.modifiers.command || i.modifiers.shift) {
                    self.selected_nodes.clear(); self.selected_annotation = None;
                }
            }
            Tool::Place(kind) => self.add_node(kind, world),
            Tool::Annotation => self.add_annotation(world),
            Tool::Wire => {
                if let Some(port) = self.port_at_screen(rect, pointer) {
                    match self.pending_wire_from {
                        None => { self.pending_wire_from = Some(port); self.notice = "Agora selecione o terminal de destino".to_owned(); }
                        Some(previous) if previous.node_id == port.node_id && previous.port == port.port => { self.pending_wire_from = None; self.notice = "Ligação cancelada".to_owned(); }
                        Some(previous) => self.connect_ports(previous, port),
                    }
                } else {
                    self.pending_wire_from = None;
                    self.notice = "Clique próximo a um terminal (ponto verde) ou ao símbolo".to_owned();
                }
            }
        }
    }

    // Guarda de undo do arraste em curso: evita criar um histórico por frame.
    fn drag_undo_saved(&self) -> bool { self.drag_undo_saved_flag }
    fn mark_drag_undo_saved(&mut self, value: bool) { self.drag_undo_saved_flag = value; }
    fn reset_drag_undo_saved(&mut self) { self.drag_undo_saved_flag = false; }

    fn hit_annotation_screen(&self, rect: Rect, screen: Pos2) -> Option<u64> {
        self.annotations.iter().rev().find(|a| {
            let pos = self.screen_from_world(rect, Pos2::new(a.x, a.y));
            let width = (a.text.chars().count() as f32 * 7.0 * self.zoom).max(40.0);
            Rect::from_min_size(pos, Vec2::new(width, 22.0 * self.zoom.max(0.8))).contains(screen)
        }).map(|a| a.id)
    }

    fn draw_help(&mut self, ctx: &egui::Context) {
        if !self.show_help { return; }
        egui::Window::new("Ajuda — Simulador")
            .open(&mut self.show_help)
            .resizable(true)
            .default_width(480.0)
            .show(ctx, |ui| {
                ui.label(RichText::new("Fluxo básico de desenho").strong().size(16.0));
                ui.label("1. Em Inserir ou na Biblioteca, escolha Fonte, Disjuntor ou Gerador e clique na área de desenho.");
                ui.label("2. Escolha Condutores e clique em um terminal verde; depois clique no terminal do componente de destino.");
                ui.label("3. Selecione um componente e edite a referência e as coordenadas no painel Propriedades.");
                ui.label("4. Arraste para mover; Ctrl+clique ou Shift+clique permite seleção múltipla.");
                ui.label("5. Espaço+arrastar move a tela; a roda do mouse controla o zoom.");
                ui.label("6. Ctrl+S salva, Ctrl+O abre, Ctrl+Z desfaz, Ctrl+Y refaz, Ctrl+C copia e Ctrl+V cola.");
                ui.add_space(6.0);
                ui.label(RichText::new("Os símbolos ainda são provisórios e o programa não calcula parâmetros elétricos nesta versão.").color(MUTED));
            });
    }
}

fn sanitize_filename(value: &str) -> String {
    let cleaned: String = value.chars().map(|c| if ['<', '>', ':', '"', '/', '\\', '|', '?', '*'].contains(&c) { '_' } else { c }).collect();
    let cleaned = cleaned.trim().trim_matches('.');
    if cleaned.is_empty() { "projeto".to_owned() } else { cleaned.to_owned() }
}

impl eframe::App for CadApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_shortcuts(ctx);

        // Reforça o tema escuro a cada frame: o aplicativo não segue as cores
        // definidas pelo Windows nem depende do tema claro/escuro do sistema.
        ctx.set_visuals(dark_visuals());

        egui::TopBottomPanel::top("menu_tabs")
            .frame(egui::Frame::none().fill(RIBBON_DARK))
            .show(ctx, |ui| self.draw_tabs(ui));
        egui::TopBottomPanel::top("tool_ribbon")
            .frame(egui::Frame::none().fill(RIBBON))
            .show(ctx, |ui| self.draw_toolbar(ui));
        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame::none().fill(RIBBON_DARK))
            .show(ctx, |ui| self.draw_statusbar(ui));

        if self.show_library {
            egui::SidePanel::left("component_library")
                .resizable(true)
                .default_width(190.0)
                .min_width(155.0)
                .frame(egui::Frame::none().fill(PANEL).stroke(Stroke::new(1.0_f32, RIBBON_DARK)))
                .show(ctx, |ui| self.draw_library(ui));
        }
        if self.show_properties {
            egui::SidePanel::right("properties_panel")
                .resizable(true)
                .default_width(245.0)
                .min_width(205.0)
                .frame(egui::Frame::none().fill(PANEL).stroke(Stroke::new(1.0_f32, RIBBON_DARK)))
                .show(ctx, |ui| self.draw_properties(ui));
        }

        egui::CentralPanel::default().frame(egui::Frame::none().fill(CANVAS).inner_margin(egui::Margin::same(0.0))).show(ctx, |ui| {
            let size = ui.available_size();
            let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
            self.handle_canvas_input(ctx, rect, &response);
            self.paint_canvas(ui.painter_at(rect), rect, ctx);
        });
        self.draw_help(ctx);
    }
}
