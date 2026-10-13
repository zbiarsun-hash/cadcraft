//! The Ribbon: tabs of command groups under the menu bar.
//!
//! It adds no commands. Every button starts an existing command through `CadApp::start`, exactly
//! as the Tool Sets palette and the Tool Bar do, and the icons and tooltips come from the same
//! tables ([`crate::palettes::DRAFTING`]) wherever a command is already in one. A command without
//! an icon of its own gets a text button, so there are no stand-in icons.

use egui::{Align2, Color32, FontId, Rect, Sense, Shape, Stroke, pos2, vec2};

use crate::CadApp;
use crate::icons::{self, Icon};
use crate::palettes;
use crate::theme::Tokens;

/// Height of the tab strip.
const TAB_H: f32 = 22.0;
/// Height of the command area under the tabs: three rows of small buttons, or one large button.
const BODY_H: f32 = 56.0;
/// Height of the strip that carries the group names.
const GROUP_LABEL_H: f32 = 13.0;
/// The ribbon's height below the tab strip: padding, command area and group names.
const PANEL_H: f32 = 3.0 + BODY_H + GROUP_LABEL_H + 2.0;
const SMALL_H: f32 = 18.0;
const SMALL_ICON_W: f32 = 22.0;
const SMALL_ROWS: usize = 3;
const LARGE_MIN_W: f32 = 44.0;
const LARGE_MAX_W: f32 = 64.0;
const GROUP_PAD: f32 = 5.0;
const LABEL_SIZE: f32 = 10.5;

/// A button: the command it starts, and an icon or label where the shared tables have none.
#[derive(Clone, Copy)]
struct Item {
    icon: Option<Icon>,
    cmd: &'static str,
}

/// A command whose icon and name come from the shared tables (or the command's own name).
const fn c(cmd: &'static str) -> Item {
    Item { icon: None, cmd }
}

/// A command with an icon CADCraft already draws.
const fn ic(icon: Icon, cmd: &'static str) -> Item {
    Item { icon: Some(icon), cmd }
}

