// mod.rs

// *************************************************************************
// * Copyright (C) 2019 Daniel Mueller (deso@posteo.net)                   *
// *                                                                       *
// * This program is free software: you can redistribute it and/or modify  *
// * it under the terms of the GNU General Public License as published by  *
// * the Free Software Foundation, either version 3 of the License, or     *
// * (at your option) any later version.                                   *
// *                                                                       *
// * This program is distributed in the hope that it will be useful,       *
// * but WITHOUT ANY WARRANTY; without even the implied warranty of        *
// * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the         *
// * GNU General Public License for more details.                          *
// *                                                                       *
// * You should have received a copy of the GNU General Public License     *
// * along with this program.  If not, see <http://www.gnu.org/licenses/>. *
// *************************************************************************

use std::ffi;
use std::fmt;

use nitrokey_test::test as test_device;

mod config;
mod encrypted;
mod hidden;
mod lock;
mod otp;
mod pin;
mod pws;
mod reset;
mod run;
mod status;
mod unencrypted;

/// A trait simplifying checking for expected errors.
pub trait UnwrapError {
  /// Unwrap an Error::Error variant.
  fn unwrap_str_err(self) -> String;
  /// Unwrap a Error::CommandError variant.
  fn unwrap_cmd_err(self) -> (Option<&'static str>, nitrokey::CommandError);
  /// Unwrap a Error::LibraryError variant.
  fn unwrap_lib_err(self) -> (Option<&'static str>, nitrokey::LibraryError);
}

impl<T> UnwrapError for crate::Result<T>
where
  T: fmt::Debug,
{
  fn unwrap_str_err(self) -> String {
    match self.unwrap_err() {
      crate::Error::Error(err) => err,
      err => panic!("Unexpected error variant found: {:?}", err),
    }
  }

  fn unwrap_cmd_err(self) -> (Option<&'static str>, nitrokey::CommandError) {
    match self.unwrap_err() {
      crate::Error::NitrokeyError(ctx, err) => match err {
        nitrokey::Error::CommandError(err) => (ctx, err),
        err => panic!("Unexpected error variant found: {:?}", err),
      },
      err => panic!("Unexpected error variant found: {:?}", err),
    }
  }

  fn unwrap_lib_err(self) -> (Option<&'static str>, nitrokey::LibraryError) {
    match self.unwrap_err() {
      crate::Error::NitrokeyError(ctx, err) => match err {
        nitrokey::Error::LibraryError(err) => (ctx, err),
        err => panic!("Unexpected error variant found: {:?}", err),
      },
      err => panic!("Unexpected error variant found: {:?}", err),
    }
  }
}

struct Nitrocli {
  model: Option<nitrokey::Model>,
  admin_pin: Option<ffi::OsString>,
  user_pin: Option<ffi::OsString>,
  new_admin_pin: Option<ffi::OsString>,
  new_user_pin: Option<ffi::OsString>,
  password: Option<ffi::OsString>,
}

impl Nitrocli {
  pub fn new() -> Self {
    Self {
      model: None,
      admin_pin: Some(nitrokey::DEFAULT_ADMIN_PIN.into()),
      user_pin: Some(nitrokey::DEFAULT_USER_PIN.into()),
      new_admin_pin: None,
      new_user_pin: None,
      password: None,
    }
  }

  pub fn with_model<M>(model: M) -> Self
  where
    M: Into<nitrokey::Model>,
  {
    Self {
      model: Some(model.into()),
      admin_pin: Some(nitrokey::DEFAULT_ADMIN_PIN.into()),
      user_pin: Some(nitrokey::DEFAULT_USER_PIN.into()),
      new_admin_pin: None,
      new_user_pin: None,
      password: Some("1234567".into()),
    }
  }

  pub fn admin_pin(&mut self, pin: impl Into<ffi::OsString>) {
    self.admin_pin = Some(pin.into())
  }

  pub fn new_admin_pin(&mut self, pin: impl Into<ffi::OsString>) {
    self.new_admin_pin = Some(pin.into())
  }

  pub fn user_pin(&mut self, pin: impl Into<ffi::OsString>) {
    self.user_pin = Some(pin.into())
  }

  pub fn new_user_pin(&mut self, pin: impl Into<ffi::OsString>) {
    self.new_user_pin = Some(pin.into())
  }

  fn model_to_arg(model: nitrokey::Model) -> &'static str {
    match model {
      nitrokey::Model::Pro => "--model=pro",
      nitrokey::Model::Storage => "--model=storage",
    }
  }

  fn do_run<F, R>(&mut self, args: &[&str], f: F) -> (R, Vec<u8>, Vec<u8>)
  where
    F: FnOnce(&mut crate::RunCtx<'_>, Vec<String>) -> R,
  {
    let args = ["nitrocli"]
      .into_iter()
      .cloned()
      .chain(self.model.map(Self::model_to_arg))
      .chain(args.into_iter().cloned())
      .map(ToOwned::to_owned)
      .collect();

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let ctx = &mut crate::RunCtx {
      stdout: &mut stdout,
      stderr: &mut stderr,
      admin_pin: self.admin_pin.clone(),
      user_pin: self.user_pin.clone(),
      new_admin_pin: self.new_admin_pin.clone(),
      new_user_pin: self.new_user_pin.clone(),
      password: self.password.clone(),
      no_cache: true,
    };

    (f(ctx, args), stdout, stderr)
  }

  /// Run `nitrocli`'s `run` function.
  pub fn run(&mut self, args: &[&str]) -> (i32, Vec<u8>, Vec<u8>) {
    self.do_run(args, |c, a| crate::run(c, a))
  }

  /// Run `nitrocli`'s `handle_arguments` function.
  pub fn handle(&mut self, args: &[&str]) -> crate::Result<String> {
    let (res, out, _) = self.do_run(args, |c, a| crate::args::handle_arguments(c, a));
    res.map(|_| String::from_utf8_lossy(&out).into_owned())
  }

  pub fn model(&self) -> Option<nitrokey::Model> {
    self.model
  }
}
