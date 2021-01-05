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
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Customer {
  pub id: u32,
  pub name: String,
  pub email: String,
  pub phone: String,
  pub tax_number: Option<TaxNumber>,
  pub address_zip: String,
  pub address_location: String,
  pub address_street: String,
  pub date_created: DateTime<Utc>,
  pub created_by: u32,
}

impl Default for Customer {
  fn default() -> Self {
    Self {
      id: 0,
      name: String::default(),
      email: String::default(),
      phone: String::default(),
      tax_number: None,
      address_zip: String::default(),
      address_location: String::default(),
      address_street: String::default(),
      date_created: Utc::now(),
      created_by: 0,
    }
  }
}

impl TryFrom for Customer {
  type TryFrom = Customer;
}

impl Customer {
  pub fn new(
    id: u32,
    name: String,
    email: String,
    phone: String,
    tax_number: Option<TaxNumber>,
    address_zip: String,
    address_location: String,
    address_street: String,
    created_by: u32,
  ) -> ServiceResult<Self> {
    // Validate Email content
    if email.len() > 0 {
      // If there is any provided email text
      if !email.contains('@') || !email.contains('.') {
        return Err(BadRequest(
          "Nem megfelelő email cím. Legalább @ jelet és pontot kell tartalmaznia".to_string(),
        ));
      }
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
      name,
      email,
      phone,
      tax_number,
      address_zip,
      address_location,
      address_street,
      date_created: Utc::now(),
      created_by,
    })
  }
}

impl Customer {
  pub fn update(
    &mut self,
    name: String,
    email: String,
    phone: String,
    tax_number: Option<TaxNumber>,
    address_zip: String,
    address_location: String,
    address_street: String,
  ) -> ServiceResult<&Self> {
    self.set_email(email)?;
    self.name = name;
    self.phone = phone;
    self.tax_number = tax_number;
    self.address_zip = address_zip;
    self.address_location = address_location;
    self.address_street = address_street;
    Ok(self)
  }
  pub fn set_email(&mut self, email: String) -> ServiceResult<&Self> {
    if email.len() > 0 {
      if email.contains('@') && email.contains('.') && email.len() > 5 {
        self.email = email;
        return Ok(self);
      } else {
        return Err(BadRequest(
          "Rossz email formátum. Legyen legalább 5 karakter, és tartalmazzon @ jelet és pontot"
            .into(),
        ));
      }
    }
    Ok(self)
  }
}

impl VecPackMember for Customer {
  type Out = u32;
  fn get_id(&self) -> &Self::Out {
    &self.id
  }
}