enum Group {
    /// A whole Tool Sets group, by name: the ribbon and the palette cannot drift apart.
    Drafting(&'static str),
    /// A group of its own: name, large buttons, small buttons.
    Custom(&'static str, &'static [Item], &'static [Item]),
}

pub struct Tab {
    pub name: &'static str,
    groups: &'static [Group],
}

pub const TABS: &[Tab] = &[
    Tab {
        name: "Home",
        groups: &[
            Group::Drafting("Draw"),
            Group::Drafting("Modify"),
            Group::Drafting("Text"),
            Group::Custom(
                "Layers",
                &[ic(Icon::Layers, "ui.dialog.layers")],
                &[c("laymcur"), c("layiso"), c("layuniso"), c("layfrz"), c("layoff"), c("laythw")],
            ),
            Group::Custom(
                "Properties",
                &[ic(Icon::Properties, "ui.panel.properties")],
                &[c("matchprop"), c("color"), c("linetype"), c("lweight"), c("ltscale")],
            ),
        ],
    },
    Tab {
        name: "Insert",
        groups: &[
            Group::Drafting("Block"),
            Group::Custom("Import", &[ic(Icon::Import, "import"), ic(Icon::Attach, "attach")], &[c("xref")]),
            Group::Drafting("Hatch"),
            Group::Drafting("Text"),
            Group::Drafting("Dimension"),
            Group::Drafting("Leader"),
        ],
    },
    Tab {
        name: "View",
        groups: &[
            Group::Custom(
                "Navigate",
                &[ic(Icon::ZoomWindow, "zoom.window"), ic(Icon::ZoomExtents, "zoom.extents"), ic(Icon::Pan, "pan"), ic(Icon::Orbit, "3dorbit")],
                &[c("zoom.previous"), c("zoom.all"), c("zoom.in"), c("zoom.out"), c("zoom.object"), c("regen")],
            ),
            Group::Custom(
                "Window",
                &[],
                &[
                    c("ui.toggle.toolsets"),
                    c("ui.panel.properties"),
                    c("ui.panel.layers"),
                    c("ui.panel.quickcalc"),
                    c("ui.toggle.cmdline"),
                    c("ui.toggle.history"),
                    c("ui.toggle.viewcube"),
                    c("ui.toggle.ucsicon"),
                ],
            ),
        ],
    },
    Tab {
        name: "Modify",
        groups: &[
            Group::Custom(
                "Modify",
                &[c("move"), c("copy"), c("rotate"), c("scale")],
                &[c("mirror"), c("stretch"), c("offset"), c("align"), c("erase")],
            ),
            Group::Custom("Edit", &[c("trim"), c("extend"), c("fillet"), c("chamfer")], &[c("break"), c("join"), c("lengthen"), c("explode")]),
            Group::Custom("Array", &[c("arrayrect"), c("arraypolar")], &[c("arraypath")]),
            Group::Custom(
                "Object",
                &[c("matchprop")],
                &[c("pedit"), c("splinedit"), c("hatchedit"), c("draworder"), c("group"), c("ungroup"), c("overkill")],
            ),
        ],
    },
];

/// Whether `name` is one of the ribbon's tabs.
pub fn is_tab(name: &str) -> bool {
    TABS.iter().any(|t| t.name == name)
}

type Tool = (Icon, &'static str, &'static str);

/// A button with everything needed to draw and run it.
struct Resolved {
    icon: Option<Icon>,
    cmd: &'static str,
    /// The English name: the tooltip, and the button text when there is no icon.
    label: &'static str,
}

fn drafting_tool(cmd: &str) -> Option<Tool> {
    palettes::DRAFTING.iter().flat_map(|(_, large, small)| large.iter().chain(small.iter())).find(|tool| tool.1 == cmd).copied()
}

/// The name a command carries in the engine's table or the UI-only command table.
fn command_label(cmd: &str) -> Option<&'static str> {
    cadcraft_engine::find_command(cmd).map(|spec| spec.label).or_else(|| crate::menus::UI_COMMANDS.iter().find(|ui| ui.0 == cmd).map(|ui| ui.1))
}

fn command_known(cmd: &str) -> bool {
    cadcraft_engine::find_command(cmd).is_some() || crate::menus::is_ui_command(cmd)
}

fn resolve(item: &Item) -> Resolved {
    let tool = drafting_tool(item.cmd);
    Resolved {
        icon: item.icon.or(tool.map(|t| t.0)),
        cmd: item.cmd,
        label: tool.map(|t| t.2).or_else(|| command_label(item.cmd)).unwrap_or(item.cmd),
    }
}

fn from_tool(tool: &Tool) -> Resolved {
    Resolved { icon: Some(tool.0), cmd: tool.1, label: tool.2 }
}

/// A group's name, large buttons and small buttons.
fn resolve_group(group: &Group) -> (&'static str, Vec<Resolved>, Vec<Resolved>) {
    match group {
        Group::Drafting(name) => match palettes::DRAFTING.iter().find(|(n, _, _)| n == name) {
            Some((_, large, small)) => (name, large.iter().map(from_tool).collect(), small.iter().map(from_tool).collect()),
            None => (name, Vec::new(), Vec::new()),
        },
        Group::Custom(name, large, small) => (name, large.iter().map(resolve).collect(), small.iter().map(resolve).collect()),
    }
}

/// Every command id the ribbon can start, for tests.
pub fn commands() -> Vec<&'static str> {
    let mut ids = Vec::new();
    for tab in TABS {
        for group in tab.groups {
            let (_, large, small) = resolve_group(group);
            ids.extend(large.iter().chain(small.iter()).map(|r| r.cmd));
        }
    }
    ids
}

/// Every English name the ribbon shows or reads out, for the translation check.
pub fn labels() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = TABS.iter().map(|t| t.name).collect();
    for tab in TABS {
        for group in tab.groups {
            let (name, large, small) = resolve_group(group);
            names.push(name);
            names.extend(large.iter().chain(small.iter()).map(|r| r.label));
        }
    }
    names
}

/// One placed button. Positions are relative to the top-left of the command area.
struct Cell {
    rect: Rect,
    item: Resolved,
    large: bool,
    /// The text drawn on the button, cut short with "…" to fit.
    text: String,
}

struct GroupLabel {
    center_x: f32,
    text: String,
}

struct Laid {
    cells: Vec<Cell>,
    labels: Vec<GroupLabel>,
    separators: Vec<f32>,
    width: f32,
}

fn text_width(ui: &egui::Ui, text: &str, size: f32) -> f32 {
    ui.painter().layout_no_wrap(text.to_string(), FontId::proportional(size), Color32::WHITE).size().x
}

/// `text`, cut short with "…" until it is at most `max` wide.
fn elide(ui: &egui::Ui, text: &str, size: f32, max: f32) -> String {
    if text_width(ui, text, size) <= max {
        return text.to_string();
    }
    let mut cut: String = text.to_string();
    while !cut.is_empty() {
        cut.pop();
        let trial = format!("{}…", cut.trim_end());
        if text_width(ui, &trial, size) <= max {
            return trial;
        }
    }
    "…".to_string()
}

