use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Clear},
    style::{Color, Style, Modifier},
    text::{Line, Span},
};
use crate::collectors::process::ProcessStats;
use crate::collectors::cpu::CpuStats;
use crate::collectors::disk::DiskStats;
use crate::gpu::GpuStats;
use crate::ui::{Theme, state::{DetailPopupState, DetailPopupType, DetailSortField, SortOrder}};

pub fn render_detail_popup(
    frame: &mut Frame,
    area: Rect,
    popup_state: &DetailPopupState,
    popup_type: DetailPopupType,
    processes: &[ProcessStats],
    cpu_stats: Option<&CpuStats>,
    disk_stats: Option<&[DiskStats]>,
    gpu_stats: Option<&[GpuStats]>,
    theme: &Theme,
) {
    // 1. Calculate popup size (80% of screen)
    let popup_width = ((area.width as f32 * 0.8).min(120.0) as u16).max(60);
    let popup_height = ((area.height as f32 * 0.8).min(40.0) as u16).max(20);
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x: area.x + x,
        y: area.y + y,
        width: popup_width,
        height: popup_height,
    };

    // 2. Clear the background to make popup opaque
    frame.render_widget(Clear, popup_area);

    // 3. Dispatch to specific rendering based on popup type
    match popup_type {
        DetailPopupType::Cpu => {
            if let Some(cpu_stats) = cpu_stats {
                render_cpu_popup(frame, popup_area, popup_state, cpu_stats, theme);
            }
        }
        DetailPopupType::Memory => {
            // TODO: Implement Memory popup
            render_process_popup(frame, popup_area, popup_state, DetailPopupType::Memory, processes, theme);
        }
        DetailPopupType::DiskIo => {
            render_process_popup(frame, popup_area, popup_state, DetailPopupType::DiskIo, processes, theme);
        }
        DetailPopupType::Network => {
            render_process_popup(frame, popup_area, popup_state, DetailPopupType::Network, processes, theme);
        }
        DetailPopupType::DiskUsage => {
            if let Some(disk_stats) = disk_stats {
                render_diskusage_popup(frame, popup_area, popup_state, disk_stats, theme);
            }
        }
        DetailPopupType::Gpu => {
            if let Some(gpu_stats) = gpu_stats {
                render_gpu_popup(frame, popup_area, popup_state, gpu_stats, theme);
            }
        }
    }
}

