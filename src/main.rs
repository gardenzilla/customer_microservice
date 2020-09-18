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

pub mod customer;
pub mod id;
pub mod prelude;
pub mod taxnumber;

use packman::*;
use prelude::*;
use protos::customer::customer_server::*;
use protos::customer::*;
use protos::email::email_client::*;
use std::path::PathBuf;
use taxnumber::*;
use tokio::sync::{oneshot, Mutex};
use tonic::transport::Channel;
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
//
// Other restrictions
// ==================
// Customer ids are auto generated with the customer
// object, and they remain the same in the future

pub struct CustomerService {
  customers: Mutex<VecPack<customer::Customer>>, // Customers db
  _email_client: Mutex<EmailClient<Channel>>,    // Email service client
}

// Init customer service
// Load database, load related service clients
// set alias lookup table and next id
impl CustomerService {
  fn init(
    customers: VecPack<customer::Customer>, // Customers db
    email_client: EmailClient<Channel>,     // Email service client
  ) -> CustomerService {
    CustomerService {
      customers: Mutex::new(customers),
      _email_client: Mutex::new(email_client),
    }
  }

  async fn create_new_customer(&self, u: CreateNewRequest) -> ServiceResult<CustomerObj> {
    // Check taxnumber
    let taxnumber = match u.tax_number.len() {
      x if x > 0 => Some(TaxNumber::new(&u.tax_number)?),
      _ => None,
    };

    let mut id: String;

    loop {
      id = id::generate_alphanumeric(7); // e.g.: 4rf6r7f
      if self.is_id_available(&id).await {
        break;
      }
    }

    // Create customer object
    let new_customer = customer::Customer::new(
      id,
      u.name,
      u.email,
      u.phone,
      taxnumber,
      customer::Address::new(u.zip, u.location, u.address),
      u.created_by,
    )?;

    let customer_obj: CustomerObj = (&new_customer).into();

    // Store new customer in DB
    self.customers.lock().await.insert(new_customer)?;

    // Returns customer proto object
    Ok(customer_obj)
  }

  // Check wheter an id is available
  // or already taken
  async fn is_id_available(&self, id: &str) -> bool {
    !self.customers.lock().await.find_id(&id).is_ok()
  }
}

#[tonic::async_trait]
impl Customer for CustomerService {
  async fn create_new(
    &self,
    request: Request<CreateNewRequest>,
  ) -> Result<Response<CreateNewResponse>, Status> {
    let resp = self.create_new_customer(request.into_inner()).await?;
    Ok(Response::new(CreateNewResponse {
      customer: Some(resp),
    }))
  }

  // Todo!: this should be a response stream
  //                        !===============
  async fn get_all(&self, _request: Request<()>) -> Result<Response<GetAllResponse>, Status> {
    let customers = self
      .customers
      .lock()
      .await
      .into_iter()
      .map(|i: &mut Pack<customer::Customer>| i.unpack().into())
      .collect::<Vec<CustomerObj>>();
    let response = GetAllResponse {
      customers: customers,
    };
    return Ok(Response::new(response));
  }

  async fn get_by_id(
    &self,
    request: Request<GetByIdRequest>,
  ) -> Result<Response<GetByIdResponse>, Status> {
    let customer: CustomerObj = self
      .customers
      .lock()
      .await
      .find_id(&request.into_inner().customer_id)
      .map_err(|_| Status::not_found("Customer not found"))?
      .unpack()
      .into();
    let response = GetByIdResponse {
      customer: Some(customer),
    };
    return Ok(Response::new(response));
  }

  async fn update_by_id(
    &self,
    request: Request<UpdateByIdRequest>,
  ) -> Result<Response<UpdateByIdResponse>, Status> {
    let _customer: CustomerUpdateObj = match request.into_inner().customer {
      Some(u) => u,
      None => return Err(Status::internal("Request has an empty customer object")),
    };
    let taxnumber = match _customer.tax_number.len() {
      x if x > 0 => Some(TaxNumber::new(&_customer.tax_number)?),
      _ => None,
    };
    let _address = if let Some(addr) = _customer.address {
      customer::Address::new(addr.zip, addr.location, addr.address)
    } else {
      return Err(ServiceError::internal_error("Missing address from message").into());
    };
    let mut lock = self.customers.lock().await;
    let customer = match lock.find_id_mut(&_customer.id) {
      Ok(u) => u,
      Err(err) => return Err(Status::not_found(format!("{}", err))),
    };

    {
      let mut customer_mut = customer.as_mut();
      let mut _customer_mut = customer_mut.unpack();
      _customer_mut.set_name(_customer.name.to_string());
      _customer_mut.set_email(_customer.email.to_string())?;
      _customer_mut.set_phone(_customer.phone.to_string());
      _customer_mut.set_tax_number(taxnumber);
      _customer_mut.set_address(_address);
    }

    let _customer = customer.unpack();

    let response = UpdateByIdResponse {
      customer: Some(_customer.into()),
    };
    return Ok(Response::new(response));
  }

  async fn add_user(
    &self,
    _request: Request<AddUserRequest>,
  ) -> Result<Response<AddUserResponse>, Status> {
    todo!()
  }

  async fn remove_user(
    &self,
    _request: Request<RemoveUserRequest>,
  ) -> Result<Response<RemoveUserResponse>, Status> {
    todo!()
  }
}

#[tokio::main]
async fn main() -> prelude::ServiceResult<()> {
  // Load customers db
  let customers: VecPack<customer::Customer> =
    VecPack::try_load_or_init(PathBuf::from("data/customers"))
      .expect("Error while loading customers storage");

  // Connect to email service
  let email_client = EmailClient::connect("http://[::1]:50053")
    .await
    .expect("Error while connecting to email service");

  // Init customer service
  let customer_service = CustomerService::init(customers, email_client);

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