/// Place every group of `tab` left to right: large buttons first, then the small ones in columns
/// of three, then the group name underneath.
fn layout(ui: &egui::Ui, tab: &Tab) -> Laid {
    let mut laid = Laid { cells: Vec::new(), labels: Vec::new(), separators: Vec::new(), width: 0.0 };
    let mut x = 0.0;
    for (index, group) in tab.groups.iter().enumerate() {
        let (name, large, small) = resolve_group(group);
        if large.is_empty() && small.is_empty() {
            continue;
        }
        x += GROUP_PAD;
        let content_start = x;
        for item in large {
            let text = crate::i18n::t(item.label).to_string();
            let width = (text_width(ui, &text, LABEL_SIZE) + 8.0).clamp(LARGE_MIN_W, LARGE_MAX_W);
            let text = elide(ui, &text, LABEL_SIZE, width - 4.0);
            laid.cells.push(Cell { rect: Rect::from_min_size(pos2(x, 0.0), vec2(width, BODY_H)), item, large: true, text });
            x += width;
        }
        let mut columns: Vec<Vec<Resolved>> = Vec::new();
        for item in small {
            if columns.last().is_none_or(|column| column.len() == SMALL_ROWS) {
                columns.push(Vec::new());
            }
            if let Some(column) = columns.last_mut() {
                column.push(item);
            }
        }
        for column in columns {
            let text_of = |item: &Resolved| crate::i18n::t(item.label).to_string();
            let width = column
                .iter()
                .map(|item| if item.icon.is_some() { SMALL_ICON_W } else { (text_width(ui, &text_of(item), 11.0) + 12.0).min(150.0) })
                .fold(SMALL_ICON_W, f32::max);
            for (row, item) in column.into_iter().enumerate() {
                let text = if item.icon.is_some() { String::new() } else { elide(ui, &text_of(&item), 11.0, width - 8.0) };
                let rect = Rect::from_min_size(pos2(x, row as f32 * (SMALL_H + 1.0)), vec2(width, SMALL_H));
                laid.cells.push(Cell { rect, item, large: false, text });
            }
            x += width;
        }
        let name = crate::i18n::t(name).to_string();
        let label_w = text_width(ui, &name, LABEL_SIZE - 0.5) + 8.0;
        let content_w = (x - content_start).max(label_w);
        laid.labels.push(GroupLabel { center_x: content_start + content_w / 2.0, text: name });
        x = content_start + content_w + GROUP_PAD;
        if index + 1 < tab.groups.len() {
            laid.separators.push(x);
        }
    }
    laid.width = x;
    laid
}

