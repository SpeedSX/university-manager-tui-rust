use crate::models::{Faculty, Student, Teacher};
use crate::terminal_size;
use crate::widgets::{self, DropdownState};
use anyhow::Result;
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph},
    Frame,
};
use std::fmt;

#[derive(Debug, Clone)]
pub enum ModalType {
    AddStudent,
    EditStudent(Student),
    AddTeacher,
    EditTeacher(Teacher),
    AddFaculty,
    EditFaculty(Faculty),
    DeleteConfirmation(String, String), // (id, name) for entity to delete
    Message(String),                     // General message display
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputField {
    FirstName,
    LastName,
    Age,
    Major,
    Gpa,
    Department,
    Title,
    Name,
    Building,
    HeadName,
    EstablishedYear,
    NumStaff,
    None,
}

impl fmt::Display for InputField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            InputField::FirstName => "First Name",
            InputField::LastName => "Last Name",
            InputField::Age => "Age",
            InputField::Major => "Major",
            InputField::Gpa => "GPA",
            InputField::Department => "Department",
            InputField::Title => "Title",
            InputField::Name => "Name",
            InputField::Building => "Building",
            InputField::HeadName => "Head Name",
            InputField::EstablishedYear => "Established Year",
            InputField::NumStaff => "Number of Staff",
            InputField::None => "",
        };
        write!(f, "{}", label)
    }
}

pub struct Modal {
    pub modal_type: ModalType,
    pub active: bool,
    pub inputs: Vec<(InputField, String)>,
    pub active_field: usize,
    pub confirm: bool,
    pub major_dropdown: DropdownState,
}

impl Modal {
    pub fn new(modal_type: ModalType) -> Self {
        let inputs = match &modal_type {
            ModalType::AddStudent => vec![
                (InputField::FirstName, String::new()),
                (InputField::LastName, String::new()),
                (InputField::Age, String::new()),
                (InputField::Major, String::new()),
                (InputField::Gpa, String::new()),
            ],
            ModalType::EditStudent(student) => vec![
                (InputField::FirstName, student.first_name.clone()),
                (InputField::LastName, student.last_name.clone()),
                (InputField::Age, student.age.to_string()),
                (InputField::Major, student.major.clone()),
                (InputField::Gpa, student.gpa.to_string()),
            ],
            ModalType::AddTeacher => vec![
                (InputField::FirstName, String::new()),
                (InputField::LastName, String::new()),
                (InputField::Age, String::new()),
                (InputField::Department, String::new()),
                (InputField::Title, String::new()),
            ],
            ModalType::EditTeacher(teacher) => vec![
                (InputField::FirstName, teacher.first_name.clone()),
                (InputField::LastName, teacher.last_name.clone()),
                (InputField::Age, teacher.age.to_string()),
                (InputField::Department, teacher.department.clone()),
                (InputField::Title, teacher.title.clone()),
            ],
            ModalType::AddFaculty => vec![
                (InputField::Name, String::new()),
                (InputField::Building, String::new()),
                (InputField::HeadName, String::new()),
                (InputField::EstablishedYear, String::new()),
                (InputField::NumStaff, String::new()),
            ],
            ModalType::EditFaculty(faculty) => vec![
                (InputField::Name, faculty.name.clone()),
                (InputField::Building, faculty.building.clone()),
                (InputField::HeadName, faculty.head_name.clone()),
                (InputField::EstablishedYear, faculty.established_year.to_string()),
                (InputField::NumStaff, faculty.num_staff.to_string()),
            ],
            ModalType::DeleteConfirmation(_, _) => vec![],
            ModalType::Message(_) => vec![],
        };

        Self {
            modal_type,
            active: true,
            inputs,
            active_field: 0,
            confirm: false,
            major_dropdown: DropdownState::new(widgets::MAJORS.iter().map(|&s| s.into()).collect()), // Initialize with predefined majors
        }
    }

    pub fn next_field(&mut self) {
        if self.inputs.is_empty() {
            return;
        }
        self.active_field = (self.active_field + 1) % self.inputs.len();
    }

    pub fn prev_field(&mut self) {
        if self.inputs.is_empty() {
            return;
        }
        self.active_field = if self.active_field == 0 {
            self.inputs.len() - 1
        } else {
            self.active_field - 1
        };
    }

