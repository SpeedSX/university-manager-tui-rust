use crate::models::{Faculty, Student, Teacher};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, List, ListItem, Paragraph, Row, Table, TableState, Tabs,
    },
    Frame,
};

// Current active tab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Students,
    Teachers,
    Faculties,
}

impl ActiveTab {
    pub fn to_string(&self) -> &'static str {
        match self {
            ActiveTab::Students => "Students",
            ActiveTab::Teachers => "Teachers",
            ActiveTab::Faculties => "Faculties",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            ActiveTab::Students => ActiveTab::Teachers,
            ActiveTab::Teachers => ActiveTab::Faculties,
            ActiveTab::Faculties => ActiveTab::Students,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            ActiveTab::Students => ActiveTab::Faculties,
            ActiveTab::Teachers => ActiveTab::Students,
            ActiveTab::Faculties => ActiveTab::Teachers,
        }
    }
}

// App state structure
pub struct AppState {
    pub active_tab: ActiveTab,
    pub student_list_state: TableState,
    pub teacher_list_state: TableState,
    pub faculty_list_state: TableState,
    pub search_query: String,
    pub show_help: bool,
    pub notification: Option<String>,
    pub notification_timer: u16,
}

impl Default for AppState {
    fn default() -> Self {
        let mut student_list_state = TableState::default();
        student_list_state.select(Some(0));

        let mut teacher_list_state = TableState::default();
        teacher_list_state.select(Some(0));

        let mut faculty_list_state = TableState::default();
        faculty_list_state.select(Some(0));

        Self {
            active_tab: ActiveTab::Students,
            student_list_state,
            teacher_list_state,
            faculty_list_state,
            search_query: String::new(),
            show_help: false,
            notification: None,
            notification_timer: 0,
        }
    }
}

impl AppState {
    pub fn select_next(&mut self) {
        let state = self.get_current_table_state();
        let next = match state.selected() {
            Some(i) => Some(i + 1),
            None => Some(0),
        };
        state.select(next);
    }

    pub fn select_previous(&mut self) {
        let state = self.get_current_table_state();
        let prev = match state.selected() {
            Some(i) => {
                if i == 0 {
                    Some(0)
                } else {
                    Some(i - 1)
                }
            }
            None => Some(0),
        };
        state.select(prev);
    }

    pub fn unselect(&mut self) {
        self.get_current_table_state().select(None);
    }

    pub fn select_first(&mut self) {
        self.get_current_table_state().select(Some(0));
    }

    pub fn get_current_table_state(&mut self) -> &mut TableState {
        match self.active_tab {
            ActiveTab::Students => &mut self.student_list_state,
            ActiveTab::Teachers => &mut self.teacher_list_state,
            ActiveTab::Faculties => &mut self.faculty_list_state,
        }
    }

    pub fn show_notification(&mut self, message: String) {
        self.notification = Some(message);
        self.notification_timer = 30; // Show notification for 3 seconds at 10 ticks/second
    }

    pub fn update_notification_timer(&mut self) {
        if self.notification_timer > 0 {
            self.notification_timer -= 1;
            if self.notification_timer == 0 {
                self.notification = None;
            }
        }
    }
}

// Define the types of UI elements that can be clicked
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiElement {
    Tab(ActiveTab),
    TableRow(usize),
    ActionButton(ActionButton),
    ModalButton(ModalButton),
    None,
}

// Types of action buttons in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionButton {
    Add,
    Edit,
    Delete,
    Search,
    Refresh,
}

// Modal buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalButton {
    Confirm,
    Cancel,
}

