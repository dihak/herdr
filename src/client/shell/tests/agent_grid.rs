use super::*;
use crate::api::schema::AgentStatus;
use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};

fn working_agent(
    pane_id: &str,
    name: &str,
    status: AgentStatus,
    focused: bool,
) -> ClientShellAgent {
    ClientShellAgent {
        pane_id: pane_id.into(),
        workspace_id: "ws_1".into(),
        tab_id: "tab_1".into(),
        name: Some(name.into()),
        display_agent: None,
        agent: Some("pi".into()),
        title: None,
        terminal_title: None,
        terminal_title_stripped: None,
        agent_status: status,
        state_change_seq: 1,
        state_labels: Vec::new(),
        tokens: Vec::new(),
        focused,
    }
}

fn grid_snapshot() -> ClientShellSnapshot {
    let mut snapshot = snapshot();
    let mut second = snapshot.panes[0].clone();
    second.pane_id = "pane_2".into();
    second.focused = false;
    snapshot.panes.push(second);
    snapshot.agents = vec![
        working_agent("pane_1", "coder", AgentStatus::Working, true),
        working_agent("pane_2", "reviewer", AgentStatus::Blocked, false),
        working_agent("pane_1", "idle-skip", AgentStatus::Idle, false),
    ];
    snapshot.agents[2].pane_id = "pane_idle".into();
    let mut idle_pane = snapshot.panes[0].clone();
    idle_pane.pane_id = "pane_idle".into();
    idle_pane.focused = false;
    snapshot.panes.push(idle_pane);
    snapshot
}

fn frame_text(frame: &crate::protocol::FrameData) -> String {
    frame
        .cells
        .chunks(frame.width as usize)
        .map(|row| {
            row.iter()
                .map(|cell| cell.symbol.as_str())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn agent_grid_defaults_to_browse_and_jumps_on_enter() {
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(grid_snapshot()));
    state.set_pane_surface(surface());
    state.open_agent_grid_overlay();
    assert!(matches!(
        state.overlay,
        Some(ClientShellOverlay::AgentGrid(ClientAgentGridOverlay {
            filter: AgentGridFilter::All,
            insert: false,
            ..
        }))
    ));
    let frame = state.compose(106, 30).expect("agent grid");
    let text = frame_text(&frame);
    assert!(text.contains("all agents"));
    assert!(text.contains("i type"));
    assert!(!text.contains("typing in selected agent"));

    let outcome = state.handle_raw_events(vec![RawInputEvent::Key(
        crate::input::TerminalKey::new(KeyCode::Enter, KeyModifiers::empty()),
    )]);
    let [ClientShellAction::Endpoint { request, .. }] = &outcome.actions[..] else {
        panic!("grid enter should focus a pane");
    };
    assert!(matches!(
        &request.method,
        crate::api::schema::Method::PaneFocus(target) if target.pane_id == "pane_1"
    ));
    assert!(state.overlay.is_none());
}

#[test]
fn agent_grid_all_filter_and_mouse_click_focus_selected_cell() {
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(grid_snapshot()));
    state.set_pane_surface(surface());
    state.open_agent_grid_overlay();
    let frame = state.compose(106, 30).expect("all agents grid");
    let text = frame_text(&frame);
    assert!(text.contains("all agents"));

    let reviewer = state
        .hits
        .agent_grid_cells
        .iter()
        .find(|(_, target)| target.pane_id == "pane_2")
        .map(|(rect, _)| *rect)
        .expect("reviewer cell");
    let accept =
        state.handle_raw_events(vec![RawInputEvent::Mouse(crossterm::event::MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: reviewer.x + 1,
            row: reviewer.y,
            modifiers: KeyModifiers::empty(),
        })]);
    let [ClientShellAction::Endpoint { request, .. }] = &accept.actions[..] else {
        panic!("grid click should focus a pane");
    };
    assert!(matches!(
        &request.method,
        crate::api::schema::Method::PaneFocus(target) if target.pane_id == "pane_2"
    ));
    assert!(state.overlay.is_none());
}

#[test]
fn agent_grid_insert_sends_keys_to_selected_pane() {
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(grid_snapshot()));
    state.set_pane_surface(surface());
    state.open_agent_grid_overlay();
    state.handle_raw_events(vec![RawInputEvent::Key(crate::input::TerminalKey::new(
        KeyCode::Char('i'),
        KeyModifiers::empty(),
    ))]);
    let typed = state.handle_raw_events(vec![RawInputEvent::Key(crate::input::TerminalKey::new(
        KeyCode::Char('x'),
        KeyModifiers::empty(),
    ))]);
    let [ClientShellAction::Endpoint { request, .. }] = &typed.actions[..] else {
        panic!("grid typing should send pane.send_input");
    };
    assert!(matches!(
        &request.method,
        crate::api::schema::Method::PaneSendInput(params) if params.pane_id == "pane_1" && params.keys == ["x"]
    ));
    state.handle_raw_events(vec![RawInputEvent::Key(crate::input::TerminalKey::new(
        KeyCode::Esc,
        KeyModifiers::empty(),
    ))]);
    assert!(matches!(
        state.overlay,
        Some(ClientShellOverlay::AgentGrid(ClientAgentGridOverlay {
            insert: false,
            ..
        }))
    ));
}

#[test]
fn agent_grid_types_into_other_workspace_pane() {
    let mut snapshot = grid_snapshot();
    snapshot.workspaces.push(ClientShellWorkspace {
        workspace_id: "ws_2".into(),
        active_tab_id: "tab_2".into(),
        new_workspace_cwd: "/other".into(),
        number: 2,
        label: "other".into(),
        custom_label: false,
        branch: None,
        git_ahead_behind: None,
        tokens: Vec::new(),
        worktree: None,
        focused: false,
        agent_status: AgentStatus::Idle,
    });
    snapshot.tabs.push(ClientShellTab {
        tab_id: "tab_2".into(),
        workspace_id: "ws_2".into(),
        number: 1,
        label: "2".into(),
        custom_label: false,
        zoomed: false,
        focused: false,
        agent_status: AgentStatus::Idle,
    });
    snapshot.panes.push(ClientShellPane {
        pane_id: "pane_other".into(),
        workspace_id: "ws_2".into(),
        tab_id: "tab_2".into(),
        label: None,
        cwd: Some("/other".into()),
        foreground_cwd: Some("/other".into()),
        focused: false,
        right_click_passthrough: false,
    });
    let mut other = working_agent("pane_other", "other-pi", AgentStatus::Idle, false);
    other.workspace_id = "ws_2".into();
    other.tab_id = "tab_2".into();
    snapshot.agents.push(other);

    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(snapshot));
    state.set_pane_surface(surface());
    state.open_agent_grid_overlay();
    if let Some(ClientShellOverlay::AgentGrid(grid)) = state.overlay.as_mut() {
        if let Some(selected) = grid.selected.as_mut() {
            selected.pane_id = "pane_other".into();
        }
        grid.insert = true;
    }
    let typed = state.handle_raw_events(vec![RawInputEvent::Key(crate::input::TerminalKey::new(
        KeyCode::Char('z'),
        KeyModifiers::empty(),
    ))]);
    let [ClientShellAction::Endpoint { request, .. }] = &typed.actions[..] else {
        panic!("other-workspace typing should send pane.send_input");
    };
    assert!(matches!(
        &request.method,
        crate::api::schema::Method::PaneSendInput(params)
            if params.pane_id == "pane_other" && params.keys == ["z"]
    ));
}
