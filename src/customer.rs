// Copyright (C) 2020 Peter Mezei
//
// This file is part of Gardenzilla.
//
// Gardenzilla is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 2 of the License, or
// (at your option) any later version.
//
// Gardenzilla is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Gardenzilla.  If not, see <http://www.gnu.org/licenses/>.

use crate::prelude::ServiceError::*;
use crate::prelude::*;
use crate::taxnumber::*;
use chrono::prelude::*;
use packman::*;
use protos::customer::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Customer {
  id: u32,
  alias: String,
  name: String,
  email: String,
  phone: String,
  tax_number: Option<TaxNumber>,
  address: Address,
  date_created: i64,
  created_by: u32,
  has_user: bool,
  users: Vec<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Address {
  zip: String,
  location: String,
  address: String,
}

impl Address {
  pub fn new(zip: String, location: String, address: String) -> Self {
    Self {
      zip,
      location,
      address,
    }
  }
}

impl From<Customer> for CustomerObj {
  fn from(u: Customer) -> Self {
    Self {
      id: u.id,
      date_created: u.date_created,
      created_by: u.created_by,
      name: u.name,
      address: Some(protos::customer::Address {
        zip: u.address.zip,
        location: u.address.location,
        address: u.address.address,
      }),
      email: u.email,
      phone: u.phone,
      tax_number: u.tax_number.unwrap_or_default().into(),
      has_user: u.has_user,
      users: u.users,
    }
  }
}

impl From<&Customer> for CustomerObj {
  fn from(u: &Customer) -> Self {
    let taxnumber: String = match &u.tax_number {
      Some(number) => number.to_owned().into(),
      None => "".into(),
    };
    Self {
      id: u.id,
      date_created: u.date_created,
      created_by: u.created_by,
      name: u.name.to_owned(),
      address: Some(protos::customer::Address {
        zip: u.address.zip.to_owned(),
        location: u.address.location.to_owned(),
        address: u.address.address.to_owned(),
      }),
      email: u.email.to_owned(),
      phone: u.phone.to_owned(),
      tax_number: taxnumber,
      has_user: u.has_user,
      users: u.users.to_owned(),
    }
  }
}

impl Default for Customer {
  fn default() -> Self {
    Self {
      id: 0,
      alias: String::default(),
      name: String::default(),
      email: String::default(),
      phone: String::default(),
      tax_number: None,
      address: Address::default(),
      date_created: Utc::now().timestamp_millis(),
      created_by: 0,
      has_user: false,
      users: Vec::new(),
    }
  }
}

impl TryFrom for Customer {
  type TryFrom = Customer;
}

// impl DateCreated for User {
//     fn get_date_created(&self) -> DateTime<Utc> {
//         self.date_created
//     }
// }

impl Customer {
  pub fn new(
    id: u32,
    alias: String,
    name: String,
    email: String,
    phone: String,
    tax_number: Option<TaxNumber>,
    address: Address,
    created_by: u32,
  ) -> ServiceResult<Self> {
    // Validate Email content
    if !email.contains('@') || !email.contains('.') {
      return Err(BadRequest(
        "Nem megfelelő email cím. Legalább @ jelet és pontot kell tartalmaznia".to_string(),
      ));
    }
    // Validate Name length
    if name.len() > 200 || name.len() < 2 {
      return Err(BadRequest(format!(
        "A név hosszúsága legalább {} max {} karakter",
        2, 200
      )));
    }

    Ok(Self {
      id,
      alias,
      name,
      email,
      phone,
      tax_number,
      address,
      date_created: Utc::now().timestamp_millis(),
      created_by,
      has_user: false,
      users: Vec::new(),
    })
  }
}

impl Customer {
  pub fn get_id(&self) -> &u32 {
    &self.id
  }
  pub fn get_alias(&self) -> &str {
    &self.alias
  }
  pub fn get_date_created(&self) -> i64 {
    self.date_created
  }
  pub fn get_name(&self) -> &str {
    &self.name
  }
  pub fn set_name(&mut self, name: String) -> &Self {
    self.name = name;
    self
  }
  pub fn get_email(&self) -> &str {
    &self.email
  }
  pub fn set_email(&mut self, email: String) -> ServiceResult<&Self> {
    if email.contains('@') && email.contains('.') && email.len() > 5 {
      self.email = email;
      Ok(self)
    } else {
      Err(BadRequest(
        "Rossz email formátum. Legyen legalább 5 karakter, és tartalmazzon @ jelet és pontot"
          .into(),
      ))
    }
  }
  pub fn get_tax_number(&self) -> &Option<TaxNumber> {
    &self.tax_number
  }
  pub fn set_tax_number(&mut self, tax_number: Option<TaxNumber>) -> &Self {
    self.tax_number = tax_number;
    self
  }
  pub fn get_address(&self) -> &Address {
    &self.address
  }
  pub fn set_address(&mut self, address: Address) -> &Self {
    self.address = address;
    self
  }
  pub fn get_phone(&self) -> &str {
    &self.phone
  }
  pub fn set_phone(&mut self, phone: String) -> &Self {
    self.phone = phone;
    self
  }
  pub fn get_created_by(&self) -> u32 {
    self.created_by
  }
  pub fn get_users(&self) -> &Vec<u32> {
    &self.users
  }
}

impl VecPackMember for Customer {
  type Out = u32;
  fn get_id(&self) -> &u32 {
    &self.id
  }
  // fn try_from(from: &str) -> StorageResult<Self::ResultType> {
  //     match deserialize_object(from) {
  //         Ok(res) => Ok(res),
  //         Err(_) => Err(ServiceError::DeserializeServiceError("user has wrong format".to_string())),
  //     }
  // }
}
