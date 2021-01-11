// Copyright (C) 2020 peter
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

mod customer;
mod prelude;
mod taxnumber;

use gzlib::proto::customer::customer_server::*;
use gzlib::proto::customer::*;
use packman::*;
use prelude::*;
use std::path::PathBuf;
use taxnumber::*;
use tokio::sync::{oneshot, Mutex};
use tonic::{transport::Server, Request, Response, Status};

// Customer service
//
// Related to manage all customer related
// tasks. Create, list, update, lookup, etc.
//
// Important
// =========
// As customer has a key role systemwide,
// we cannot remove a customer object anyway.
struct CustomerService {
  customers: Mutex<VecPack<customer::Customer>>, // Customers db
}

// Init customer service
// Load database, load related service clients
// set alias lookup table and next id
impl CustomerService {
  // Init CustomerService
  fn init(customers: VecPack<customer::Customer>, // Customers db
  ) -> CustomerService {
    CustomerService {
      customers: Mutex::new(customers),
    }
  }
  // Get next customer ID
  async fn next_customer_id(&self) -> u32 {
    let mut latest_id: u32 = 0;
    self.customers.lock().await.iter().for_each(|customer| {
      let id: u32 = *customer.unpack().get_id();
      if id > latest_id {
        latest_id = id;
      }
    });
    latest_id + 1
  }
  // Create new customer
  async fn create_new(&self, u: NewCustomerObj) -> ServiceResult<CustomerObj> {
    // Check taxnumber
    let taxnumber = match u.tax_number.len() {
      x if x > 0 => Some(TaxNumber::new(&u.tax_number)?),
      _ => None,
    };
    // Get the next customer ID
    let next_customer_id = self.next_customer_id().await;

    // Create customer object
    let new_customer = customer::Customer::new(
      next_customer_id,
      u.name,
      u.email,
      u.phone,
      taxnumber,
      u.address_zip,
      u.address_location,
      u.address_street,
      u.created_by,
    )?;

    // Store new customer into storage
    self.customers.lock().await.insert(new_customer.clone())?;

    // Returns customer proto object
    Ok(new_customer.into())
  }
  // Get all customer IDs
  async fn get_all(&self) -> ServiceResult<Vec<u32>> {
    let res = self
      .customers
      .lock()
      .await
      .iter()
      .map(|c| c.unpack().id)
      .collect::<Vec<u32>>();
    Ok(res)
  }
  // Get customer by ID
  async fn get_by_id(&self, r: GetByIdRequest) -> ServiceResult<CustomerObj> {
    let res = self
      .customers
      .lock()
      .await
      .find_id(&r.customer_id)?
      .unpack()
      .clone();
    Ok(res.into())
  }
  // Get customers in bulk
  async fn get_bulk(&self, r: GetBulkRequest) -> ServiceResult<Vec<CustomerObj>> {
    let res = self
      .customers
      .lock()
      .await
      .iter()
      .filter(|c| r.customer_ids.contains(&c.unpack().id))
      .map(|c| c.unpack().clone().into())
      .collect::<Vec<CustomerObj>>();
    Ok(res)
  }
  // Update customer by ID
  async fn update_by_id(&self, r: CustomerObj) -> ServiceResult<CustomerObj> {
    // Check taxnumber
    let taxnumber = match r.tax_number.len() {
      x if x > 0 => Some(TaxNumber::new(&r.tax_number)?),
      _ => None,
    };
    // Update customer
    let res = self
      .customers
      .lock()
      .await
      .find_id_mut(&r.id)?
      .as_mut()
      .unpack()
      .update(
        r.name,
        r.email,
        r.phone,
        taxnumber,
        r.address_zip,
        r.address_location,
        r.address_street,
      )?
      .clone();
    Ok(res.into())
  }
  // Find customers by query
  async fn find_customer(&self, r: FindCustomerRequest) -> ServiceResult<Vec<u32>> {
    let res = self
      .customers
      .lock()
      .await
      .iter()
      .filter(|c| c.unpack().name.to_lowercase().contains(&r.query))
      .map(|c| c.unpack().id)
      .collect::<Vec<u32>>();
    Ok(res)
  }
}

#[tonic::async_trait]
impl Customer for CustomerService {
  async fn create_new(
    &self,
    request: Request<NewCustomerObj>,
  ) -> Result<Response<CustomerObj>, Status> {
    let resp = self.create_new(request.into_inner()).await?;
    Ok(Response::new(resp))
  }

  async fn get_all(&self, _request: Request<()>) -> Result<Response<CustomerIds>, Status> {
    let res = self.get_all().await?;
    Ok(Response::new(CustomerIds { customer_ids: res }))
  }

  async fn get_by_id(
    &self,
    request: Request<GetByIdRequest>,
  ) -> Result<Response<CustomerObj>, Status> {
    let res = self.get_by_id(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  type GetBulkStream = tokio::sync::mpsc::Receiver<Result<CustomerObj, Status>>;

  async fn get_bulk(
    &self,
    request: Request<GetBulkRequest>,
  ) -> Result<Response<Self::GetBulkStream>, Status> {
    // Create channel for stream response
    let (mut tx, rx) = tokio::sync::mpsc::channel(100);

    // Get resources as Vec<SourceObject>
    let res = self.get_bulk(request.into_inner()).await?;

    // Send the result items through the channel
    tokio::spawn(async move {
      for ots in res.into_iter() {
        tx.send(Ok(ots)).await.unwrap();
      }
    });

    // Send back the receiver
    Ok(Response::new(rx))
  }

  async fn update_by_id(
    &self,
    request: Request<CustomerObj>,
  ) -> Result<Response<CustomerObj>, Status> {
    let res = self.update_by_id(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn find_customer(
    &self,
    request: Request<FindCustomerRequest>,
  ) -> Result<Response<CustomerIds>, Status> {
    let res = self.find_customer(request.into_inner()).await?;
    Ok(Response::new(CustomerIds { customer_ids: res }))
  }
}

#[tokio::main]
async fn main() -> prelude::ServiceResult<()> {
  // Load customers db
  let db: VecPack<customer::Customer> = VecPack::try_load_or_init(PathBuf::from("data/customers"))
    .expect("Error while loading customers storage");

  // Init customer service
  let customer_service = CustomerService::init(db);

  let addr = "[::1]:50055".parse().unwrap();

  // Create shutdown channel
  let (tx, rx) = oneshot::channel();

  // Spawn the server into a runtime
  tokio::task::spawn(async move {
    Server::builder()
      .add_service(CustomerServer::new(customer_service))
      .serve_with_shutdown(addr, async { rx.await.unwrap() })
      .await
  });

  tokio::signal::ctrl_c().await.unwrap();

  println!("SIGINT");

  // Send shutdown signal after SIGINT received
  let _ = tx.send(());

  Ok(())
}