// CPU Popup - Show core details
fn render_cpu_popup(
    frame: &mut Frame,
    popup_area: Rect,
    popup_state: &DetailPopupState,
    cpu_stats: &CpuStats,
    theme: &Theme,
) {
    let cores = &cpu_stats.cores;

    let title = format!(" CPU Core Details ({} cores) ", cores.len());

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let mut lines = Vec::new();

    // Sort indicator
    lines.push(Line::from(vec![
        Span::styled("Sort by: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{} ", popup_state.sort_field.name()),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        ),
        Span::styled(
            match popup_state.sort_order {
                SortOrder::Ascending => "↑",
                SortOrder::Descending => "↓",
            },
            Style::default().fg(Color::Cyan)
        ),
    ]));
    lines.push(Line::from(""));

    // Table header
    lines.push(Line::from(vec![
        Span::styled(format!("{:>4} ", "Core"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:>7} ", "Total%"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:>7} ", "User%"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:>7} ", "Sys%"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:>7} ", "I/O%"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:>7} ", "IRQ%"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:>7}", "Idle%"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
    ]));

    // Sort cores
    let mut sorted_cores: Vec<_> = cores.iter().collect();
    sorted_cores.sort_by(|a, b| {
        use DetailSortField::*;
        let cmp = match popup_state.sort_field {
            CoreId => a.core_id.cmp(&b.core_id),
            CpuTotal => b.usage_percent.partial_cmp(&a.usage_percent).unwrap_or(std::cmp::Ordering::Equal),
            CpuUser => {
                let a_user = calculate_percent(a.user_time, a.user_time + a.system_time + a.idle_time);
                let b_user = calculate_percent(b.user_time, b.user_time + b.system_time + b.idle_time);
                b_user.partial_cmp(&a_user).unwrap_or(std::cmp::Ordering::Equal)
            }
            CpuSystem => {
                let a_sys = calculate_percent(a.system_time, a.user_time + a.system_time + a.idle_time);
                let b_sys = calculate_percent(b.system_time, b.user_time + b.system_time + b.idle_time);
                b_sys.partial_cmp(&a_sys).unwrap_or(std::cmp::Ordering::Equal)
            }
            CpuIoWait => {
                let a_io = calculate_percent(a.iowait_time, a.user_time + a.system_time + a.idle_time);
                let b_io = calculate_percent(b.iowait_time, b.user_time + b.system_time + b.idle_time);
                b_io.partial_cmp(&a_io).unwrap_or(std::cmp::Ordering::Equal)
            }
            CpuIrq => {
                let a_irq = calculate_percent(a.irq_time + a.softirq_time, a.user_time + a.system_time + a.idle_time);
                let b_irq = calculate_percent(b.irq_time + b.softirq_time, b.user_time + b.system_time + b.idle_time);
                b_irq.partial_cmp(&a_irq).unwrap_or(std::cmp::Ordering::Equal)
            }
            _ => a.core_id.cmp(&b.core_id),
        };

        match popup_state.sort_order {
            SortOrder::Ascending => cmp.reverse(),
            SortOrder::Descending => cmp,
        }
    });

    // Calculate visible rows
    let header_lines = 4;
    let footer_lines = 3;
    let visible_rows = (inner.height as usize).saturating_sub(header_lines + footer_lines);

    // Apply scroll and render
    let visible_cores: Vec<_> = sorted_cores
        .iter()
        .skip(popup_state.scroll_offset)
        .take(visible_rows)
        .collect();

    for core in visible_cores {
        let total = core.user_time + core.system_time + core.idle_time;
        let user_pct = calculate_percent(core.user_time, total);
        let sys_pct = calculate_percent(core.system_time, total);
        let io_pct = calculate_percent(core.iowait_time, total);
        let irq_pct = calculate_percent(core.irq_time + core.softirq_time, total);
        let idle_pct = calculate_percent(core.idle_time, total);

        let line = Line::from(vec![
            Span::styled(format!("{:>4} ", core.core_id), Style::default().fg(Color::Cyan)),
            Span::styled(format!("{:>6.1}% ", core.usage_percent), Style::default().fg(theme.cpu_color(core.usage_percent))),
            Span::styled(format!("{:>6.1}% ", user_pct), Style::default().fg(Color::Green)),
            Span::styled(format!("{:>6.1}% ", sys_pct), Style::default().fg(Color::Yellow)),
            Span::styled(format!("{:>6.1}% ", io_pct), Style::default().fg(Color::Magenta)),
            Span::styled(format!("{:>6.1}% ", irq_pct), Style::default().fg(Color::Red)),
            Span::styled(format!("{:>6.1}%", idle_pct), Style::default().fg(Color::DarkGray)),
        ]);
        lines.push(line);
    }

    // Footer
    lines.push(Line::from(""));
    let start = if sorted_cores.is_empty() { 0 } else { popup_state.scroll_offset + 1 };
    let end = (popup_state.scroll_offset + visible_rows).min(sorted_cores.len());
    let scroll_info = format!("Showing {}-{} of {}", start, end, sorted_cores.len());
    lines.push(Line::from(vec![
        Span::styled(scroll_info, Style::default().fg(Color::Cyan)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("j/k:scroll | s:sort | r:reverse | ESC:close", Style::default().fg(Color::DarkGray)),
    ]));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn calculate_percent(value: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (value as f64 / total as f64) * 100.0
    }
}