    pub fn input(&mut self, c: char) {
        if self.inputs.is_empty() || self.active_field >= self.inputs.len() {
            return;
        }
        
        match self.inputs[self.active_field].0 {
            InputField::Age | InputField::EstablishedYear | InputField::NumStaff => {
                // Only allow digits for numerical fields
                if c.is_digit(10) {
                    self.inputs[self.active_field].1.push(c);
                }
            }
            InputField::Gpa => {
                // Allow digits and one decimal point for GPA
                if c.is_digit(10) || (c == '.' && !self.inputs[self.active_field].1.contains('.')) {
                    self.inputs[self.active_field].1.push(c);
                }
            }
            _ => {
                // Allow any character for text fields
                self.inputs[self.active_field].1.push(c);
            }
        }
    }

    pub fn backspace(&mut self) {
        if self.inputs.is_empty() || self.active_field >= self.inputs.len() {
            return;
        }
        self.inputs[self.active_field].1.pop();
    }

    pub fn create_student(&self) -> Option<Student> {
        if self.inputs.len() < 5 {
            return None;
        }

        // Extract values
        let first_name = &self.inputs[0].1;
        let last_name = &self.inputs[1].1;
        let age_str = &self.inputs[2].1;
        let major = &self.inputs[3].1;
        let gpa_str = &self.inputs[4].1;

        // Basic validation
        if first_name.is_empty() || last_name.is_empty() || major.is_empty() || 
           age_str.is_empty() || gpa_str.is_empty() {
            return None;
        }

        // Parse numeric values
        let age = match age_str.parse::<u32>() {
            Ok(a) if a >= 16 && a <= 99 => a,
            _ => return None,
        };

        let gpa = match gpa_str.parse::<f32>() {
            Ok(g) if g >= 0.0 && g <= 4.0 => g,
            _ => return None,
        };

        // Create Student
        match &self.modal_type {
            ModalType::EditStudent(student) => Some(Student::with_id(
                student.id.clone(),
                first_name.clone(),
                last_name.clone(),
                age,
                major.clone(),
                gpa,
            )),
            _ => Some(Student::new(
                first_name.clone(),
                last_name.clone(),
                age,
                major.clone(),
                gpa,
            )),
        }
    }

    pub fn create_teacher(&self) -> Option<Teacher> {
        if self.inputs.len() < 5 {
            return None;
        }

        // Extract values
        let first_name = &self.inputs[0].1;
        let last_name = &self.inputs[1].1;
        let age_str = &self.inputs[2].1;
        let department = &self.inputs[3].1;
        let title = &self.inputs[4].1;

        // Basic validation
        if first_name.is_empty() || last_name.is_empty() || department.is_empty() || 
           title.is_empty() || age_str.is_empty() {
            return None;
        }

        // Parse numeric values
        let age = match age_str.parse::<u32>() {
            Ok(a) if a >= 18 && a <= 99 => a,
            _ => return None,
        };

        // Create Teacher
        match &self.modal_type {
            ModalType::EditTeacher(teacher) => Some(Teacher::with_id(
                teacher.id.clone(),
                first_name.clone(),
                last_name.clone(),
                age,
                department.clone(),
                title.clone(),
            )),
            _ => Some(Teacher::new(
                first_name.clone(),
                last_name.clone(),
                age,
                department.clone(),
                title.clone(),
            )),
        }
    }

    pub fn create_faculty(&self) -> Option<Faculty> {
        if self.inputs.len() < 5 {
            return None;
        }

        // Extract values
        let name = &self.inputs[0].1;
        let building = &self.inputs[1].1;
        let head_name = &self.inputs[2].1;
        let established_year_str = &self.inputs[3].1;
        let num_staff_str = &self.inputs[4].1;

        // Basic validation
        if name.is_empty() || building.is_empty() || head_name.is_empty() || 
           established_year_str.is_empty() || num_staff_str.is_empty() {
            return None;
        }

        // Parse numeric values
        let established_year = match established_year_str.parse::<u32>() {
            Ok(y) if y >= 1500 && y <= 2025 => y, // Assuming current year is 2025
            _ => return None,
        };

        let num_staff = match num_staff_str.parse::<u32>() {
            Ok(n) if n > 0 => n,
            _ => return None,
        };

        // Create Faculty
        match &self.modal_type {
            ModalType::EditFaculty(faculty) => Some(Faculty::with_id(
                faculty.id.clone(),
                name.clone(),
                building.clone(),
                head_name.clone(),
                established_year,
                num_staff,
            )),
            _ => Some(Faculty::new(
                name.clone(),
                building.clone(),
                head_name.clone(),
                established_year,
                num_staff,
            )),
        }
    }

