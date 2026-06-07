// Application state management for interactive UI
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViewMode {
    Cpu,
    Memory,
    Processes,
    Network,
    DiskIo,
    DiskUsage,
    Gpu,
    GpuProcesses,
}

impl ViewMode {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(ViewMode::Cpu),
            1 => Some(ViewMode::Memory),
            2 => Some(ViewMode::Processes),
            3 => Some(ViewMode::Network),
            4 => Some(ViewMode::DiskIo),
            5 => Some(ViewMode::DiskUsage),
            6 => Some(ViewMode::Gpu),
            7 => Some(ViewMode::GpuProcesses),
            _ => None,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            ViewMode::Cpu => 0,
            ViewMode::Memory => 1,
            ViewMode::Processes => 2,
            ViewMode::Network => 3,
            ViewMode::DiskIo => 4,
            ViewMode::DiskUsage => 5,
            ViewMode::Gpu => 6,
            ViewMode::GpuProcesses => 7,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ViewMode::Cpu => "CPU",
            ViewMode::Memory => "Memory",
            ViewMode::Processes => "Processes",
            ViewMode::Network => "Network",
            ViewMode::DiskIo => "Disk I/O",
            ViewMode::DiskUsage => "Disk Usage",
            ViewMode::Gpu => "GPU",
            ViewMode::GpuProcesses => "GPU Processes",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            ViewMode::Cpu => ViewMode::Memory,
            ViewMode::Memory => ViewMode::Processes,
            ViewMode::Processes => ViewMode::Network,
            ViewMode::Network => ViewMode::DiskIo,
            ViewMode::DiskIo => ViewMode::DiskUsage,
            ViewMode::DiskUsage => ViewMode::Gpu,
            ViewMode::Gpu => ViewMode::GpuProcesses,
            ViewMode::GpuProcesses => ViewMode::Cpu,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            ViewMode::Cpu => ViewMode::GpuProcesses,
            ViewMode::Memory => ViewMode::Cpu,
            ViewMode::Processes => ViewMode::Memory,
            ViewMode::Network => ViewMode::Processes,
            ViewMode::DiskIo => ViewMode::Network,
            ViewMode::DiskUsage => ViewMode::DiskIo,
            ViewMode::Gpu => ViewMode::DiskUsage,
            ViewMode::GpuProcesses => ViewMode::Gpu,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Pid,
    Name,
    Cpu,
    Memory,
}

