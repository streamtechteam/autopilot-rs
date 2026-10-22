use super::*;

#[test]
fn tabs_are_comparable() {
    assert_eq!(Tabs::Overview, Tabs::Overview);
    assert_ne!(Tabs::Overview, Tabs::Jobs);
    assert_ne!(Tabs::Jobs, Tabs::Settings);
}

#[test]
fn gui_starts_on_the_overview_tab() {
    let gui = AutopilotGUI {
        active_tab: Tabs::Overview,
    };

    assert_eq!(gui.active_tab, Tabs::Overview);
}

#[test]
fn gui_can_switch_tabs() {
    let mut gui = AutopilotGUI {
        active_tab: Tabs::Overview,
    };
    gui.active_tab = Tabs::Settings;

    assert_eq!(gui.active_tab, Tabs::Settings);
}