    pub fn get_delete_id(&self) -> Option<String> {
        match &self.modal_type {
            ModalType::DeleteConfirmation(id, _) => Some(id.clone()),
            _ => None,
        }
    }
}

// Render the active modal
pub fn render_modal(f: &mut Frame, modal: &mut Modal) {
    if !modal.active {
        return;
    }

    // Create a centered box for our modal
    let area = centered_rect(60, 60, f.area());
    
    // Clear the area
    f.render_widget(Clear, area);
    
    // Render the appropriate modal content
    match &modal.modal_type {
        ModalType::AddStudent | ModalType::EditStudent(_) => {
            render_student_modal(f, modal, area);
        }
        ModalType::AddTeacher | ModalType::EditTeacher(_) => {
            render_teacher_modal(f, modal, area);
        }
        ModalType::AddFaculty | ModalType::EditFaculty(_) => {
            render_faculty_modal(f, modal, area);
        }
        ModalType::DeleteConfirmation(_, name) => {
            render_delete_modal(f, name, area);
        }
        ModalType::Message(msg) => {
            render_message_modal(f, msg, area);
        }
    }
}

fn render_student_modal(f: &mut Frame, modal: &mut Modal, area: Rect) {
    let is_edit = matches!(modal.modal_type, ModalType::EditStudent(_));
    let title = if is_edit { "Edit Student" } else { "Add Student" };
    
    // Create modal border with title
    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Green));
    
    f.render_widget(Clear, area); // Clear the area first
    f.render_widget(block.clone(), area);
    
    // Create inner area for content - use Margin::new(1, 1) for a 1-character margin
    let inner_area = area.inner(Margin::new(1, 1));
    
    // Create layout for fields
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3), // First Name
                Constraint::Length(3), // Last Name
                Constraint::Length(3), // Age
                Constraint::Length(3), // Major
                Constraint::Length(3), // GPA
                Constraint::Length(3), // Buttons
            ]
            .as_ref(),
        )
        .split(inner_area);
    
    // Render all fields first
    for i in 0..5 {
        let (field, value) = &modal.inputs[i];
        let is_active = modal.active_field == i;
        
        // For Major field, just render the field (dropdown will come later)
        if i == 3 { // Major field is at index 3
            widgets::render_dropdown_field(
                f,
                chunks[i],
                &field.to_string(),
                &value,
                is_active,
                modal.major_dropdown.is_open
            );
        } else {
            // Normal field rendering for non-dropdown fields
            let style = if is_active {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray)
            } else {
                Style::default()
            };
            
            let field_block = Block::default()
                .borders(Borders::ALL)
                .border_style(style);
            
            let cursor = if is_active { "|" } else { "" };
            let label_style = Style::default().fg(Color::Cyan);
            let value_style = Style::default().fg(Color::White);
            
            let text = Line::from(vec![
                Span::styled(format!("{}: ", field), label_style),
                Span::styled(value.clone(), value_style),
                Span::styled(cursor, Style::default().fg(Color::Yellow)),
            ]);
            
            let paragraph = Paragraph::new(text).block(field_block);
            f.render_widget(paragraph, chunks[i]);
        }
    }
    
    // Render buttons with colors
    let button_area = chunks[5];
    let button_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(button_area);
    
    render_modal_button(f, button_layout[0], "Enter: Save", Color::Green);
    render_modal_button(f, button_layout[1], "Esc: Cancel", Color::Red);
    
    // Render the dropdown last, so it appears on top of everything else
    if modal.active_field == 3 && modal.major_dropdown.is_open {
        widgets::render_dropdown(f, &mut modal.major_dropdown, chunks[3]);
    }
}

