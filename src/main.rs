mod data_manager;
mod modal;
mod models;
mod ui;
mod widgets;

use crate::data_manager::DataManager;
use crate::modal::{Modal, ModalType};
use crate::models::{Faculty, Student, Teacher};
use crate::ui::{AppState, ActiveTab, render, get_element_at_position};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEventKind, MouseButton},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use std::{
    io,
    time::{Duration, Instant},
};

enum AppMode {
    Normal,
    Search,
    Modal(Modal),
}

struct App {
    state: AppState,
    data_manager: DataManager,
    mode: AppMode,
    should_quit: bool,
    tick_rate: Duration,
    last_tick: Instant,
}

impl App {
    fn new() -> Result<Self> {
        let data_manager = DataManager::new(None)?;
        
        Ok(Self {
            state: AppState::default(),
            data_manager,
            mode: AppMode::Normal,
            should_quit: false,
            tick_rate: Duration::from_millis(100), // 10 ticks per second
            last_tick: Instant::now(),
        })
    }

    fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|f| self.render(f))?;
            self.handle_events()?;
            self.tick()?;
        }
        
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        match &mut self.mode {
            AppMode::Normal | AppMode::Search => {
                let students = self.data_manager.get_all_students();
                let teachers = self.data_manager.get_all_teachers();
                let faculties = self.data_manager.get_all_faculties();
                
                render(frame, &mut self.state, students, teachers, faculties);
            }
            AppMode::Modal(modal) => {
                // Render the base UI first
                let students = self.data_manager.get_all_students();
                let teachers = self.data_manager.get_all_teachers();
                let faculties = self.data_manager.get_all_faculties();
                
                render(frame, &mut self.state, students, teachers, faculties);
                
                // Then render the modal on top
                modal::render_modal(frame, modal);
            }
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key_event(key.code)?;
                    }
                },
                Event::Mouse(mouse) => {
                    self.handle_mouse_event(mouse)?;
                },
                _ => {}
            }
        }
        
        Ok(())
    }

    fn handle_mouse_event(&mut self, mouse: event::MouseEvent) -> Result<()> {
        // Only handle mouse press events
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return Ok(());
        }

        // Get the position of the mouse click
        let position = (mouse.column, mouse.row);

        // Check if this is a click in a modal
        if let AppMode::Modal(modal) = &self.mode {
            // Special handling for student modal with open dropdown
            if (matches!(modal.modal_type, ModalType::AddStudent | ModalType::EditStudent(_)) && 
                modal.active_field == 3 && 
                modal.major_dropdown.is_open) {
                
                // Check if click is in the dropdown list area
                if let Some(selected_item) = modal::is_dropdown_item_clicked(position, &modal.major_dropdown, modal) {
                    // Update the major field with the selected item
                    if let AppMode::Modal(modal) = &mut self.mode {
                        modal.inputs[3].1 = selected_item;
                        modal.major_dropdown.is_open = false;
                    }
                    return Ok(());
                }
                
                // If click is outside dropdown area, close the dropdown
                if let AppMode::Modal(modal) = &mut self.mode {
                    modal.major_dropdown.is_open = false;
                }
                return Ok(());
            }

            // Regular modal button detection
            if let Some(button) = modal::get_modal_element_at_position(position, modal, terminal_size()) {
                match button {
                    ui::ModalButton::Confirm => self.handle_modal_key_event(KeyCode::Enter)?,
                    ui::ModalButton::Cancel => self.handle_modal_key_event(KeyCode::Esc)?,
                }
                return Ok(());
            }
        }

        // Get the UI element at the position
        let element = get_element_at_position(
            position,
            self.state.active_tab,
            &self.data_manager,
            &mut self.state
        );

        // Handle the click based on the element
        match element {
            ui::UiElement::Tab(tab) => {
                self.state.active_tab = tab;
                self.refresh_data();
            },
            ui::UiElement::TableRow(index) => {
                match self.state.active_tab {
                    ActiveTab::Students => self.state.student_list_state.select(Some(index)),
                    ActiveTab::Teachers => self.state.teacher_list_state.select(Some(index)),
                    ActiveTab::Faculties => self.state.faculty_list_state.select(Some(index)),
                }
            },
            ui::UiElement::ActionButton(action) => {
                match action {
                    ui::ActionButton::Add => self.show_add_modal(),
                    ui::ActionButton::Edit => self.show_edit_modal(),
                    ui::ActionButton::Delete => self.show_delete_modal(),
                    ui::ActionButton::Search => self.mode = AppMode::Search,
                    ui::ActionButton::Refresh => {
                        self.refresh_data();
                        self.state.show_notification("Data refreshed".to_string());
                    },
                }
            },
            ui::UiElement::None => {},
            _ => {},
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyCode) -> Result<()> {
        // First determine what type of mode we're in and handle accordingly
        match self.mode {
            AppMode::Normal => {
                return self.handle_normal_mode(key);
            }
            AppMode::Search => {
                return self.handle_search_mode(key);
            }
            AppMode::Modal(_) => {
                // For modal mode, we need a different approach to avoid borrow conflicts
                return self.handle_modal_key_event(key);
            }
        }
    }

    fn handle_normal_mode(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('f') => {
                self.mode = AppMode::Search;
            }
            KeyCode::Char('a') => {
                self.show_add_modal();
            }
            KeyCode::Char('e') => {
                self.show_edit_modal();
            }
            KeyCode::Char('d') => {
                self.show_delete_modal();
            }
            KeyCode::Char('r') => {
                self.refresh_data();
                self.state.show_notification("Data refreshed".to_string());
            }
            KeyCode::Tab => {
                self.state.active_tab = self.state.active_tab.next();
                self.refresh_data();
            }
            KeyCode::Char('1') => {
                self.state.active_tab = ActiveTab::Students;
                self.refresh_data();
            }
            KeyCode::Char('2') => {
                self.state.active_tab = ActiveTab::Teachers;
                self.refresh_data();
            }
            KeyCode::Char('3') => {
                self.state.active_tab = ActiveTab::Faculties;
                self.refresh_data();
            }
            KeyCode::Up => {
                self.state.select_previous();
            }
            KeyCode::Down => {
                self.state.select_next();
            }
            _ => {}
        }
        
        Ok(())
    }

    fn handle_search_mode(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.refresh_data();
            }
            KeyCode::Enter => {
                self.perform_search();
                self.mode = AppMode::Normal;
            }
            KeyCode::Backspace => {
                if !self.state.search_query.is_empty() {
                    let new_len = self.state.search_query.len() - 1;
                    self.state.search_query.truncate(new_len);
                }
            }
            KeyCode::Char(c) => {
                self.state.search_query.push(c);
            }
            _ => {}
        }
        
        Ok(())
    }

    fn handle_modal_key_event(&mut self, key: KeyCode) -> Result<()> {
        // Handle common modal actions that don't require direct modal access
        if key == KeyCode::Esc {
            if let AppMode::Modal(modal) = &mut self.mode {
                // If dropdown is open, close it instead of closing the modal
                if modal.active_field == 3 && // Major field
                   matches!(modal.modal_type, ModalType::AddStudent | ModalType::EditStudent(_)) &&
                   modal.major_dropdown.is_open {
                    modal.major_dropdown.is_open = false;
                    return Ok(());
                }
            }
            self.mode = AppMode::Normal;
            return Ok(());
        }

        // For Enter key, we need special handling to avoid borrowing conflicts
        if key == KeyCode::Enter {
            // Special handling for dropdowns
            if let AppMode::Modal(modal) = &mut self.mode {
                // If this is a student form and major field is active
                if modal.active_field == 3 && // Major field 
                   matches!(modal.modal_type, ModalType::AddStudent | ModalType::EditStudent(_)) {
                    if modal.major_dropdown.is_open {
                        // If dropdown is open, select current item and close dropdown
                        if let Some(selected) = modal.major_dropdown.selected_item() {
                            modal.inputs[3].1 = selected.clone();
                            modal.major_dropdown.is_open = false;
                        }
                        return Ok(());
                    } else {
                        // Open dropdown when Enter is pressed on the major field
                        modal.major_dropdown.is_open = true;
                        return Ok(());
                    }
                }
            }

            // Clone modal data that we need before changing borrowing
            let modal_type = if let AppMode::Modal(modal) = &self.mode {
                modal.modal_type.clone()
            } else {
                return Ok(());
            };

            // Process based on modal type
            match modal_type {
                ModalType::AddStudent => {
                    if let AppMode::Modal(modal) = &mut self.mode {
                        if let Some(student) = modal.create_student() {
                            self.data_manager.add_student(student.clone())?;
                            self.state.show_notification(format!("Added student: {}", student.full_name()));
                            self.mode = AppMode::Normal;
                            self.refresh_data();
                        } else {
                            self.state.show_notification("Invalid student data".to_string());
                        }
                    }
                }
                ModalType::EditStudent(_) => {
                    if let AppMode::Modal(modal) = &mut self.mode {
                        if let Some(student) = modal.create_student() {
                            self.data_manager.update_student(student.clone())?;
                            self.state.show_notification(format!("Updated student: {}", student.full_name()));
                            self.mode = AppMode::Normal;
                            self.refresh_data();
                        } else {
                            self.state.show_notification("Invalid student data".to_string());
                        }
                    }
                }
                ModalType::AddTeacher => {
                    if let AppMode::Modal(modal) = &mut self.mode {
                        if let Some(teacher) = modal.create_teacher() {
                            self.data_manager.add_teacher(teacher.clone())?;
                            self.state.show_notification(format!("Added teacher: {}", teacher.full_name()));
                            self.mode = AppMode::Normal;
                            self.refresh_data();
                        } else {
                            self.state.show_notification("Invalid teacher data".to_string());
                        }
                    }
                }
                ModalType::EditTeacher(_) => {
                    if let AppMode::Modal(modal) = &mut self.mode {
                        if let Some(teacher) = modal.create_teacher() {
                            self.data_manager.update_teacher(teacher.clone())?;
                            self.state.show_notification(format!("Updated teacher: {}", teacher.full_name()));
                            self.mode = AppMode::Normal;
                            self.refresh_data();
                        } else {
                            self.state.show_notification("Invalid teacher data".to_string());
                        }
                    }
                }
                ModalType::AddFaculty => {
                    if let AppMode::Modal(modal) = &mut self.mode {
                        if let Some(faculty) = modal.create_faculty() {
                            self.data_manager.add_faculty(faculty.clone())?;
                            self.state.show_notification(format!("Added faculty: {}", faculty.name));
                            self.mode = AppMode::Normal;
                            self.refresh_data();
                        } else {
                            self.state.show_notification("Invalid faculty data".to_string());
                        }
                    }
                }
                ModalType::EditFaculty(_) => {
                    if let AppMode::Modal(modal) = &mut self.mode {
                        if let Some(faculty) = modal.create_faculty() {
                            self.data_manager.update_faculty(faculty.clone())?;
                            self.state.show_notification(format!("Updated faculty: {}", faculty.name));
                            self.mode = AppMode::Normal;
                            self.refresh_data();
                        } else {
                            self.state.show_notification("Invalid faculty data".to_string());
                        }
                    }
                }
                ModalType::DeleteConfirmation(id, name) => {
                    let success = match self.state.active_tab {
                        ActiveTab::Students => self.data_manager.delete_student(&id)?,
                        ActiveTab::Teachers => self.data_manager.delete_teacher(&id)?,
                        ActiveTab::Faculties => self.data_manager.delete_faculty(&id)?,
                    };
                    
                    if success {
                        self.state.show_notification(format!("Deleted: {}", name));
                    } else {
                        self.state.show_notification(format!("Failed to delete: {}", name));
                    }
                    
                    self.mode = AppMode::Normal;
                    self.refresh_data();
                }
                ModalType::Message(_) => {
                    self.mode = AppMode::Normal;
                }
            }
            
            return Ok(());
        }
        
        // Handle other modal actions
        if let AppMode::Modal(modal) = &mut self.mode {
            match key {
                KeyCode::Up => {
                    // If dropdown is open, navigate dropdown
                    if modal.active_field == 3 && 
                       matches!(modal.modal_type, ModalType::AddStudent | ModalType::EditStudent(_)) &&
                       modal.major_dropdown.is_open {
                        modal.major_dropdown.select_prev();
                    } else {
                        modal.prev_field();
                    }
                }
                KeyCode::Down => {
                    // If dropdown is open, navigate dropdown
                    if modal.active_field == 3 && 
                       matches!(modal.modal_type, ModalType::AddStudent | ModalType::EditStudent(_)) &&
                       modal.major_dropdown.is_open {
                        modal.major_dropdown.select_next();
                    } else {
                        modal.next_field();
                    }
                }
                KeyCode::Tab => {
                    modal.next_field();
                }
                KeyCode::BackTab => {
                    modal.prev_field();
                }
                KeyCode::Backspace => {
                    modal.backspace();
                }
                KeyCode::Char(' ') => {
                    // Special handling for Space key on Major field - toggle dropdown
                    if modal.active_field == 3 && 
                       matches!(modal.modal_type, ModalType::AddStudent | ModalType::EditStudent(_)) {
                        modal.major_dropdown.toggle_open();
                    } else {
                        modal.input(' ');
                    }
                }
                KeyCode::Char(c) => {
                    // Handle regular character input (including 'j' and 'k')
                    modal.input(c);
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    fn perform_search(&mut self) {
        if self.state.search_query.is_empty() {
            self.refresh_data();
            return;
        }

        let query = &self.state.search_query;
        
        match self.state.active_tab {
            ActiveTab::Students => {
                let results = self.data_manager.search_students(query);
                self.state.show_notification(format!("Found {} matching students", results.len()));
            }
            ActiveTab::Teachers => {
                let results = self.data_manager.search_teachers(query);
                self.state.show_notification(format!("Found {} matching teachers", results.len()));
            }
            ActiveTab::Faculties => {
                let results = self.data_manager.search_faculties(query);
                self.state.show_notification(format!("Found {} matching faculties", results.len()));
            }
        }
    }

    fn refresh_data(&mut self) {
        // Reset table selection if needed
        match self.state.active_tab {
            ActiveTab::Students => {
                if self.data_manager.get_all_students().is_empty() {
                    self.state.student_list_state.select(None);
                } else {
                    self.state.student_list_state.select(Some(0));
                }
            }
            ActiveTab::Teachers => {
                if self.data_manager.get_all_teachers().is_empty() {
                    self.state.teacher_list_state.select(None);
                } else {
                    self.state.teacher_list_state.select(Some(0));
                }
            }
            ActiveTab::Faculties => {
                if self.data_manager.get_all_faculties().is_empty() {
                    self.state.faculty_list_state.select(None);
                } else {
                    self.state.faculty_list_state.select(Some(0));
                }
            }
        }
    }

    fn tick(&mut self) -> Result<()> {
        let now = Instant::now();
        if now.duration_since(self.last_tick) >= self.tick_rate {
            self.last_tick = now;
            self.state.update_notification_timer();
        }
        
        Ok(())
    }

    fn show_add_modal(&mut self) {
        let modal_type = match self.state.active_tab {
            ActiveTab::Students => ModalType::AddStudent,
            ActiveTab::Teachers => ModalType::AddTeacher,
            ActiveTab::Faculties => ModalType::AddFaculty,
        };
        
        self.mode = AppMode::Modal(Modal::new(modal_type));
    }

    fn show_edit_modal(&mut self) {
        match self.state.active_tab {
            ActiveTab::Students => {
                let state = &mut self.state.student_list_state;
                if let Some(index) = state.selected() {
                    let students = self.data_manager.get_all_students();
                    if index < students.len() {
                        let student = students[index].clone();
                        self.mode = AppMode::Modal(Modal::new(ModalType::EditStudent(student)));
                    } else {
                        self.state.show_notification("No student selected".to_string());
                    }
                } else {
                    self.state.show_notification("No student selected".to_string());
                }
            }
            ActiveTab::Teachers => {
                let state = &mut self.state.teacher_list_state;
                if let Some(index) = state.selected() {
                    let teachers = self.data_manager.get_all_teachers();
                    if index < teachers.len() {
                        let teacher = teachers[index].clone();
                        self.mode = AppMode::Modal(Modal::new(ModalType::EditTeacher(teacher)));
                    } else {
                        self.state.show_notification("No teacher selected".to_string());
                    }
                } else {
                    self.state.show_notification("No teacher selected".to_string());
                }
            }
            ActiveTab::Faculties => {
                let state = &mut self.state.faculty_list_state;
                if let Some(index) = state.selected() {
                    let faculties = self.data_manager.get_all_faculties();
                    if index < faculties.len() {
                        let faculty = faculties[index].clone();
                        self.mode = AppMode::Modal(Modal::new(ModalType::EditFaculty(faculty)));
                    } else {
                        self.state.show_notification("No faculty selected".to_string());
                    }
                } else {
                    self.state.show_notification("No faculty selected".to_string());
                }
            }
        }
    }

    fn show_delete_modal(&mut self) {
        match self.state.active_tab {
            ActiveTab::Students => {
                let state = &mut self.state.student_list_state;
                if let Some(index) = state.selected() {
                    let students = self.data_manager.get_all_students();
                    if index < students.len() {
                        let student = &students[index];
                        let modal_type = ModalType::DeleteConfirmation(
                            student.id.clone(),
                            student.full_name(),
                        );
                        self.mode = AppMode::Modal(Modal::new(modal_type));
                    } else {
                        self.state.show_notification("No student selected".to_string());
                    }
                } else {
                    self.state.show_notification("No student selected".to_string());
                }
            }
            ActiveTab::Teachers => {
                let state = &mut self.state.teacher_list_state;
                if let Some(index) = state.selected() {
                    let teachers = self.data_manager.get_all_teachers();
                    if index < teachers.len() {
                        let teacher = &teachers[index];
                        let modal_type = ModalType::DeleteConfirmation(
                            teacher.id.clone(),
                            teacher.full_name(),
                        );
                        self.mode = AppMode::Modal(Modal::new(modal_type));
                    } else {
                        self.state.show_notification("No teacher selected".to_string());
                    }
                } else {
                    self.state.show_notification("No teacher selected".to_string());
                }
            }
            ActiveTab::Faculties => {
                let state = &mut self.state.faculty_list_state;
                if let Some(index) = state.selected() {
                    let faculties = self.data_manager.get_all_faculties();
                    if index < faculties.len() {
                        let faculty = &faculties[index];
                        let modal_type = ModalType::DeleteConfirmation(
                            faculty.id.clone(),
                            faculty.name.clone(),
                        );
                        self.mode = AppMode::Modal(Modal::new(modal_type));
                    } else {
                        self.state.show_notification("No faculty selected".to_string());
                    }
                } else {
                    self.state.show_notification("No faculty selected".to_string());
                }
            }
        }
    }
}

// Helper function to get terminal dimensions
fn terminal_size() -> Rect {
    let size = crossterm::terminal::size()
        .unwrap_or((80, 24));
    Rect::new(0, 0, size.0, size.1)
}

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create and run app
    let mut app = App::new()?;
    let result = app.run(&mut terminal);
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    result
}