impl SortField {
    pub fn next(&self) -> Self {
        match self {
            SortField::Pid => SortField::Name,
            SortField::Name => SortField::Cpu,
            SortField::Cpu => SortField::Memory,
            SortField::Memory => SortField::Pid,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            SortField::Pid => SortField::Memory,
            SortField::Name => SortField::Pid,
            SortField::Cpu => SortField::Name,
            SortField::Memory => SortField::Cpu,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            SortField::Pid => "PID",
            SortField::Name => "Name",
            SortField::Cpu => "CPU%",
            SortField::Memory => "MEM%",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn toggle(&self) -> Self {
        match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }
}

/// Detail popup sort field options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailSortField {
    // Common fields
    Pid,
    Name,
    User,

    // CPU-specific fields
    CoreId,
    CpuTotal,
    CpuUser,
    CpuSystem,
    CpuIoWait,
    CpuIrq,

    // Memory/Process fields
    Cpu,
    Memory,
    MemorySize,

    // I/O fields
    IoRead,
    IoWrite,
    IoTotal,

    // Disk fields
    MountPoint,
    DiskUsage,
    DiskUsed,
    DiskAvailable,
    FsType,

    // GPU fields
    GpuId,
    GpuUtil,
    GpuVram,
    GpuTemp,
    GpuPower,
}

impl DetailSortField {
    pub fn next(&self, popup_type: DetailPopupType) -> Self {
        use DetailSortField::*;
        match popup_type {
            DetailPopupType::Cpu => match self {
                CoreId => CpuTotal,
                CpuTotal => CpuUser,
                CpuUser => CpuSystem,
                CpuSystem => CpuIoWait,
                CpuIoWait => CpuIrq,
                CpuIrq => CoreId,
                _ => CoreId,
            },
            DetailPopupType::Memory => match self {
                Pid => Name,
                Name => User,
                User => Memory,
                Memory => MemorySize,
                MemorySize => Cpu,
                Cpu => Pid,
                _ => Pid,
            },
            DetailPopupType::Process => match self {
                Pid => User,
                User => Name,
                Name => Cpu,
                Cpu => Memory,
                Memory => MemorySize,
                MemorySize => Pid,
                _ => Pid,
            },
            DetailPopupType::DiskIo => match self {
                Pid => Name,
                Name => User,
                User => Cpu,
                Cpu => Memory,
                Memory => IoRead,
                IoRead => IoWrite,
                IoWrite => IoTotal,
                IoTotal => Pid,
                _ => Pid,
            },
            DetailPopupType::Network => match self {
                Pid => Name,
                Name => User,
                User => Cpu,
                Cpu => Pid,
                _ => Pid,
            },
            DetailPopupType::DiskUsage => match self {
                MountPoint => DiskUsage,
                DiskUsage => DiskUsed,
                DiskUsed => DiskAvailable,
                DiskAvailable => FsType,
                FsType => MountPoint,
                _ => MountPoint,
            },
            DetailPopupType::Gpu => match self {
                GpuId => GpuUtil,
                GpuUtil => GpuVram,
                GpuVram => GpuTemp,
                GpuTemp => GpuPower,
                GpuPower => GpuId,
                _ => GpuId,
            },
            DetailPopupType::GpuProcesses => match self {
                Pid => Name,
                Name => GpuVram,
                GpuVram => GpuUtil,
                GpuUtil => Pid,
                _ => Pid,
            },
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            DetailSortField::Pid => "PID",
            DetailSortField::Name => "Name",
            DetailSortField::User => "User",
            DetailSortField::CoreId => "Core ID",
            DetailSortField::CpuTotal => "Total%",
            DetailSortField::CpuUser => "User%",
            DetailSortField::CpuSystem => "System%",
            DetailSortField::CpuIoWait => "I/O Wait%",
            DetailSortField::CpuIrq => "IRQ%",
            DetailSortField::Cpu => "CPU%",
            DetailSortField::Memory => "Memory%",
            DetailSortField::MemorySize => "Size MB",
            DetailSortField::IoRead => "I/O Read",
            DetailSortField::IoWrite => "I/O Write",
            DetailSortField::IoTotal => "I/O Total",
            DetailSortField::MountPoint => "Mount Point",
            DetailSortField::DiskUsage => "Usage%",
            DetailSortField::DiskUsed => "Used GB",
            DetailSortField::DiskAvailable => "Available GB",
            DetailSortField::FsType => "FS Type",
            DetailSortField::GpuId => "GPU ID",
            DetailSortField::GpuUtil => "Utilization%",
            DetailSortField::GpuVram => "VRAM%",
            DetailSortField::GpuTemp => "Temperature°C",
            DetailSortField::GpuPower => "Power W",
        }
    }
}

/// Type of detail popup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailPopupType {
    Cpu,
    Memory,
    Process,
    DiskIo,
    Network,
    DiskUsage,
    Gpu,
    GpuProcesses,
}

/// State for detail popup window
#[derive(Clone)]
pub struct DetailPopupState {
    pub scroll_offset: usize,
    pub search_text: String,
    pub search_mode: bool,
    pub sort_field: DetailSortField,
    pub sort_order: SortOrder,
    pub show_full_command: bool,
    pub selected_index: Option<usize>,
}

impl DetailPopupState {
    pub fn new(popup_type: DetailPopupType) -> Self {
        Self {
            scroll_offset: 0,
            search_text: String::new(),
            search_mode: false,
            sort_field: match popup_type {
                DetailPopupType::Cpu => DetailSortField::CpuTotal,
                DetailPopupType::Memory => DetailSortField::Memory,
                DetailPopupType::Process => DetailSortField::Cpu,
                DetailPopupType::DiskIo => DetailSortField::IoTotal,
                DetailPopupType::Network => DetailSortField::Cpu,
                DetailPopupType::DiskUsage => DetailSortField::DiskUsage,
                DetailPopupType::Gpu => DetailSortField::GpuUtil,
                DetailPopupType::GpuProcesses => DetailSortField::GpuVram,
            },
            sort_order: SortOrder::Descending,
            show_full_command: false,
            selected_index: None,
        }
    }