// Determine which UI element is at a specific position
pub fn get_element_at_position(
    position: (u16, u16),
    active_tab: ActiveTab,
    data_manager: &crate::data_manager::DataManager,
    app_state: &mut AppState,
) -> UiElement {
    let (x, y) = position;
    
    // Get terminal size to calculate proportional positions
    let terminal_size = crossterm::terminal::size().unwrap_or((80, 24));
    let terminal_width = terminal_size.0;
    
    // Tab handling - first 3 rows - adjusted with better calculation
    if y <= 2 {
        // For tabs, use exact divisions - each tab is exactly 1/3 of the width
        let tab_width = terminal_width / 3;
        
        if x < tab_width {
            return UiElement::Tab(ActiveTab::Students);
        } else if x < tab_width * 2 {
            return UiElement::Tab(ActiveTab::Teachers);
        } else {
            return UiElement::Tab(ActiveTab::Faculties);
        }
    }
    
    let terminal_height = terminal_size.1;
    
    // Action buttons - near bottom of screen
    let action_bar_row = terminal_height - 4; // One row for footer, plus action bar height
    
    // Check if clicking on action buttons row
    if y >= action_bar_row && y <= action_bar_row + 2 {
        // Match the actual rendering constraints in the render_action_bar function
        let total_width = terminal_width - 2; // Account for borders
        
        // Calculate button boundaries based on percentages from render_action_bar
        let add_width = total_width * 15 / 100;
        let edit_width = total_width * 15 / 100;
        let delete_width = total_width * 15 / 100;
        let search_width = total_width * 25 / 100;
        
        // Calculate the cumulative positions
        let add_end = 1 + add_width; // +1 for left border
        let edit_end = add_end + edit_width;
        let delete_end = edit_end + delete_width;
        let search_end = delete_end + search_width;
        
        // Check which button was clicked based on the adjusted positions
        if x < add_end {
            return UiElement::ActionButton(ActionButton::Add);
        } else if x < edit_end {
            return UiElement::ActionButton(ActionButton::Edit);
        } else if x < delete_end {
            return UiElement::ActionButton(ActionButton::Delete);
        } else if x < search_end {
            return UiElement::ActionButton(ActionButton::Search);
        } else {
            return UiElement::ActionButton(ActionButton::Refresh);
        }
    }
    
    // Table rows handling - CORRECTED by increasing offset by 1 
    // Based on testing, we need to increase the offset to fix grid selection
    let table_header_row = 6; 
    
    // Data rows start at position 9 (increased by 1 from previous value)
    let data_start_row = 9;  // CORRECTED: Changed from 7 to 9 to fix grid selection
    
    // Table ends right above action buttons
    let table_end_row = action_bar_row;
    
    // Check if clicking in the table area
    if y >= data_start_row && y < table_end_row {
        // Calculate row index by subtracting starting position
        let row_index = (y - data_start_row) as usize;
        
        // Verify the row index is valid for the current tab
        match active_tab {
            ActiveTab::Students => {
                if row_index < data_manager.get_all_students().len() {
                    return UiElement::TableRow(row_index);
                }
            },
            ActiveTab::Teachers => {
                if row_index < data_manager.get_all_teachers().len() {
                    return UiElement::TableRow(row_index);
                }
            },
            ActiveTab::Faculties => {
                if row_index < data_manager.get_all_faculties().len() {
                    return UiElement::TableRow(row_index);
                }
            },
        }
    }
    
    UiElement::None
}

impl AppState {
    // ...existing code...
}


// UI rendering functions
pub fn render(f: &mut Frame, app_state: &mut AppState, students: &[Student], teachers: &[Teacher], faculties: &[Faculty]) {
    // Set a dark background for the entire screen
    let background = Block::default()
        .style(Style::default().bg(Color::Rgb(16, 16, 28))); // Dark blue/purple background
    f.render_widget(background, f.area());
    
    // Create a layout with header, footer, and main content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(1),  // Footer
        ])
        .split(f.area());

    // Render the header with tabs
    render_header(f, chunks[0], app_state);

    // Render the main content area (tab content)
    render_main_content(f, chunks[1], app_state, students, teachers, faculties);

    // Render the footer with shortcuts
    render_footer(f, chunks[2]);

    // Render notification if present
    if let Some(notification) = &app_state.notification {
        render_notification(f, notification);
    }
}

fn render_header(f: &mut Frame, area: Rect, app_state: &AppState) {
    let titles: Vec<_> = ["Students (1)", "Teachers (2)", "Faculties (3)"]
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let (first, rest) = t.split_at(1);
            let color = if i == app_state.active_tab as usize {
                Color::Yellow
            } else {
                Color::White
            };
            
            Line::from(vec![
                Span::styled(first, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::styled(rest, Style::default().fg(color)),
            ])
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .title("University Manager")
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
        .select(app_state.active_tab as usize)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(tabs, area);
}

fn render_main_content(
    f: &mut Frame,
    area: Rect,
    app_state: &mut AppState,
    students: &[Student],
    teachers: &[Teacher],
    faculties: &[Faculty],
) {
    // Split the main area into search bar and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Search bar
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Action bar
        ])
        .split(area);

    // Render search bar
    render_search_bar(f, chunks[0], app_state);

    // Render content based on active tab
    match app_state.active_tab {
        ActiveTab::Students => render_students_table(f, chunks[1], app_state, students),
        ActiveTab::Teachers => render_teachers_table(f, chunks[1], app_state, teachers),
        ActiveTab::Faculties => render_faculties_table(f, chunks[1], app_state, faculties),
    }

    // Render action bar
    render_action_bar(f, chunks[2]);
}

fn render_search_bar(f: &mut Frame, area: Rect, app_state: &AppState) {
    let search_text = Paragraph::new(format!("Search: {}", app_state.search_query))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Blue))
            .title("Search")
            .title_style(Style::default().fg(Color::Magenta)))
        .style(Style::default().fg(Color::White));
    f.render_widget(search_text, area);
}

fn render_students_table(f: &mut Frame, area: Rect, app_state: &mut AppState, students: &[Student]) {
    let selected_style = Style::default()
        .bg(Color::Blue)
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let normal_style = Style::default().bg(Color::Black);
    
    let header_cells = ["Name", "Age", "Major", "GPA"]
        .iter()
        .map(|h| {
            Span::styled(*h, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        });
    let header = Row::new(header_cells)
        .height(1)
        .bottom_margin(1)
        .style(normal_style);
    
    let rows = students.iter().map(|s| {
        let cells = [
            s.full_name(),
            s.age.to_string(),
            s.major.clone(),
            format!("{:.2}", s.gpa),
        ];
        Row::new(cells).height(1).bottom_margin(0)
    });
    
    let widths = [
        Constraint::Percentage(40),
        Constraint::Percentage(10),
        Constraint::Percentage(35),
        Constraint::Percentage(15),
    ];
    
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Green))
            .title("Students")
            .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)))
        .row_highlight_style(selected_style)
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(table, area, &mut app_state.student_list_state);
}