/// Draw the ribbon and run the command a button starts. Nothing is drawn without a drawing open:
/// the commands need one, and the Start page has its own content.
pub fn show(app: &mut CadApp, ui: &mut egui::Ui) {
    if !app.ui.show_ribbon || app.session.docs.is_empty() || app.ui.start_tab {
        return;
    }
    let t = Tokens::get();
    let active = TABS.iter().position(|tab| tab.name == app.ui.ribbon_tab).unwrap_or(0);
    let minimized = app.ui.ribbon_minimized;
    let height = TAB_H + if minimized { 0.0 } else { PANEL_H };
    let laid = layout(ui, &TABS[active]);
    egui::Panel::top("cc_ribbon").exact_size(height).frame(egui::Frame::NONE.fill(t.panel)).show(ui, |ui| {
        let r = ui.max_rect();
        let strip = Rect::from_min_size(r.min, vec2(r.width(), TAB_H));
        ui.painter().rect_filled(strip, 0.0, t.chrome_dark);
        ui.painter().hline(r.x_range(), strip.bottom() - 0.5, Stroke::new(1.0, t.border));

        let mut selected: Option<&'static str> = None;
        let mut x = r.left() + 6.0;
        for tab in TABS {
            let name = crate::i18n::t(tab.name).to_string();
            let width = text_width(ui, &name, 13.0) + 24.0;
            let tab_rect = Rect::from_min_size(pos2(x, strip.top()), vec2(width, TAB_H));
            let is_active = tab.name == TABS[active].name;
            let resp = ui.interact(tab_rect, ui.id().with(("ribbon_tab", tab.name)), Sense::click());
            let fill = if is_active {
                t.panel
            } else if resp.hovered() {
                t.chrome
            } else {
                t.chrome_dark
            };
            ui.painter().rect_filled(tab_rect, 0.0, fill);
            if is_active {
                ui.painter().hline(tab_rect.x_range(), strip.bottom() - 0.5, Stroke::new(1.0, t.panel));
            }
            let color = if is_active { t.text } else { t.text_dim };
            ui.painter().text(tab_rect.center(), Align2::CENTER_CENTER, &name, crate::theme::body(), color);
            icons::describe_control(ui, &resp, &name, Some(is_active), true);
            if resp.clicked() {
                selected = Some(tab.name);
            }
            x += width;
        }

        // Minimise or expand: a chevron at the right end of the tab strip.
        let chevron = Rect::from_min_size(pos2(r.right() - 26.0, strip.top()), vec2(22.0, TAB_H));
        let resp = ui.interact(chevron, ui.id().with("ribbon_minimize"), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(chevron.shrink2(vec2(0.0, 2.0)), 3.0, t.control_hover.gamma_multiply(0.6));
        }
        let c = chevron.center();
        let (dy, dx) = (if minimized { -2.0 } else { 2.0 }, 4.0);
        ui.painter().add(Shape::line(vec![pos2(c.x - dx, c.y - dy), pos2(c.x, c.y + dy), pos2(c.x + dx, c.y - dy)], Stroke::new(1.4, t.text_dim)));
        icons::describe_control(ui, &resp, "Ribbon", Some(!minimized), false);
        let toggle_min = resp.on_hover_text(crate::tl!("Ribbon")).clicked();

        let mut clicked: Option<&'static str> = None;
        if !minimized {
            let body = Rect::from_min_size(pos2(r.left(), strip.bottom()), vec2(r.width(), PANEL_H));
            ui.scope_builder(egui::UiBuilder::new().max_rect(body), |ui| {
                egui::ScrollArea::horizontal().id_salt("ribbon_scroll").auto_shrink([false, false]).show(ui, |ui| {
                    let (area, _) = ui.allocate_exact_size(vec2(laid.width.max(body.width()), PANEL_H), Sense::hover());
                    let origin = pos2(area.left(), area.top() + 3.0);
                    for sep in &laid.separators {
                        let sx = origin.x + sep - 1.0;
                        ui.painter().vline(sx, (origin.y + 2.0)..=(origin.y + BODY_H + GROUP_LABEL_H - 2.0), Stroke::new(1.0, t.border));
                    }
                    for label in &laid.labels {
                        let center = pos2(origin.x + label.center_x, origin.y + BODY_H + GROUP_LABEL_H / 2.0);
                        ui.painter().text(center, Align2::CENTER_CENTER, &label.text, FontId::proportional(LABEL_SIZE - 0.5), t.text_faint);
                    }
                    for (index, cell) in laid.cells.iter().enumerate() {
                        let rect = cell.rect.translate(origin.to_vec2());
                        let known = command_known(cell.item.cmd);
                        let resp = ui.interact(
                            rect,
                            ui.id().with(("ribbon_cmd", active, index, cell.item.cmd)),
                            if known { Sense::click() } else { Sense::hover() },
                        );
                        if resp.hovered() && known {
                            ui.painter().rect_filled(rect.shrink(1.0), 3.0, t.control_hover.gamma_multiply(0.6));
                        }
                        let dim = !known;
                        let text_color = if dim { t.text_faint } else { t.text };
                        match (cell.large, cell.item.icon) {
                            (true, Some(icon)) => {
                                let icon_rect = Rect::from_center_size(pos2(rect.center().x, rect.top() + 4.0 + 16.0), vec2(32.0, 32.0));
                                icons::paint(ui.painter(), icon_rect.shrink(2.0), icon, dim);
                                ui.painter().text(
                                    pos2(rect.center().x, rect.bottom() - 1.0),
                                    Align2::CENTER_BOTTOM,
                                    &cell.text,
                                    FontId::proportional(LABEL_SIZE),
                                    text_color,
                                );
                            }
                            (_, Some(icon)) => {
                                icons::paint(ui.painter(), Rect::from_center_size(rect.center(), vec2(16.0, 16.0)), icon, dim);
                            }
                            (true, None) => {
                                // A large button for a command with no icon: its name, centred.
                                ui.painter().text(rect.center(), Align2::CENTER_CENTER, &cell.text, FontId::proportional(11.0), text_color);
                            }
                            (false, None) => {
                                ui.painter().text(
                                    pos2(rect.left() + 6.0, rect.center().y),
                                    Align2::LEFT_CENTER,
                                    &cell.text,
                                    FontId::proportional(11.0),
                                    text_color,
                                );
                            }
                        }
                        let tip = crate::i18n::t(cell.item.label).to_string();
                        icons::describe_control(ui, &resp, &tip, None, false);
                        if resp.on_hover_text(tip).clicked() {
                            clicked = Some(cell.item.cmd);
                        }
                    }
                });
            });
        }
        ui.painter().hline(r.x_range(), r.bottom() - 0.5, Stroke::new(1.0, t.border));

        if let Some(name) = selected {
            if name == TABS[active].name {
                app.ui.ribbon_minimized = !app.ui.ribbon_minimized;
            } else {
                app.ui.ribbon_tab = name.to_string();
                app.ui.ribbon_minimized = false;
            }
        }
        if toggle_min {
            app.ui.ribbon_minimized = !app.ui.ribbon_minimized;
        }
        if let Some(command) = clicked {
            app.cmd.buffer.clear();
            app.start(command);
        }
    });
}