// Process-based popup (for DiskIO, Network, Memory)
fn render_process_popup(
    frame: &mut Frame,
    popup_area: Rect,
    popup_state: &DetailPopupState,
    popup_type: DetailPopupType,
    processes: &[ProcessStats],
    theme: &Theme,
) {
    // Filter and sort processes
    let filtered_processes = filter_processes(processes, popup_state, popup_type);
    let mut sorted_processes = filtered_processes;
    sort_processes(&mut sorted_processes, popup_state);

    let title = match popup_type {
        DetailPopupType::DiskIo => format!(" Disk I/O Processes ({}) ", sorted_processes.len()),
        DetailPopupType::Network => format!(" Network Processes ({}) ", sorted_processes.len()),
        DetailPopupType::Memory => format!(" Memory Usage by Process ({}) ", sorted_processes.len()),
        _ => String::from(" Processes "),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let mut lines = Vec::new();

    // Sort indicator
    lines.push(Line::from(vec![
        Span::styled("Sort by: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{} ", popup_state.sort_field.name()),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        ),
        Span::styled(
            match popup_state.sort_order {
                SortOrder::Ascending => "↑",
                SortOrder::Descending => "↓",
            },
            Style::default().fg(Color::Cyan)
        ),
    ]));
    lines.push(Line::from(""));

    // Table header
    let header = match popup_type {
        DetailPopupType::DiskIo => Line::from(vec![
            Span::styled(format!("{:>7} ", "PID"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<10} ", "USER"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<25} ", "NAME"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:>6} ", "CPU%"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:>7} ", "MEM%"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:>10} ", "READ MB/s"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:>10}", "WRITE MB/s"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        ]),
        DetailPopupType::Network => Line::from(vec![
            Span::styled(format!("{:>7} ", "PID"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<10} ", "USER"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<35} ", "NAME"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:>6} ", "CPU%"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:>7}", "MEM%"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        ]),
        DetailPopupType::Memory => Line::from(vec![
            Span::styled(format!("{:>7} ", "PID"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<10} ", "USER"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<30} ", "NAME"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:>6} ", "CPU%"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:>7} ", "MEM%"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:>10}", "SIZE MB"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        ]),
        _ => Line::from(""),
    };
    lines.push(header);

    // Calculate visible rows
    let header_lines = 4;
    let footer_lines = 3;
    let visible_rows = (inner.height as usize).saturating_sub(header_lines + footer_lines);

    // Apply scroll offset and render process list
    let visible_processes: Vec<_> = sorted_processes
        .iter()
        .skip(popup_state.scroll_offset)
        .take(visible_rows)
        .collect();

    for process in visible_processes {
        let line = render_process_line(process, popup_type, theme);
        lines.push(line);
    }

    // Footer
    if popup_state.search_mode {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Search: ", Style::default().fg(Color::Yellow)),
            Span::styled(&popup_state.search_text, Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(Color::Cyan)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("ESC to cancel | Enter to apply", Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        lines.push(Line::from(""));
        let start = if sorted_processes.is_empty() { 0 } else { popup_state.scroll_offset + 1 };
        let end = (popup_state.scroll_offset + visible_rows).min(sorted_processes.len());
        let scroll_info = format!("Showing {}-{} of {}", start, end, sorted_processes.len());
        lines.push(Line::from(vec![
            Span::styled(scroll_info, Style::default().fg(Color::Cyan)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("j/k:scroll | s:sort | r:reverse | /:search | ESC:close", Style::default().fg(Color::DarkGray)),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn filter_processes<'a>(
    processes: &'a [ProcessStats],
    popup_state: &DetailPopupState,
    popup_type: DetailPopupType,
) -> Vec<&'a ProcessStats> {
    processes
        .iter()
        .filter(|p| {
            // Filter by type
            let type_match = match popup_type {
                DetailPopupType::DiskIo => {
                    p.io_read_rate > 0.0 || p.io_write_rate > 0.0
                }
                DetailPopupType::Network => {
                    p.cpu_percent > 0.5
                }
                DetailPopupType::Memory => {
                    p.memory_percent > 0.1
                }
                _ => true,
            };

            if !type_match {
                return false;
            }

            // Search filter
            if popup_state.search_text.is_empty() {
                true
            } else {
                let search_lower = popup_state.search_text.to_lowercase();
                p.name.to_lowercase().contains(&search_lower)
                    || p.cmdline.to_lowercase().contains(&search_lower)
                    || p.user.to_lowercase().contains(&search_lower)
            }
        })
        .collect()
}

fn sort_processes(
    processes: &mut Vec<&ProcessStats>,
    popup_state: &DetailPopupState,
) {
    processes.sort_by(|a, b| {
        use DetailSortField::*;

        let cmp = match popup_state.sort_field {
            Pid => a.pid.cmp(&b.pid),
            Name => a.name.cmp(&b.name),
            User => a.user.cmp(&b.user),
            Cpu => b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal),
            Memory | MemorySize => b.memory_percent.partial_cmp(&a.memory_percent).unwrap_or(std::cmp::Ordering::Equal),
            IoRead => b.io_read_rate.partial_cmp(&a.io_read_rate).unwrap_or(std::cmp::Ordering::Equal),
            IoWrite => b.io_write_rate.partial_cmp(&a.io_write_rate).unwrap_or(std::cmp::Ordering::Equal),
            IoTotal => {
                let a_total = a.io_read_rate + a.io_write_rate;
                let b_total = b.io_read_rate + b.io_write_rate;
                b_total.partial_cmp(&a_total).unwrap_or(std::cmp::Ordering::Equal)
            }
            _ => std::cmp::Ordering::Equal,
        };

        match popup_state.sort_order {
            SortOrder::Ascending => cmp.reverse(),
            SortOrder::Descending => cmp,
        }
    });
}

fn render_process_line(
    process: &ProcessStats,
    popup_type: DetailPopupType,
    theme: &Theme,
) -> Line<'static> {
    let process_name = if process.name.len() > 25 {
        format!("{}...", &process.name[..22])
    } else {
        process.name.clone()
    };

    match popup_type {
        DetailPopupType::DiskIo => {
            let read_mb = process.io_read_rate / 1024.0 / 1024.0;
            let write_mb = process.io_write_rate / 1024.0 / 1024.0;
            let total_mb = read_mb + write_mb;

            let io_color = if total_mb > 10.0 {
                Color::Red
            } else if total_mb > 1.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            Line::from(vec![
                Span::styled(format!("{:>7} ", process.pid), Style::default().fg(Color::Cyan)),
                Span::styled(format!("{:<10} ", truncate_str(&process.user, 10)), Style::default().fg(Color::Gray)),
                Span::styled(format!("{:<25} ", process_name), Style::default().fg(Color::White)),
                Span::styled(format!("{:>6.1} ", process.cpu_percent), Style::default().fg(theme.cpu_color(process.cpu_percent))),
                Span::styled(format!("{:>6.1}% ", process.memory_percent), Style::default().fg(theme.mem_color(process.memory_percent))),
                Span::styled(format!("{:>10.2} ", read_mb), Style::default().fg(io_color)),
                Span::styled(format!("{:>10.2}", write_mb), Style::default().fg(io_color)),
            ])
        }
        DetailPopupType::Network => {
            let name_for_display = if process.name.len() > 35 {
                format!("{}...", &process.name[..32])
            } else {
                process.name.clone()
            };

            Line::from(vec![
                Span::styled(format!("{:>7} ", process.pid), Style::default().fg(Color::Cyan)),
                Span::styled(format!("{:<10} ", truncate_str(&process.user, 10)), Style::default().fg(Color::Gray)),
                Span::styled(format!("{:<35} ", name_for_display), Style::default().fg(Color::White)),
                Span::styled(format!("{:>6.1} ", process.cpu_percent), Style::default().fg(theme.cpu_color(process.cpu_percent))),
                Span::styled(format!("{:>6.1}%", process.memory_percent), Style::default().fg(theme.mem_color(process.memory_percent))),
            ])
        }
        DetailPopupType::Memory => {
            let name_for_display = if process.name.len() > 30 {
                format!("{}...", &process.name[..27])
            } else {
                process.name.clone()
            };
            let size_mb = process.memory_kb as f64 / 1024.0;

            Line::from(vec![
                Span::styled(format!("{:>7} ", process.pid), Style::default().fg(Color::Cyan)),
                Span::styled(format!("{:<10} ", truncate_str(&process.user, 10)), Style::default().fg(Color::Gray)),
                Span::styled(format!("{:<30} ", name_for_display), Style::default().fg(Color::White)),
                Span::styled(format!("{:>6.1} ", process.cpu_percent), Style::default().fg(theme.cpu_color(process.cpu_percent))),
                Span::styled(format!("{:>6.1}% ", process.memory_percent), Style::default().fg(theme.mem_color(process.memory_percent))),
                Span::styled(format!("{:>9.1}", size_mb), Style::default().fg(Color::Yellow)),
            ])
        }
        _ => Line::from(""),
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

// DiskUsage Popup - Show mount point details
fn render_diskusage_popup(
    frame: &mut Frame,
    popup_area: Rect,
    popup_state: &DetailPopupState,
    disk_stats: &[DiskStats],
    theme: &Theme,
) {
    // Filter and sort disk stats
    let filtered_disks = filter_disk_stats(disk_stats, popup_state);
    let mut sorted_disks = filtered_disks;
    sort_disk_stats(&mut sorted_disks, popup_state);

    let title = format!(" Disk Usage Details ({} mounts) ", sorted_disks.len());

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let mut lines = Vec::new();

    // Sort indicator
    lines.push(Line::from(vec![
        Span::styled("Sort by: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{} ", popup_state.sort_field.name()),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        ),
        Span::styled(
            match popup_state.sort_order {
                SortOrder::Ascending => "↑",
                SortOrder::Descending => "↓",
            },
            Style::default().fg(Color::Cyan)
        ),
    ]));
    lines.push(Line::from(""));

    // Table header
    lines.push(Line::from(vec![
        Span::styled(format!("{:<25} ", "MOUNT POINT"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:<15} ", "DEVICE"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:<8} ", "TYPE"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:>7} ", "USED%"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:>10} ", "USED GB"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:>10}", "AVAIL GB"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
    ]));

    // Calculate visible rows
    let header_lines = 4;
    let footer_lines = 3;
    let visible_rows = (inner.height as usize).saturating_sub(header_lines + footer_lines);

    // Apply scroll and render
    let visible_disks: Vec<_> = sorted_disks
        .iter()
        .skip(popup_state.scroll_offset)
        .take(visible_rows)
        .collect();

    for disk in visible_disks {
        let used_gb = disk.used_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
        let avail_gb = disk.available_bytes as f64 / 1024.0 / 1024.0 / 1024.0;

        let usage_color = if disk.usage_percent > 90.0 {
            Color::Red
        } else if disk.usage_percent > 75.0 {
            Color::Yellow
        } else {
            Color::Green
        };

        let mount_point = truncate_str(&disk.mount_point, 25);
        let device = truncate_str(&disk.device, 15);
        let fs_type = truncate_str(&disk.fs_type, 8);

        let line = Line::from(vec![
            Span::styled(format!("{:<25} ", mount_point), Style::default().fg(Color::Cyan)),
            Span::styled(format!("{:<15} ", device), Style::default().fg(Color::Gray)),
            Span::styled(format!("{:<8} ", fs_type), Style::default().fg(Color::White)),
            Span::styled(format!("{:>6.1}% ", disk.usage_percent), Style::default().fg(usage_color)),
            Span::styled(format!("{:>10.2} ", used_gb), Style::default().fg(Color::Yellow)),
            Span::styled(format!("{:>10.2}", avail_gb), Style::default().fg(Color::Green)),
        ]);
        lines.push(line);
    }

    // Footer
    if popup_state.search_mode {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Search: ", Style::default().fg(Color::Yellow)),
            Span::styled(&popup_state.search_text, Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(Color::Cyan)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("ESC to cancel | Enter to apply", Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        lines.push(Line::from(""));
        let start = if sorted_disks.is_empty() { 0 } else { popup_state.scroll_offset + 1 };
        let end = (popup_state.scroll_offset + visible_rows).min(sorted_disks.len());
        let scroll_info = format!("Showing {}-{} of {}", start, end, sorted_disks.len());
        lines.push(Line::from(vec![
            Span::styled(scroll_info, Style::default().fg(Color::Cyan)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("j/k:scroll | s:sort | r:reverse | /:search | ESC:close", Style::default().fg(Color::DarkGray)),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn filter_disk_stats<'a>(
    disk_stats: &'a [DiskStats],
    popup_state: &DetailPopupState,
) -> Vec<&'a DiskStats> {
    disk_stats
        .iter()
        .filter(|d| {
            if popup_state.search_text.is_empty() {
                true
            } else {
                let search_lower = popup_state.search_text.to_lowercase();
                d.mount_point.to_lowercase().contains(&search_lower)
                    || d.device.to_lowercase().contains(&search_lower)
                    || d.fs_type.to_lowercase().contains(&search_lower)
            }
        })
        .collect()
}

fn sort_disk_stats(
    disk_stats: &mut Vec<&DiskStats>,
    popup_state: &DetailPopupState,
) {
    disk_stats.sort_by(|a, b| {
        use DetailSortField::*;

        let cmp = match popup_state.sort_field {
            MountPoint => a.mount_point.cmp(&b.mount_point),
            DiskUsage => b.usage_percent.partial_cmp(&a.usage_percent).unwrap_or(std::cmp::Ordering::Equal),
            DiskUsed => b.used_bytes.cmp(&a.used_bytes),
            DiskAvailable => b.available_bytes.cmp(&a.available_bytes),
            FsType => a.fs_type.cmp(&b.fs_type),
            _ => a.mount_point.cmp(&b.mount_point),
        };

        match popup_state.sort_order {
            SortOrder::Ascending => cmp.reverse(),
            SortOrder::Descending => cmp,
        }
    });
}

// GPU Popup - Show GPU details
fn render_gpu_popup(
    frame: &mut Frame,
    popup_area: Rect,
    popup_state: &DetailPopupState,
    gpu_stats: &[GpuStats],
    theme: &Theme,
) {
    // Filter and sort GPU stats
    let filtered_gpus = filter_gpu_stats(gpu_stats, popup_state);
    let mut sorted_gpus = filtered_gpus;
    sort_gpu_stats(&mut sorted_gpus, popup_state);

    let title = format!(" GPU Details ({} GPUs) ", sorted_gpus.len());

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let mut lines = Vec::new();

    // Sort indicator
    lines.push(Line::from(vec![
        Span::styled("Sort by: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{} ", popup_state.sort_field.name()),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        ),
        Span::styled(
            match popup_state.sort_order {
                SortOrder::Ascending => "↑",
                SortOrder::Descending => "↓",
            },
            Style::default().fg(Color::Cyan)
        ),
    ]));
    lines.push(Line::from(""));

    // Table header
    lines.push(Line::from(vec![
        Span::styled(format!("{:>3} ", "ID"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:<25} ", "NAME"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:<10} ", "VENDOR"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:>7} ", "UTIL%"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:>10} ", "VRAM"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:>8} ", "TEMP°C"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{:>8}", "POWER W"), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)),
    ]));

    // Calculate visible rows
    let header_lines = 4;
    let footer_lines = 3;
    let visible_rows = (inner.height as usize).saturating_sub(header_lines + footer_lines);

    // Apply scroll and render
    let visible_gpus: Vec<_> = sorted_gpus
        .iter()
        .enumerate()
        .skip(popup_state.scroll_offset)
        .take(visible_rows)
        .collect();

    for (gpu_id, gpu) in visible_gpus {
        let vram_percent = if gpu.memory_total_mb > 0 {
            (gpu.memory_used_mb as f64 / gpu.memory_total_mb as f64) * 100.0
        } else {
            0.0
        };

        let util_color = if gpu.utilization_percent > 90.0 {
            Color::Red
        } else if gpu.utilization_percent > 70.0 {
            Color::Yellow
        } else {
            Color::Green
        };

        let temp_color = if gpu.temperature_c > 80.0 {
            Color::Red
        } else if gpu.temperature_c > 70.0 {
            Color::Yellow
        } else {
            Color::Green
        };

        let name = truncate_str(&gpu.name, 25);
        let vendor = truncate_str(&gpu.vendor, 10);
        let vram_text = format!("{}/{} MB ({:.0}%)", gpu.memory_used_mb, gpu.memory_total_mb, vram_percent);

        let line = Line::from(vec![
            Span::styled(format!("{:>3} ", gpu_id), Style::default().fg(Color::Cyan)),
            Span::styled(format!("{:<25} ", name), Style::default().fg(Color::White)),
            Span::styled(format!("{:<10} ", vendor), Style::default().fg(Color::Gray)),
            Span::styled(format!("{:>6.1}% ", gpu.utilization_percent), Style::default().fg(util_color)),
            Span::styled(format!("{:<10} ", truncate_str(&vram_text, 10)), Style::default().fg(theme.mem_color(vram_percent))),
            Span::styled(format!("{:>7.1}° ", gpu.temperature_c), Style::default().fg(temp_color)),
            Span::styled(format!("{:>7.1}", gpu.power_watts), Style::default().fg(Color::Yellow)),
        ]);
        lines.push(line);
    }

    // Footer
    if popup_state.search_mode {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Search: ", Style::default().fg(Color::Yellow)),
            Span::styled(&popup_state.search_text, Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(Color::Cyan)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("ESC to cancel | Enter to apply", Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        lines.push(Line::from(""));
        let start = if sorted_gpus.is_empty() { 0 } else { popup_state.scroll_offset + 1 };
        let end = (popup_state.scroll_offset + visible_rows).min(sorted_gpus.len());
        let scroll_info = format!("Showing {}-{} of {}", start, end, sorted_gpus.len());
        lines.push(Line::from(vec![
            Span::styled(scroll_info, Style::default().fg(Color::Cyan)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("j/k:scroll | s:sort | r:reverse | /:search | ESC:close", Style::default().fg(Color::DarkGray)),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn filter_gpu_stats<'a>(
    gpu_stats: &'a [GpuStats],
    popup_state: &DetailPopupState,
) -> Vec<&'a GpuStats> {
    gpu_stats
        .iter()
        .filter(|g| {
            if popup_state.search_text.is_empty() {
                true
            } else {
                let search_lower = popup_state.search_text.to_lowercase();
                g.name.to_lowercase().contains(&search_lower)
                    || g.vendor.to_lowercase().contains(&search_lower)
            }
        })
        .collect()
}

fn sort_gpu_stats(
    gpu_stats: &mut Vec<&GpuStats>,
    popup_state: &DetailPopupState,
) {
    gpu_stats.sort_by(|a, b| {
        use DetailSortField::*;

        let cmp = match popup_state.sort_field {
            GpuId => std::cmp::Ordering::Equal, // Will be handled by enumerate
            GpuUtil => b.utilization_percent.partial_cmp(&a.utilization_percent).unwrap_or(std::cmp::Ordering::Equal),
            GpuVram => {
                let a_vram_pct = if a.memory_total_mb > 0 { (a.memory_used_mb as f64 / a.memory_total_mb as f64) * 100.0 } else { 0.0 };
                let b_vram_pct = if b.memory_total_mb > 0 { (b.memory_used_mb as f64 / b.memory_total_mb as f64) * 100.0 } else { 0.0 };
                b_vram_pct.partial_cmp(&a_vram_pct).unwrap_or(std::cmp::Ordering::Equal)
            }
            GpuTemp => b.temperature_c.partial_cmp(&a.temperature_c).unwrap_or(std::cmp::Ordering::Equal),
            GpuPower => b.power_watts.partial_cmp(&a.power_watts).unwrap_or(std::cmp::Ordering::Equal),
            _ => std::cmp::Ordering::Equal,
        };

        match popup_state.sort_order {
            SortOrder::Ascending => cmp.reverse(),
            SortOrder::Descending => cmp,
        }
    });
}

// Placeholder for unimplemented popup types
fn render_placeholder_popup(
    frame: &mut Frame,
    popup_area: Rect,
    title: &str,
) {
    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("This feature is coming soon!", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC to close", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