    pub fn scroll_up(&mut self, count: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(count);
    }

    pub fn scroll_down(&mut self, count: usize, max: usize) {
        self.scroll_offset = (self.scroll_offset + count).min(max);
    }

    pub fn toggle_search(&mut self) {
        self.search_mode = !self.search_mode;
        if !self.search_mode {
            self.search_text.clear();
        }
    }

    pub fn toggle_full_command(&mut self) {
        self.show_full_command = !self.show_full_command;
    }

    pub fn select_next(&mut self, max_len: usize, visible_height: usize) {
        if let Some(idx) = self.selected_index {
            if idx + 1 < max_len {
                self.selected_index = Some(idx + 1);
                // Auto-scroll if needed
                if idx + 1 >= self.scroll_offset + visible_height {
                    self.scroll_offset = (idx + 2).saturating_sub(visible_height);
                }
            }
        } else if max_len > 0 {
            self.selected_index = Some(0);
        }
    }

    pub fn select_prev(&mut self, visible_height: usize) {
        if let Some(idx) = self.selected_index {
            if idx > 0 {
                self.selected_index = Some(idx - 1);
                // Auto-scroll if needed
                if idx - 1 < self.scroll_offset {
                    self.scroll_offset = idx - 1;
                }
            }
        }
    }

    pub fn get_selected_index(&self) -> Option<usize> {
        self.selected_index
    }
}

#[derive(Clone)]
pub struct AppState {
    pub visible_panels: BTreeSet<ViewMode>,
    pub active_panel: ViewMode,
    pub filter_active: bool,
    pub filter_text: String,
    pub modal_active: bool,
    pub selected_process_index: Option<usize>,
    pub selected_process_pid: Option<u32>,
    pub help_visible: bool,
    pub sort_field: SortField,
    pub sort_order: SortOrder,
    pub detail_mode: BTreeSet<ViewMode>,
    pub show_full_command: bool,
    pub detail_popup: Option<DetailPopupState>,
    pub detail_popup_type: DetailPopupType,
    pub opened_by_key: Option<char>,
    pub popup_modal_active: bool,
    pub popup_selected_process_pid: Option<u32>,
}

impl AppState {
    pub fn new() -> Self {
        // Default: all panels visible
        let mut visible_panels = BTreeSet::new();
        visible_panels.insert(ViewMode::Cpu);
        visible_panels.insert(ViewMode::Memory);
        visible_panels.insert(ViewMode::Processes);
        visible_panels.insert(ViewMode::Network);
        visible_panels.insert(ViewMode::DiskIo);
        visible_panels.insert(ViewMode::DiskUsage);
        visible_panels.insert(ViewMode::Gpu);
        visible_panels.insert(ViewMode::GpuProcesses);

        Self {
            visible_panels,
            active_panel: ViewMode::Cpu,
            filter_active: false,
            filter_text: String::new(),
            modal_active: false,
            selected_process_index: None,
            selected_process_pid: None,
            help_visible: false,
            sort_field: SortField::Cpu,
            sort_order: SortOrder::Descending,
            detail_mode: BTreeSet::new(),
            show_full_command: false,
            detail_popup: None,
            detail_popup_type: DetailPopupType::DiskIo,
            opened_by_key: None,
            popup_modal_active: false,
            popup_selected_process_pid: None,
        }
    }

    /// Toggle panel visibility (returns false if cannot hide last panel)
    pub fn toggle_panel(&mut self, panel: ViewMode) -> bool {
        if self.visible_panels.contains(&panel) {
            if self.visible_panels.len() > 1 {
                self.visible_panels.remove(&panel);

                // If hiding active panel, switch to another
                if self.active_panel == panel {
                    self.active_panel = *self.visible_panels.iter().next().unwrap();
                }
                true
            } else {
                false // Cannot hide last panel
            }
        } else {
            self.visible_panels.insert(panel);
            true
        }
    }

