// Layout management for terminal display

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use super::state::ViewMode;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct PanelRect {
    pub panel: ViewMode,
    pub rect: Rect,
}

// Layer weight constants for dynamic layout calculation
const TOP_LAYER_WEIGHT: u16 = 1;
const MIDDLE_LAYER_WEIGHT: u16 = 1;
const BOTTOM_LAYER_WEIGHT: u16 = 2;

pub struct MultiPanelLayout {
    min_panel_height: u16,
    min_panel_width: u16,
}

impl MultiPanelLayout {
    pub fn new() -> Self {
        Self {
            min_panel_height: 8,
            min_panel_width: 30,
        }
    }

    /// Validate minimum terminal size
    pub fn validate_minimum_size(&self, area: Rect) -> Result<(), String> {
        if area.height < self.min_panel_height * 2 {
            return Err(format!(
                "Terminal too small (height: {}). Need at least {} rows.",
                area.height,
                self.min_panel_height * 2
            ));
        }
        if area.width < self.min_panel_width {
            return Err(format!(
                "Terminal too small (width: {}). Need at least {} columns.",
                area.width,
                self.min_panel_width
            ));
        }
        Ok(())
    }

    /// Calculate panel rectangles based on three-layer layout
    pub fn calculate(
        &self,
        area: Rect,
        visible_panels: &BTreeSet<ViewMode>,
    ) -> Vec<PanelRect> {
        // Special case: single panel uses full area
        if visible_panels.len() == 1 {
            let panel = *visible_panels.iter().next().unwrap();
            return vec![PanelRect { panel, rect: area }];
        }

        // Determine which layers have visible panels
        let has_top = visible_panels.contains(&ViewMode::Cpu)
            || visible_panels.contains(&ViewMode::Gpu);
        let has_middle = visible_panels.contains(&ViewMode::Memory)
            || visible_panels.contains(&ViewMode::Network)
            || visible_panels.contains(&ViewMode::DiskIo)
            || visible_panels.contains(&ViewMode::DiskUsage);
        let has_bottom = visible_panels.contains(&ViewMode::Processes);

        // Calculate vertical splits with weight-based dynamic percentages
        let mut total_weight: u16 = 0;
        if has_top {
            total_weight += TOP_LAYER_WEIGHT;
        }
        if has_middle {
            total_weight += MIDDLE_LAYER_WEIGHT;
        }
        if has_bottom {
            total_weight += BOTTOM_LAYER_WEIGHT;
        }

        let mut vertical_constraints = Vec::new();
        if has_top {
            let percent = (TOP_LAYER_WEIGHT * 100) / total_weight;
            vertical_constraints.push(Constraint::Percentage(percent));
        }
        if has_middle {
            let percent = (MIDDLE_LAYER_WEIGHT * 100) / total_weight;
            vertical_constraints.push(Constraint::Percentage(percent));
        }
        if has_bottom {
            // Use Min(0) for last constraint to absorb rounding errors
            vertical_constraints.push(Constraint::Min(0));
        }

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vertical_constraints)
            .split(area);

        let mut results = Vec::new();
        let mut chunk_idx = 0;

        // Top layer: CPU and/or GPU
        if has_top {
            results.extend(self.calculate_top_layer(
                vertical_chunks[chunk_idx],
                visible_panels,
            ));
            chunk_idx += 1;
        }

        // Middle layer: Memory, Network, DiskIo, DiskUsage
        if has_middle {
            results.extend(self.calculate_middle_layer(
                vertical_chunks[chunk_idx],
                visible_panels,
            ));
            chunk_idx += 1;
        }

        // Bottom layer: Processes
        if has_bottom {
            results.push(PanelRect {
                panel: ViewMode::Processes,
                rect: vertical_chunks[chunk_idx],
            });
        }

