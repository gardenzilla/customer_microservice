pub mod customer;
pub mod id;
pub mod prelude;
pub mod taxnumber;

use prelude::*;
use protos::customer::customer_server::*;
use protos::customer::*;
use protos::email::email_client::*;
use protos::email::*;
use std::path::PathBuf;
use storaget::*;
use taxnumber::*;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::{transport::Server, Request, Response, Status};

pub struct CustomerService {
    customers: Mutex<VecPack<customer::Customer>>,
    email_client: Mutex<EmailClient<Channel>>,
}

impl CustomerService {
    fn new(
        customers: Mutex<VecPack<customer::Customer>>,
        email_client: EmailClient<Channel>,
    ) -> Self {
        Self {
            customers,
            email_client: Mutex::new(email_client),
        }
    }
    async fn create_new_customer(&self, u: CreateNewRequest) -> ServiceResult<CustomerObj> {
        let new_customer = customer::Customer::new(
            u.name,
            u.email,
            u.phone,
            TaxNumber::new(&u.tax_number)?,
            customer::Address::new(u.zip, u.location, u.address),
            u.created_by,
        )?;
        let customer_obj: CustomerObj = (&new_customer).into();
        self.customers.lock().await.insert(new_customer)?;
        Ok(customer_obj)
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
            _customer_mut.set_tax_number(TaxNumber::new(&_customer.tax_number)?);
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
        request: Request<AddUserRequest>,
    ) -> Result<Response<AddUserResponse>, Status> {
        todo!()
    }

    async fn remove_user(
        &self,
        request: Request<RemoveUserRequest>,
    ) -> Result<Response<RemoveUserResponse>, Status> {
        todo!()
    }
}

#[tokio::main]
async fn main() -> prelude::ServiceResult<()> {
    let customers: Mutex<VecPack<customer::Customer>> = Mutex::new(
        VecPack::try_load_or_init(PathBuf::from("data/customers"))
            .expect("Error while loading customers storage"),
    );

    let email_client = EmailClient::connect("http://[::1]:50053")
        .await
        .expect("Error while connecting to email service");

    let customer_service = CustomerService::new(customers, email_client);

    let addr = "[::1]:50055".parse().unwrap();

    Server::builder()
        .add_service(CustomerServer::new(customer_service))
        .serve(addr)
        .await
        .expect("Error while staring server"); // Todo implement ? from<?>

    Ok(())
}