fn render_teachers_table(f: &mut Frame, area: Rect, app_state: &mut AppState, teachers: &[Teacher]) {
    let selected_style = Style::default()
        .bg(Color::Blue)
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let normal_style = Style::default().bg(Color::Black);
    
    let header_cells = ["Name", "Age", "Department", "Title"]
        .iter()
        .map(|h| {
            Span::styled(*h, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        });
    let header = Row::new(header_cells)
        .height(1)
        .bottom_margin(1)
        .style(normal_style);
    
    let rows = teachers.iter().map(|t| {
        let cells = [
            t.full_name(),
            t.age.to_string(),
            t.department.clone(),
            t.title.clone(),
        ];
        Row::new(cells).height(1).bottom_margin(0)
    });
    
    let widths = [
        Constraint::Percentage(30),
        Constraint::Percentage(10),
        Constraint::Percentage(40),
        Constraint::Percentage(20),
    ];
    
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Blue))
            .title("Teachers")
            .title_style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)))
        .row_highlight_style(selected_style)
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(table, area, &mut app_state.teacher_list_state);
}

fn render_faculties_table(f: &mut Frame, area: Rect, app_state: &mut AppState, faculties: &[Faculty]) {
    let selected_style = Style::default()
        .bg(Color::Blue)
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let normal_style = Style::default().bg(Color::Black);
    
    let header_cells = ["Name", "Building", "Head", "Est. Year", "Staff"]
        .iter()
        .map(|h| {
            Span::styled(*h, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        });
    let header = Row::new(header_cells)
        .height(1)
        .bottom_margin(1)
        .style(normal_style);
    
    let rows = faculties.iter().map(|f| {
        let cells = [
            f.name.clone(),
            f.building.clone(),
            f.head_name.clone(),
            f.established_year.to_string(),
            f.num_staff.to_string(),
        ];
        Row::new(cells).height(1).bottom_margin(0)
    });
    
    let widths = [
        Constraint::Percentage(25),
        Constraint::Percentage(20),
        Constraint::Percentage(25),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
    ];
    
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Magenta))
            .title("Faculties")
            .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)))
        .row_highlight_style(selected_style)
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(table, area, &mut app_state.faculty_list_state);
}

fn render_action_bar(f: &mut Frame, area: Rect) {
    // Create a background for the action bar
    let block = Block::default()
        .title(" Actions ")
        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow));
    
    f.render_widget(block.clone(), area);
    
    // Create inner area for buttons
    let inner_area = area.inner(Margin::new(1, 1));
    
    // Calculate button widths
    let button_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(15), // Add button
            Constraint::Percentage(15), // Edit button
            Constraint::Percentage(15), // Delete button
            Constraint::Percentage(25), // Focus Search button
            Constraint::Percentage(15), // Refresh button
            Constraint::Percentage(15), // Extra space
        ])
        .split(inner_area);
    
    // Render colored buttons similar to the delete modal buttons
    render_button(f, button_layout[0], "A: Add", Color::Green);
    render_button(f, button_layout[1], "E: Edit", Color::Blue);
    render_button(f, button_layout[2], "D: Delete", Color::Red);
    render_button(f, button_layout[3], "F: Focus Search", Color::Yellow);
    render_button(f, button_layout[4], "R: Refresh", Color::Cyan);
}

// Helper function to render a button
fn render_button(f: &mut Frame, area: Rect, text: &str, color: Color) {
    let button = Paragraph::new(text)
        .alignment(ratatui::layout::Alignment::Center)
        .style(Style::default()
            .fg(Color::White)
            .bg(color)
            .add_modifier(Modifier::BOLD));
    
    f.render_widget(button, area);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let text = Line::from(vec![
        Span::styled("Q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(": Quit   "),
        Span::styled("Tab/1/2/3", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(": Switch tabs   "),
        Span::styled("↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(": Navigate   "),
        Span::styled("H", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(": Help"),
    ]);
    
    let paragraph = Paragraph::new(text).style(Style::default().fg(Color::White));
    f.render_widget(paragraph, area);
}

fn render_notification(f: &mut Frame, notification: &str) {
    let area = centered_rect(60, 4, f.area());
    
    let block = Block::default()
        .title(" Notification ")
        .title_style(Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(Color::DarkGray));
    
    let inner = area.inner(Margin::new(1, 0));
    f.render_widget(block, area);
    
    let text = Text::from(notification);
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .alignment(ratatui::layout::Alignment::Center);
    
    f.render_widget(paragraph, inner);
}

// Helper function to create a centered rect using percentage of the available rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}