fn render_teacher_modal(f: &mut Frame, modal: &mut Modal, area: Rect) {
    let is_edit = matches!(modal.modal_type, ModalType::EditTeacher(_));
    let title = if is_edit { "Edit Teacher" } else { "Add Teacher" };
    
    // Create modal border with title
    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Blue));
    
    f.render_widget(Clear, area); // Clear the area first
    f.render_widget(block.clone(), area);
    
    // Create inner area for content - use Margin::new(1, 1) for a 1-character margin
    let inner_area = area.inner(Margin::new(1, 1));
    
    // Create layout for fields
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3), // First Name
                Constraint::Length(3), // Last Name
                Constraint::Length(3), // Age
                Constraint::Length(3), // Department
                Constraint::Length(3), // Title
                Constraint::Length(3), // Buttons
            ]
            .as_ref(),
        )
        .split(inner_area);
    
    // Render fields
    for i in 0..5 {
        let (field, value) = &modal.inputs[i];
        let is_active = modal.active_field == i;
        
        let style = if is_active {
            Style::default().fg(Color::Yellow).bg(Color::DarkGray)
        } else {
            Style::default()
        };
        
        let field_block = Block::default()
            .borders(Borders::ALL)
            .border_style(style);
        
        let cursor = if is_active { "|" } else { "" };
        let label_style = Style::default().fg(Color::Cyan);
        let value_style = Style::default().fg(Color::White);
        
        let text = Line::from(vec![Span::styled(format!("{}: ", field), label_style), Span::styled(value.clone(), value_style), Span::styled(cursor, Style::default().fg(Color::Yellow)),]);
        
        let paragraph = Paragraph::new(text)
            .block(field_block);
        
        f.render_widget(paragraph, chunks[i]);
    }
    
    // Render buttons with colors
    let button_area = chunks[5];
    let button_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(button_area);
    
    render_modal_button(f, button_layout[0], "Enter: Save", Color::Green);
    render_modal_button(f, button_layout[1], "Esc: Cancel", Color::Red);
}

fn render_faculty_modal(f: &mut Frame, modal: &mut Modal, area: Rect) {
    let is_edit = matches!(modal.modal_type, ModalType::EditFaculty(_));
    let title = if is_edit { "Edit Faculty" } else { "Add Faculty" };
    
    // Create modal border with title
    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Magenta));
    
    f.render_widget(Clear, area); // Clear the area first
    f.render_widget(block.clone(), area);
    
    // Create inner area for content - use Margin::new(1, 1) for a 1-character margin
    let inner_area = area.inner(Margin::new(1, 1));
    
    // Create layout for fields
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3), // Name
                Constraint::Length(3), // Building
                Constraint::Length(3), // Head Name
                Constraint::Length(3), // Established Year
                Constraint::Length(3), // Number of Staff
                Constraint::Length(3), // Buttons
            ]
            .as_ref(),
        )
        .split(inner_area);
    
    // Render fields
    for i in 0..5 {
        let (field, value) = &modal.inputs[i];
        let is_active = modal.active_field == i;
        
        let style = if is_active {
            Style::default().fg(Color::Yellow).bg(Color::DarkGray)
        } else {
            Style::default()
        };
        
        let field_block = Block::default()
            .borders(Borders::ALL)
            .border_style(style);
        
        let cursor = if is_active { "|" } else { "" };
        let label_style = Style::default().fg(Color::Cyan);
        let value_style = Style::default().fg(Color::White);
        
        let text = Line::from(vec![Span::styled(format!("{}: ", field), label_style), Span::styled(value.clone(), value_style), Span::styled(cursor, Style::default().fg(Color::Yellow)),]);
        
        let paragraph = Paragraph::new(text)
            .block(field_block);
        
        f.render_widget(paragraph, chunks[i]);
    }
    
    // Render buttons with colors
    let button_area = chunks[5];
    let button_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(button_area);
    
    render_modal_button(f, button_layout[0], "Enter: Save", Color::Green);
    render_modal_button(f, button_layout[1], "Esc: Cancel", Color::Red);
}

