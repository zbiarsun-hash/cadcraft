//! The ribbon: tabs, groups, and buttons that start the commands CADCraft already has.

use cadcraft_engine::Session;
use cadcraft_ui_egui::{CadApp, Services, ribbon};
use egui::accesskit::Role;
use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use serde_json::json;

/// With the integrated title row (28), the tool bar (34), the tab strip (22) and the groups (74)
/// above it, everything the ribbon draws is above this line; the file tabs and Tool Sets below it.
const RIBBON_BOTTOM: f32 = 165.0;

fn harness(width: f32, height: f32) -> Harness<'static, CadApp> {
    let mut app = CadApp::new(Session::default(), Services::default());
    app.run("new", json!({})).unwrap();
    app.ui.start_tab = false;
    app.ui.in_window_menu = false;
    app.integrated_titlebar = true;
    let mut h = Harness::builder().with_size(vec2(width, height)).build_ui_state(
        |ui, app: &mut CadApp| {
            app.logic(&ui.ctx().clone());
            app.ui(ui);
        },
        app,
    );
    h.run_steps(5);
    h
}

/// The ribbon's button called `label`: the same name also labels the Tool Sets palette further down.
fn click_ribbon(h: &mut Harness<'_, CadApp>, label: &str) {
    h.get_all_by_label(label).find(|node| node.rect().top() < RIBBON_BOTTOM).unwrap_or_else(|| panic!("no ribbon button {label:?}")).click();
    h.run_steps(3);
}

fn in_ribbon(h: &Harness<'_, CadApp>, label: &str) -> bool {
    h.get_all_by_label(label).any(|node| node.rect().top() < RIBBON_BOTTOM)
}

fn tab(h: &mut Harness<'_, CadApp>, name: &str) {
    h.get_by_role_and_label(Role::Tab, name).click();
    h.run_steps(3);
}

#[test]
fn every_ribbon_button_is_an_existing_command() {
    let ids = ribbon::commands();
    assert!(ids.len() > 60, "the ribbon should expose the drafting commands, got {}", ids.len());
    let unknown: Vec<_> =
        ids.into_iter().filter(|id| cadcraft_engine::find_command(id).is_none() && !cadcraft_ui_egui::menus::is_ui_command(id)).collect();
    assert!(unknown.is_empty(), "ribbon buttons without a command: {unknown:?}");
    for name in ["Home", "Insert", "View", "Modify"] {
        assert!(ribbon::is_tab(name));
    }
    assert!(!ribbon::is_tab("Annotate"));
}

#[test]
fn home_tab_starts_the_same_commands_as_the_tool_sets_palette() {
    let mut h = harness(1600.0, 900.0);
    assert_eq!(h.state().ui.ribbon_tab, "Home", "the ribbon opens on Home");
    for (label, command) in [("Circle", "circle"), ("Move", "move"), ("Rectangle", "rectang"), ("Copy", "copy")] {
        click_ribbon(&mut h, label);
        assert_eq!(h.state().session.running.as_ref().map(|r| r.id.as_str()), Some(command), "{label}");
        h.state_mut().session.cancel();
    }
}

#[test]
fn clicking_a_tab_shows_its_groups_and_remembers_it() {
    let mut h = harness(1600.0, 900.0);
    assert!(in_ribbon(&h, "Circle") && !in_ribbon(&h, "Path Array"));
    tab(&mut h, "Modify");
    assert_eq!(h.state().ui.ribbon_tab, "Modify");
    assert!(in_ribbon(&h, "Path Array"), "the Modify tab has the Array group");
    assert!(!in_ribbon(&h, "Circle"), "the Draw group belongs to Home");
    click_ribbon(&mut h, "Path Array");
    assert_eq!(h.state().session.running.as_ref().map(|r| r.id.as_str()), Some("arraypath"));
    h.state_mut().session.cancel();

    tab(&mut h, "View");
    assert!(in_ribbon(&h, "Zoom Extents"));
    tab(&mut h, "Insert");
    assert!(in_ribbon(&h, "Insert Block"));
    assert_eq!(h.state().ui.ribbon_tab, "Insert");
}

