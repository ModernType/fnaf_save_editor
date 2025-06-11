#![allow(dead_code)]

use std::{error::Error, fmt::Display};
use slint::{Weak, ToSharedString};
use crate::MainWindow;


pub trait ResultExt<T> {
    /// Reports error to user interface through `app` reference ignoring `Ok()` value
    fn report_to_user(self, app: Weak<MainWindow>);
    /// Reports error to user interface through `app` reference if error or process value stored in `Ok()`
    fn process_or_report(self, app: Weak<MainWindow>, f: impl FnOnce(T));
}

impl<T, E: Display> ResultExt<T> for std::result::Result<T, E> {
    fn report_to_user(self, app: Weak<MainWindow>) {
        match self {
            Ok(_) => {},
            Err(e) => app.unwrap().invoke_report(e.to_shared_string()),
        }
    }

    fn process_or_report(self, app: Weak<MainWindow>, f: impl FnOnce(T)) {
        match self {
            Ok(v) => { f(v); },
            Err(e) => app.unwrap().invoke_report(e.to_shared_string()),
        }
    }
}

pub trait ResultExtThread<T> {
    fn report_to_user_from_thread(self, app: Weak<MainWindow>) -> Result<(), slint::EventLoopError>;
    fn process_or_report_from_thread(self, app: Weak<MainWindow>, f: impl FnOnce(T)) -> Result<(), slint::EventLoopError>;
}

impl<T, E: Error + Send + 'static> ResultExtThread<T> for std::result::Result<T, E> {
    fn report_to_user_from_thread(self, app: Weak<MainWindow>) -> Result<(), slint::EventLoopError> {
        match self {
            Ok(_) => Ok(()),
            Err(e) => app.upgrade_in_event_loop(move |ui| ui.invoke_report(e.to_shared_string()))
        }
    }

    fn process_or_report_from_thread(self, app: Weak<MainWindow>, f: impl FnOnce(T)) -> Result<(), slint::EventLoopError> {
        match self {
            Ok(v) => {
                f(v);
                Ok(())
            },
            Err(e) => app.upgrade_in_event_loop(move |ui| ui.invoke_report(e.to_shared_string())),
        }
    }
}