fn render_delete_modal(f: &mut Frame, name: &str, area: Rect) {
    // Create a modal with fixed minimum width and height
    // 50 characters wide, 12 characters tall (minimum)
    let width = std::cmp::max(50, area.width.saturating_mul(80).saturating_div(100));
    let height = 12; // Fixed minimum height
    
    let modal_area = centered_rect_with_min_size(width, height, area);
    
    // Create a clear area
    let clear_area = modal_area;
    f.render_widget(Clear, clear_area);
    
    // Create modal border with title
    let block = Block::default()
        .title(" Confirm Delete ")
        .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Red));
    
    f.render_widget(block.clone(), modal_area);
    
    // Create inner area for content with 2 character horizontal margin, 1 character vertical
    let inner_area = modal_area.inner(Margin::new(2, 1));
    
    // Create layout for message and buttons
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),    // Space for warning symbol + message
            Constraint::Length(1),    // Empty space
            Constraint::Length(3),    // Buttons height
        ])
        .split(inner_area);
    
    // Warning symbol inline with text
    let warning_text = format!("⚠  Are you sure you want to delete {}?", name);
    let message = Paragraph::new(warning_text)
        .style(Style::default().fg(Color::White))
        .alignment(ratatui::layout::Alignment::Center);
    
    f.render_widget(message, chunks[0]);
    
    // Create button layout
    let button_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),  // Left spacing
            Constraint::Percentage(30),  // Delete button
            Constraint::Percentage(30),  // Cancel button
            Constraint::Percentage(20),  // Right spacing
        ])
        .split(chunks[2]);
    
    // Render delete button (red background, no borders)
    let delete_button = Paragraph::new("Enter: Delete")
        .style(Style::default()
            .fg(Color::White)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().padding(Padding::new(1, 0, 0, 0)));
    
    // Render cancel button (blue background, no borders)
    let cancel_button = Paragraph::new("Esc: Cancel")
        .style(Style::default()
            .fg(Color::White)
            .bg(Color::Blue)
            .add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().padding(Padding::new(1, 0, 0, 0)));
    
    f.render_widget(delete_button, button_layout[1]);
    f.render_widget(cancel_button, button_layout[2]);
}

fn render_message_modal(f: &mut Frame, message: &str, area: Rect) {
    // Create modal border with title
    let block = Block::default()
        .title(" Message ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::DarkGray));
    
    f.render_widget(Clear, area); // Clear the area first
    f.render_widget(block.clone(), area);
    
    // Create inner area for content - use Margin::new(1, 1) for a 1-character margin
    let inner_area = area.inner(Margin::new(1, 1));
    
    // Create layout for message and buttons
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
        .split(inner_area);
    
    // Render message with info icon
    let message_text = Line::from(vec![
        Span::styled("ℹ ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        Span::styled(message, Style::default().fg(Color::White)),
    ]);
    
    let message = Paragraph::new(message_text)
        .alignment(ratatui::layout::Alignment::Center);
    
    f.render_widget(message, chunks[0]);
    
    // Render button with color
    render_modal_button(f, chunks[1], "Press Esc to close", Color::Blue);
}

// Helper function to render a modal button
fn render_modal_button(f: &mut Frame, area: Rect, text: &str, color: Color) {
    let button = Paragraph::new(text)
        .alignment(ratatui::layout::Alignment::Center)
        .style(Style::default().fg(Color::White).bg(color).add_modifier(Modifier::BOLD));
    
    f.render_widget(button, area);
}

// Helper function to create a centered rectangle
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

// Helper function to check if a position is within a rectangle
pub fn is_position_in_rect(position: (u16, u16), rect: ratatui::layout::Rect) -> bool {
    let (x, y) = position;
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}

// Helper function to create a centered rect with a minimum size
pub fn centered_rect_with_min_size(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    use ratatui::layout::{Constraint, Direction, Layout};
    
    // Calculate width and height
    let width = std::cmp::max(percent_x, r.width.saturating_mul(percent_x).saturating_div(100));
    let height = std::cmp::max(percent_y, r.height.saturating_mul(percent_y).saturating_div(100));
    
    // Make sure width and height don't exceed the available area
    let width = std::cmp::min(width, r.width.saturating_sub(4));  // Leave at least 2 chars on each side
    let height = std::cmp::min(height, r.height.saturating_sub(4));  // Leave at least 2 chars on each side
    
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Length((r.height.saturating_sub(height)) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((r.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Length((r.width.saturating_sub(width)) / 2),
        ])
        .split(popup_layout[1])[1]
}

// Add this function at the end of the file to detect mouse clicks on modal buttons
pub fn get_modal_element_at_position(
    position: (u16, u16),
    modal: &Modal,
    area: Rect
) -> Option<crate::ui::ModalButton> {
    // Only process if modal is active
    if !modal.active {
        return None;
    }
    
    // For delete confirmation modal
    if let ModalType::DeleteConfirmation(_, _) = modal.modal_type {
        // Use the same method to calculate the modal area as in render_delete_modal
        let width = std::cmp::max(50, area.width.saturating_mul(80).saturating_div(100));
        let height = 12; // Same fixed height as in render_delete_modal
        let modal_area = centered_rect_with_min_size(width, height, area);
        
        if !is_position_in_rect(position, modal_area) {
            return None;
        }
        
        // Create inner area for content with the same margins as render_delete_modal
        let inner_area = modal_area.inner(Margin::new(2, 1));
        
        // Use the same layout as in render_delete_modal
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),    // Space for warning symbol + message
                Constraint::Length(1),    // Empty space
                Constraint::Length(3),    // Buttons height
            ])
            .split(inner_area);
            
        // Use the same button layout as in render_delete_modal
        let button_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),  // Left spacing
                Constraint::Percentage(30),  // Delete button
                Constraint::Percentage(30),  // Cancel button
                Constraint::Percentage(20),  // Right spacing
            ])
            .split(chunks[2]);
        
        // Check if clicking on the delete button - use the entire button area
        if is_position_in_rect(position, button_layout[1]) {
            return Some(crate::ui::ModalButton::Confirm);
        }
        
        // Check if clicking on the cancel button - use the entire button area
        if is_position_in_rect(position, button_layout[2]) {
            return Some(crate::ui::ModalButton::Cancel);
        }
        
        return None;
    }
    
    // For all other modals, use the default 60, 60 size
    let modal_area = centered_rect(60, 60, area);
    if !is_position_in_rect(position, modal_area) {
        return None;
    }
    
    // Create inner area for content
    let inner_area = modal_area.inner(Margin::new(1, 1));
    
    // For other modals with form fields
    match modal.modal_type {
        ModalType::AddStudent | ModalType::EditStudent(_) |
        ModalType::AddTeacher | ModalType::EditTeacher(_) |
        ModalType::AddFaculty | ModalType::EditFaculty(_) => {
            // Layout for form fields
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3), // Field 1
                    Constraint::Length(3), // Field 2
                    Constraint::Length(3), // Field 3
                    Constraint::Length(3), // Field 4
                    Constraint::Length(3), // Field 5
                    Constraint::Length(3), // Buttons
                ])
                .split(inner_area);
                
            // Check the buttons row
            let button_area = chunks[5];
            let button_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(button_area);
                
            // Check if clicking on the save button - use the entire button area
            if is_position_in_rect(position, button_layout[0]) {
                return Some(crate::ui::ModalButton::Confirm);
            }
            
            // Check if clicking on the cancel button - use the entire button area
            if is_position_in_rect(position, button_layout[1]) {
                return Some(crate::ui::ModalButton::Cancel);
            }
        },
        ModalType::Message(_) => {
            // For message modal, any click anywhere should close it (like pressing Esc)
            return Some(crate::ui::ModalButton::Cancel);
        },
        _ => {}
    }
    
    None
}