        results
    }

    fn calculate_top_layer(
        &self,
        area: Rect,
        visible_panels: &BTreeSet<ViewMode>,
    ) -> Vec<PanelRect> {
        let has_cpu = visible_panels.contains(&ViewMode::Cpu);
        let has_gpu = visible_panels.contains(&ViewMode::Gpu);

        if has_cpu && has_gpu {
            // Both visible: split 50/50
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

            vec![
                PanelRect {
                    panel: ViewMode::Cpu,
                    rect: chunks[0],
                },
                PanelRect {
                    panel: ViewMode::Gpu,
                    rect: chunks[1],
                },
            ]
        } else if has_cpu {
            vec![PanelRect {
                panel: ViewMode::Cpu,
                rect: area,
            }]
        } else {
            vec![PanelRect {
                panel: ViewMode::Gpu,
                rect: area,
            }]
        }
    }

    fn calculate_middle_layer(
        &self,
        area: Rect,
        visible_panels: &BTreeSet<ViewMode>,
    ) -> Vec<PanelRect> {
        let middle_panels: Vec<ViewMode> = [
            ViewMode::Memory,
            ViewMode::Network,
            ViewMode::DiskIo,
            ViewMode::DiskUsage,
        ]
        .iter()
        .filter(|p| visible_panels.contains(p))
        .copied()
        .collect();

        if middle_panels.is_empty() {
            return Vec::new();
        }

        let width = area.width as usize;

        // Smart width adaptation
        if width >= 120 {
            // Wide: All panels in single row
            self.calculate_middle_single_row(area, &middle_panels)
        } else if width >= 90 {
            // Medium: Memory full row, others in second row
            self.calculate_middle_two_rows(area, &middle_panels)
        } else {
            // Narrow: Each panel gets own row
            self.calculate_middle_stacked(area, &middle_panels)
        }
    }

    fn calculate_middle_single_row(
        &self,
        area: Rect,
        middle_panels: &[ViewMode],
    ) -> Vec<PanelRect> {
        let panel_count = middle_panels.len();
        let percent = 100 / panel_count as u16;

        let mut constraints = vec![Constraint::Percentage(percent); panel_count];
        // Adjust last constraint to take remaining space
        if let Some(last) = constraints.last_mut() {
            *last = Constraint::Min(0);
        }

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        middle_panels
            .iter()
            .enumerate()
            .map(|(idx, &panel)| PanelRect {
                panel,
                rect: chunks[idx],
            })
            .collect()
    }

    fn calculate_middle_two_rows(
        &self,
        area: Rect,
        middle_panels: &[ViewMode],
    ) -> Vec<PanelRect> {
        let has_memory = middle_panels.contains(&ViewMode::Memory);

        if has_memory && middle_panels.len() > 1 {
            // Memory gets first row, others split second row
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

            let mut results = vec![PanelRect {
                panel: ViewMode::Memory,
                rect: rows[0],
            }];

            let other_panels: Vec<ViewMode> = middle_panels
                .iter()
                .filter(|&&p| p != ViewMode::Memory)
                .copied()
                .collect();

            if !other_panels.is_empty() {
                let other_count = other_panels.len();
                let percent = 100 / other_count as u16;
                let mut constraints = vec![Constraint::Percentage(percent); other_count];
                if let Some(last) = constraints.last_mut() {
                    *last = Constraint::Min(0);
                }

                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(constraints)
                    .split(rows[1]);

                for (idx, &panel) in other_panels.iter().enumerate() {
                    results.push(PanelRect {
                        panel,
                        rect: chunks[idx],
                    });
                }
            }

            results
        } else {
            // No memory or only memory: fall back to single row
            self.calculate_middle_single_row(area, middle_panels)
        }
    }

    fn calculate_middle_stacked(
        &self,
        area: Rect,
        middle_panels: &[ViewMode],
    ) -> Vec<PanelRect> {
        let panel_count = middle_panels.len();
        let percent = 100 / panel_count as u16;

        let mut constraints = vec![Constraint::Percentage(percent); panel_count];
        if let Some(last) = constraints.last_mut() {
            *last = Constraint::Min(0);
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        middle_panels
            .iter()
            .enumerate()
            .map(|(idx, &panel)| PanelRect {
                panel,
                rect: chunks[idx],
            })
            .collect()
    }
}

impl Default for MultiPanelLayout {
    fn default() -> Self {
        Self::new()
    }
}
