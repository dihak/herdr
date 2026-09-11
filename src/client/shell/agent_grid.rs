//! Client-only mosaic of detected agents. Does not resize PTYs or mutate tab layout.

use super::*;

pub(super) const AGENT_GRID_PAGE_CAP: usize = 16;
pub(super) const CARD_WIDTH: u16 = 36;
pub(super) const CARD_HEIGHT: u16 = 10;
pub(super) const CELL_GAP: u16 = 1;

impl AgentGridFilter {
    pub(super) fn includes(self, status: crate::api::schema::AgentStatus) -> bool {
        match self {
            Self::Attention => matches!(
                status,
                crate::api::schema::AgentStatus::Working | crate::api::schema::AgentStatus::Blocked
            ),
            Self::All => true,
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Attention => "working/blocked",
            Self::All => "all agents",
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct AgentGridTile {
    pub(super) target: AgentGridTarget,
    pub(super) stale: bool,
    pub(super) current: bool,
}

pub(super) fn agent_grid_tiles(
    endpoints: &[ClientShellEndpoint],
    active_endpoint_id: &ClientEndpointId,
    sort: crate::config::AgentPanelSortConfig,
    filter: AgentGridFilter,
) -> Vec<AgentGridTile> {
    super::aggregate_navigation::aggregate_agent_rows(endpoints, sort)
        .into_iter()
        .filter(|row| filter.includes(row.agent.agent_status))
        .map(|row| AgentGridTile {
            target: AgentGridTarget {
                endpoint_id: row.endpoint.endpoint_id.clone(),
                pane_id: row.agent.pane_id.clone(),
            },
            stale: row.endpoint.stale(),
            current: row.endpoint.endpoint_id == active_endpoint_id && row.agent.focused,
        })
        .collect()
}

pub(super) fn grid_geometry(count: usize, area: Rect) -> (u16, u16, usize) {
    if count == 0 || area.width < CARD_WIDTH || area.height < CARD_HEIGHT {
        return (0, 0, 0);
    }
    let max_cols = ((area.width + CELL_GAP) / (CARD_WIDTH + CELL_GAP)).max(1);
    let max_rows = ((area.height + CELL_GAP) / (CARD_HEIGHT + CELL_GAP)).max(1);
    let page_cap = (max_cols as usize)
        .saturating_mul(max_rows as usize)
        .clamp(1, AGENT_GRID_PAGE_CAP);
    let visible = count.min(page_cap);
    let cols = (visible as u16).clamp(1, max_cols);
    let rows = (visible.div_ceil(cols as usize) as u16).clamp(1, max_rows);
    let page_len = (cols as usize).saturating_mul(rows as usize).min(page_cap);
    (cols, rows, page_len.max(1))
}

pub(super) fn page_count(total: usize, page_len: usize) -> usize {
    if total == 0 || page_len == 0 {
        0
    } else {
        total.div_ceil(page_len)
    }
}

pub(super) fn selected_index(tiles: &[AgentGridTile], selected: Option<&AgentGridTarget>) -> usize {
    selected
        .and_then(|selected| tiles.iter().position(|tile| &tile.target == selected))
        .unwrap_or(0)
}

pub(super) fn clamp_page(page: usize, total: usize, page_len: usize) -> usize {
    let pages = page_count(total, page_len);
    if pages == 0 {
        0
    } else {
        page.min(pages - 1)
    }
}

pub(super) fn visible_page(
    tiles: &[AgentGridTile],
    page: usize,
    page_len: usize,
) -> &[AgentGridTile] {
    if page_len == 0 || tiles.is_empty() {
        return &[];
    }
    let page = clamp_page(page, tiles.len(), page_len);
    let start = page.saturating_mul(page_len);
    let end = (start + page_len).min(tiles.len());
    &tiles[start..end]
}

pub(super) fn cell_rect(body: Rect, cols: u16, rows: u16, index: usize) -> Rect {
    let cols = cols.max(1);
    let rows = rows.max(1);
    let col = (index as u16) % cols;
    let row = (index as u16) / cols;
    let cell_w = body
        .width
        .saturating_sub(CELL_GAP.saturating_mul(cols.saturating_sub(1)))
        / cols;
    let cell_h = body
        .height
        .saturating_sub(CELL_GAP.saturating_mul(rows.saturating_sub(1)))
        / rows;
    Rect::new(
        body.x + col * (cell_w + CELL_GAP),
        body.y + row * (cell_h + CELL_GAP),
        cell_w.max(1),
        cell_h.max(1),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_geometry_caps_page_and_keeps_minimum_cells() {
        let area = Rect::new(0, 0, 80, 24);
        let (cols, rows, page_len) = grid_geometry(40, area);
        assert!(cols >= 1);
        assert!(rows >= 1);
        assert!(page_len <= AGENT_GRID_PAGE_CAP);
        assert!(page_len >= 1);
        let tiny = Rect::new(0, 0, 10, 3);
        assert_eq!(grid_geometry(8, tiny), (0, 0, 0));
        let two = grid_geometry(2, Rect::new(0, 0, 80, 24));
        assert_eq!(two.0, 2);
        assert_eq!(two.1, 1);
    }

    #[test]
    fn attention_filter_keeps_working_and_blocked() {
        assert!(AgentGridFilter::Attention.includes(crate::api::schema::AgentStatus::Working));
        assert!(AgentGridFilter::Attention.includes(crate::api::schema::AgentStatus::Blocked));
        assert!(!AgentGridFilter::Attention.includes(crate::api::schema::AgentStatus::Idle));
        assert!(AgentGridFilter::All.includes(crate::api::schema::AgentStatus::Idle));
    }
}