// Check if a click is on a dropdown item and return the selected item if it is
pub fn is_dropdown_item_clicked(position: (u16, u16), dropdown: &widgets::DropdownState, modal: &Modal) -> Option<String> {
    // Only process if dropdown is open
    if !dropdown.is_open {
        return None;
    }
    
    // Find the index of the major field
    let major_field_index = 3; // We know it's index 3 in the student form
    
    // Calculate the position of the dropdown
    let area = centered_rect(60, 60, terminal_size()); // Get the modal area
    let inner_area = area.inner(Margin::new(1, 1));
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3), // First Name
                Constraint::Length(3), // Last Name
                Constraint::Length(3), // Age
                Constraint::Length(3), // Major
                Constraint::Length(3), // GPA
                Constraint::Length(3), // Buttons
            ]
            .as_ref(),
        )
        .split(inner_area);
    
    // Get the area of the Major field
    let major_field_area = chunks[major_field_index];
    
    // Calculate the dropdown area using the same logic as in widgets::render_dropdown
    let dropdown_area = Rect::new(
        major_field_area.x,
        major_field_area.y + 1,
        major_field_area.width,
        12.min(dropdown.options.len() as u16 + 2),
    );
    
    // Check if click is within the dropdown area
    if !is_position_in_rect(position, dropdown_area) {
        return None;
    }
    
    // Calculate which item was clicked (account for the top border)
    let relative_y = position.1 - dropdown_area.y - 1;
    if relative_y >= dropdown.options.len() as u16 {
        return None;
    }
    
    // Return the selected item
    dropdown.options.get(relative_y as usize).map(|s| s.clone())
}