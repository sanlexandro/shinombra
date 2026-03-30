use crate::unit::{Millimeters, Pixels};

pub struct Geometry {
    pub x: Millimeters,
    pub y: Millimeters,
    pub alg: Alg,

    pub unset: Pixels,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alg {
    Alg1,
    Alg2,
}

impl Geometry {
    pub fn calculate (& mut self) {
        self.unset = Pixels((self.x.0 + self.y.0) as usize);
    }

    pub fn print(&self) {
        println!("unset = {}, alg = {}", self.unset.0, if self.alg == Alg::Alg1 {"alg1"} else {"alg2"});
    }
}