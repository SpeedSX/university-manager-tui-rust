use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Student {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub age: u32,
    pub major: String,
    pub gpa: f32,
}

impl Student {
    pub fn new(first_name: String, last_name: String, age: u32, major: String, gpa: f32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            first_name,
            last_name,
            age,
            major,
            gpa,
        }
    }

    pub fn with_id(
        id: String,
        first_name: String,
        last_name: String,
        age: u32,
        major: String,
        gpa: f32,
    ) -> Self {
        Self {
            id,
            first_name,
            last_name,
            age,
            major,
            gpa,
        }
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Teacher {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub age: u32,
    pub department: String,
    pub title: String,
}

impl Teacher {
    pub fn new(
        first_name: String,
        last_name: String,
        age: u32,
        department: String,
        title: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            first_name,
            last_name,
            age,
            department,
            title,
        }
    }

    pub fn with_id(
        id: String,
        first_name: String,
        last_name: String,
        age: u32,
        department: String,
        title: String,
    ) -> Self {
        Self {
            id,
            first_name,
            last_name,
            age,
            department,
            title,
        }
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faculty {
    pub id: String,
    pub name: String,
    pub building: String,
    pub head_name: String,
    pub established_year: u32,
    pub num_staff: u32,
}

impl Faculty {
    pub fn new(
        name: String,
        building: String,
        head_name: String,
        established_year: u32,
        num_staff: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            building,
            head_name,
            established_year,
            num_staff,
        }
    }

    pub fn with_id(
        id: String,
        name: String,
        building: String,
        head_name: String,
        established_year: u32,
        num_staff: u32,
    ) -> Self {
        Self {
            id,
            name,
            building,
            head_name,
            established_year,
            num_staff,
        }
    }
}