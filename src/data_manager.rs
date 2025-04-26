use crate::models::{Faculty, Student, Teacher};
use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

pub struct DataManager {
    data_dir: PathBuf,
    students: Vec<Student>,
    teachers: Vec<Teacher>,
    faculties: Vec<Faculty>,
}

impl DataManager {
    pub fn new(data_dir: Option<PathBuf>) -> Result<Self> {
        // Use the specified data directory or create a default one
        let data_dir = match data_dir {
            Some(dir) => dir,
            None => PathBuf::from("data"),
        };

        // Create the data directory if it doesn't exist
        fs::create_dir_all(&data_dir).context("Failed to create data directory")?;

        // Initialize an empty data manager
        let mut dm = Self {
            data_dir,
            students: Vec::new(),
            teachers: Vec::new(),
            faculties: Vec::new(),
        };

        // Load data
        dm.load_data()?;

        Ok(dm)
    }

    // Helper method to load data from JSON files
    fn load_data(&mut self) -> Result<()> {
        self.students = self.load_from_file("students.json").unwrap_or_default();
        self.teachers = self.load_from_file("teachers.json").unwrap_or_default();
        self.faculties = self.load_from_file("faculties.json").unwrap_or_default();
        Ok(())
    }

    // Generic method to load entities from a JSON file
    fn load_from_file<T: DeserializeOwned>(&self, filename: &str) -> Result<Vec<T>> {
        let file_path = self.data_dir.join(filename);

        if !file_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&file_path).context(format!("Failed to open {}", filename))?;
        let reader = BufReader::new(file);
        let data = serde_json::from_reader(reader).context(format!("Failed to parse {}", filename))?;
        Ok(data)
    }

    // Generic method to save entities to a JSON file
    fn save_to_file<T: Serialize>(&self, data: &[T], filename: &str) -> Result<()> {
        let file_path = self.data_dir.join(filename);
        let file = File::create(&file_path).context(format!("Failed to create {}", filename))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, data)
            .context(format!("Failed to write data to {}", filename))?;
        Ok(())
    }

    // Student methods
    pub fn get_all_students(&self) -> &[Student] {
        &self.students
    }

    pub fn add_student(&mut self, student: Student) -> Result<()> {
        self.students.push(student);
        self.save_students()
    }

    pub fn get_student_by_id(&self, id: &str) -> Option<&Student> {
        self.students.iter().find(|s| s.id == id)
    }

    pub fn update_student(&mut self, updated_student: Student) -> Result<bool> {
        if let Some(index) = self.students.iter().position(|s| s.id == updated_student.id) {
            self.students[index] = updated_student;
            self.save_students()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn delete_student(&mut self, id: &str) -> Result<bool> {
        let len_before = self.students.len();
        self.students.retain(|s| s.id != id);
        
        if self.students.len() < len_before {
            self.save_students()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn search_students(&self, query: &str) -> Vec<&Student> {
        let query = query.to_lowercase();
        self.students
            .iter()
            .filter(|s| {
                s.first_name.to_lowercase().contains(&query)
                    || s.last_name.to_lowercase().contains(&query)
                    || s.major.to_lowercase().contains(&query)
            })
            .collect()
    }

    fn save_students(&self) -> Result<()> {
        self.save_to_file(&self.students, "students.json")
    }

    // Teacher methods
    pub fn get_all_teachers(&self) -> &[Teacher] {
        &self.teachers
    }

    pub fn add_teacher(&mut self, teacher: Teacher) -> Result<()> {
        self.teachers.push(teacher);
        self.save_teachers()
    }

    pub fn get_teacher_by_id(&self, id: &str) -> Option<&Teacher> {
        self.teachers.iter().find(|t| t.id == id)
    }

    pub fn update_teacher(&mut self, updated_teacher: Teacher) -> Result<bool> {
        if let Some(index) = self.teachers.iter().position(|t| t.id == updated_teacher.id) {
            self.teachers[index] = updated_teacher;
            self.save_teachers()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn delete_teacher(&mut self, id: &str) -> Result<bool> {
        let len_before = self.teachers.len();
        self.teachers.retain(|t| t.id != id);
        
        if self.teachers.len() < len_before {
            self.save_teachers()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn search_teachers(&self, query: &str) -> Vec<&Teacher> {
        let query = query.to_lowercase();
        self.teachers
            .iter()
            .filter(|t| {
                t.first_name.to_lowercase().contains(&query)
                    || t.last_name.to_lowercase().contains(&query)
                    || t.department.to_lowercase().contains(&query)
                    || t.title.to_lowercase().contains(&query)
            })
            .collect()
    }

    fn save_teachers(&self) -> Result<()> {
        self.save_to_file(&self.teachers, "teachers.json")
    }

    // Faculty methods
    pub fn get_all_faculties(&self) -> &[Faculty] {
        &self.faculties
    }

    pub fn add_faculty(&mut self, faculty: Faculty) -> Result<()> {
        self.faculties.push(faculty);
        self.save_faculties()
    }

    pub fn get_faculty_by_id(&self, id: &str) -> Option<&Faculty> {
        self.faculties.iter().find(|f| f.id == id)
    }

    pub fn update_faculty(&mut self, updated_faculty: Faculty) -> Result<bool> {
        if let Some(index) = self.faculties.iter().position(|f| f.id == updated_faculty.id) {
            self.faculties[index] = updated_faculty;
            self.save_faculties()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn delete_faculty(&mut self, id: &str) -> Result<bool> {
        let len_before = self.faculties.len();
        self.faculties.retain(|f| f.id != id);
        
        if self.faculties.len() < len_before {
            self.save_faculties()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn search_faculties(&self, query: &str) -> Vec<&Faculty> {
        let query = query.to_lowercase();
        self.faculties
            .iter()
            .filter(|f| {
                f.name.to_lowercase().contains(&query)
                    || f.building.to_lowercase().contains(&query)
                    || f.head_name.to_lowercase().contains(&query)
            })
            .collect()
    }

    fn save_faculties(&self) -> Result<()> {
        self.save_to_file(&self.faculties, "faculties.json")
    }
}