    pub fn is_panel_visible(&self, panel: ViewMode) -> bool {
        self.visible_panels.contains(&panel)
    }

    pub fn switch_view(&mut self, view: ViewMode) {
        if self.visible_panels.contains(&view) {
            self.active_panel = view;
        }
    }

    pub fn next_view(&mut self) {
        let visible: Vec<ViewMode> = self.visible_panels.iter().copied().collect();
        if let Some(idx) = visible.iter().position(|&p| p == self.active_panel) {
            let next_idx = (idx + 1) % visible.len();
            self.active_panel = visible[next_idx];
        }
    }

    pub fn prev_view(&mut self) {
        let visible: Vec<ViewMode> = self.visible_panels.iter().copied().collect();
        if let Some(idx) = visible.iter().position(|&p| p == self.active_panel) {
            let prev_idx = if idx == 0 { visible.len() - 1 } else { idx - 1 };
            self.active_panel = visible[prev_idx];
        }
    }

    pub fn toggle_filter(&mut self) {
        self.filter_active = !self.filter_active;
        if !self.filter_active {
            self.filter_text.clear();
        }
    }

    pub fn add_filter_char(&mut self, c: char) {
        if self.filter_active {
            self.filter_text.push(c);
        }
    }

    pub fn remove_filter_char(&mut self) {
        if self.filter_active {
            self.filter_text.pop();
        }
    }

    pub fn toggle_modal(&mut self) {
        self.modal_active = !self.modal_active;
        if !self.modal_active {
            self.selected_process_pid = None;
        }
    }

    pub fn toggle_help(&mut self) {
        self.help_visible = !self.help_visible;
    }

    pub fn next_sort_field(&mut self) {
        self.sort_field = self.sort_field.next();
    }

    pub fn prev_sort_field(&mut self) {
        self.sort_field = self.sort_field.prev();
    }

    pub fn toggle_sort_order(&mut self) {
        self.sort_order = self.sort_order.toggle();
    }

    pub fn toggle_detail_mode(&mut self, panel: ViewMode) {
        if self.detail_mode.contains(&panel) {
            self.detail_mode.remove(&panel);
        } else {
            self.detail_mode.insert(panel);
        }
    }

    pub fn is_detail_mode(&self, panel: ViewMode) -> bool {
        self.detail_mode.contains(&panel)
    }

    pub fn toggle_full_command(&mut self) {
        self.show_full_command = !self.show_full_command;
    }

    pub fn is_full_command(&self) -> bool {
        self.show_full_command
    }

    pub fn open_detail_popup(&mut self, popup_type: DetailPopupType, key: Option<char>) {
        self.detail_popup = Some(DetailPopupState::new(popup_type));
        self.detail_popup_type = popup_type;
        self.opened_by_key = key;
    }

    pub fn close_detail_popup(&mut self) {
        self.detail_popup = None;
        self.opened_by_key = None;
    }

    pub fn is_detail_popup_open(&self) -> bool {
        self.detail_popup.is_some()
    }

    pub fn open_popup_modal(&mut self, pid: u32) {
        self.popup_modal_active = true;
        self.popup_selected_process_pid = Some(pid);
    }

    pub fn close_popup_modal(&mut self) {
        self.popup_modal_active = false;
        self.popup_selected_process_pid = None;
    }

    /// Handle 'q' or 'ESC' key - close windows in priority order
    /// Returns true if a window was closed, false if should quit app
    pub fn handle_close_key(&mut self) -> bool {
        if self.help_visible {
            self.toggle_help();
            true
        } else if self.popup_modal_active {
            self.close_popup_modal();
            true
        } else if self.is_detail_popup_open() {
            self.close_detail_popup();
            true
        } else if self.modal_active {
            self.toggle_modal();
            true
        } else if self.filter_active {
            self.toggle_filter();
            true
        } else {
            // No windows open, should quit
            false
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
