//! This file is part of the easybox package.
//
// (c) Xing Huang <navihx@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
//! This mod does not implement parentheses operator, because the expr parser parse the exprs
//! recursively by parentheses.

use uucore::error::UResult;

use super::{FindFilter, FindInstruction};
use std::fmt::Debug;

/// Logical operator and
#[derive(Debug)]
pub struct And {
    f1: Box<dyn FindFilter>,
    f2: Box<dyn FindFilter>,
}

/// Logical operator or
#[derive(Debug)]
pub struct Or {
    f1: Box<dyn FindFilter>,
    f2: Box<dyn FindFilter>,
}

/// Logical operator not
#[derive(Debug)]
pub struct Not {
    f: Box<dyn FindFilter>,
}

impl And {
    ///
    pub fn new(f1: Box<dyn FindFilter>, f2: Box<dyn FindFilter>) -> Self {
        Self { f1, f2 }
    }
}

impl Or {
    ///
    pub fn new(f1: Box<dyn FindFilter>, f2: Box<dyn FindFilter>) -> Self {
        Self { f1, f2 }
    }
}

impl Not {
    ///
    pub fn new(f: Box<dyn FindFilter>) -> Self {
        Self { f }
    }
}

impl FindFilter for And {
    fn filter(&mut self, _file: &super::FindFile) -> UResult<bool> {
        unreachable!()
    }

    fn filter_with_side_effects(
        &mut self,
        file: &super::FindFile,
        side_effecst: &mut Vec<FindInstruction>,
    ) -> UResult<bool> {
        Ok(self.f1.filter_with_side_effects(file, side_effecst)?
            && self.f2.filter_with_side_effects(file, side_effecst)?)
    }

    fn has_side_effects(&self) -> bool {
        self.f1.has_side_effects() || self.f2.has_side_effects()
    }

    fn based_on_name(&self) -> bool {
        self.f1.based_on_name() && self.f2.based_on_name()
    }
}

impl FindFilter for Or {
    fn filter(&mut self, _file: &super::FindFile) -> UResult<bool> {
        unreachable!()
    }

    fn filter_with_side_effects(
        &mut self,
        file: &super::FindFile,
        side_effects: &mut Vec<FindInstruction>,
    ) -> UResult<bool> {
        Ok(self.f1.filter_with_side_effects(file, side_effects)?
            || self.f2.filter_with_side_effects(file, side_effects)?)
    }

    fn has_side_effects(&self) -> bool {
        self.f1.has_side_effects() || self.f2.has_side_effects()
    }

    fn based_on_name(&self) -> bool {
        self.f1.based_on_name() && self.f2.based_on_name()
    }
}

impl FindFilter for Not {
    fn filter(&mut self, _file: &super::FindFile) -> UResult<bool> {
        unreachable!()
    }

    fn filter_with_side_effects(
        &mut self,
        file: &super::FindFile,
        side_effects: &mut Vec<FindInstruction>,
    ) -> UResult<bool> {
        Ok(!self.f.filter_with_side_effects(file, side_effects)?)
    }

    fn has_side_effects(&self) -> bool {
        self.f.has_side_effects()
    }

    fn based_on_name(&self) -> bool {
        self.f.based_on_name()
    }
}

#[derive(Debug)]

///
pub struct Comma {
    car: Box<dyn FindFilter>,
    cdr: Box<dyn FindFilter>,
}

impl Comma {
    ///
    pub fn new(car: Box<dyn FindFilter>, cdr: Box<dyn FindFilter>) -> Self {
        Self { car, cdr }
    }
}

impl FindFilter for Comma {
    fn filter(&mut self, _file: &super::FindFile) -> UResult<bool> {
        unreachable!()
    }

    fn filter_with_side_effects(
        &mut self,
        file: &super::FindFile,
        side_effects: &mut Vec<FindInstruction>,
    ) -> UResult<bool> {
        self.car.filter_with_side_effects(file, side_effects)?;
        self.cdr.filter_with_side_effects(file, side_effects)
    }

    fn has_side_effects(&self) -> bool {
        self.car.has_side_effects() || self.cdr.has_side_effects()
    }

    fn based_on_name(&self) -> bool {
        self.car.based_on_name() && self.cdr.based_on_name()
    }
}

///
pub fn and(a: Box<dyn FindFilter>, b: Box<dyn FindFilter>) -> Box<dyn FindFilter> {
    Box::new(And::new(a, b))
}

///
pub fn or(a: Box<dyn FindFilter>, b: Box<dyn FindFilter>) -> Box<dyn FindFilter> {
    Box::new(Or::new(a, b))
}

///
pub fn cons(a: Box<dyn FindFilter>, b: Box<dyn FindFilter>) -> Box<dyn FindFilter> {
    Box::new(Comma::new(a, b))
}

///
pub fn not(f: Box<dyn FindFilter>) -> Box<dyn FindFilter> {
    Box::new(Not::new(f))
}