#[test]
fn every_tab_runs_its_buttons_through_app_start() {
    let mut h = harness(1600.0, 900.0);
    for (tab_name, label, command) in [
        ("Insert", "Insert Block", "insert"),
        ("View", "Zoom Extents", "zoom.extents"),
        ("Modify", "Fillet", "fillet"),
        ("Home", "Polyline", "pline"),
    ] {
        tab(&mut h, tab_name);
        click_ribbon(&mut h, label);
        match h.state().session.running.as_ref() {
            Some(running) => assert_eq!(running.id, command, "{tab_name} / {label}"),
            // A command that finishes on its own (zoom) leaves nothing running.
            None => assert_eq!(command, "zoom.extents", "{tab_name} / {label} should start {command}"),
        }
        h.state_mut().session.cancel();
    }
}

#[test]
fn clicking_the_active_tab_minimizes_and_the_chevron_restores() {
    let mut h = harness(1600.0, 900.0);
    assert!(in_ribbon(&h, "Circle"));
    tab(&mut h, "Home");
    assert!(h.state().ui.ribbon_minimized);
    assert!(!in_ribbon(&h, "Circle"), "the groups are hidden when only the tabs are shown");
    // The drawing area gains the room the groups used.
    h.get_by_label("Ribbon").click();
    h.run_steps(3);
    assert!(!h.state().ui.ribbon_minimized);
    assert!(in_ribbon(&h, "Circle"));
    // Choosing another tab while minimized expands the ribbon.
    tab(&mut h, "Home");
    tab(&mut h, "View");
    assert!(!h.state().ui.ribbon_minimized);
    assert_eq!(h.state().ui.ribbon_tab, "View");
}

#[test]
fn the_ribbon_can_be_hidden_from_the_window_command_and_keeps_the_tool_sets() {
    let mut h = harness(1600.0, 900.0);
    assert!(h.query_by_role_and_label(Role::Tab, "Home").is_some());
    h.state_mut().run("ui.toggle.ribbon", json!({"on": false})).unwrap();
    h.run_steps(3);
    assert!(h.query_by_role_and_label(Role::Tab, "Home").is_none(), "hidden ribbon draws no tabs");
    assert!(h.query_by_label("Draw").is_some(), "the Tool Sets palette is untouched");
    h.state_mut().run("ui.toggle.ribbon", json!({"on": true})).unwrap();
    h.run_steps(3);
    assert!(h.query_by_role_and_label(Role::Tab, "Home").is_some());
}

#[test]
fn no_ribbon_on_the_start_page() {
    let mut h = harness(1600.0, 900.0);
    h.state_mut().ui.start_tab = true;
    h.run_steps(3);
    assert!(h.query_by_role_and_label(Role::Tab, "Home").is_none());
}

#[test]
fn a_small_window_scrolls_the_groups_and_keeps_the_ribbon_in_place() {
    for (width, height) in [(900.0, 560.0), (520.0, 480.0)] {
        let mut h = harness(width, height);
        let tabs: Vec<_> = ["Home", "Insert", "View", "Modify"].iter().map(|name| h.get_by_role_and_label(Role::Tab, name).rect()).collect();
        assert!(tabs.iter().all(|r| r.bottom() <= RIBBON_BOTTOM), "{width}x{height}: tabs stay in the top strip");
        // The first group is reachable without scrolling, and the strip never grows past its height.
        assert!(in_ribbon(&h, "Circle"), "{width}x{height}");
        click_ribbon(&mut h, "Circle");
        assert_eq!(h.state().session.running.as_ref().map(|r| r.id.as_str()), Some("circle"));
        h.state_mut().session.cancel();
    }
}

#[test]
fn the_ribbon_adds_a_fixed_height_and_typed_commands_still_work() {
    let mut h = harness(1280.0, 760.0);
    let top = h.get_by_role_and_label(Role::Tab, "Home").rect().top();
    let circle = h.get_all_by_label("Circle").find(|n| n.rect().top() < RIBBON_BOTTOM).unwrap().rect();
    assert!(circle.bottom() - top <= 22.0 + 74.0, "ribbon groups stay within the 96 px strip");
    // The command line is untouched by the ribbon.
    h.state_mut().cmdline("line");
    h.run_steps(3);
    assert_eq!(h.state().session.running.as_ref().map(|r| r.id.as_str()), Some("line"));
}
