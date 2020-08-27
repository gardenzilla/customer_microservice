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
        let _address = if let Some(addr) = u.address {
            customer::Address::new(addr.zip, addr.location, addr.address)
        } else {
            return Err(ServiceError::internal_error("Missing address from message"));
        };
        let new_customer = customer::Customer::new(
            u.name,
            u.email,
            u.phone,
            TaxNumber::new(&u.tax_number)?,
            _address,
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
        todo!()
    }

    async fn get_all(&self, request: Request<()>) -> Result<Response<GetAllResponse>, Status> {
        todo!()
    }

    async fn get_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<GetByIdResponse>, Status> {
        todo!()
    }

    async fn update_by_id(
        &self,
        request: Request<UpdateByIdRequest>,
    ) -> Result<Response<UpdateByIdResponse>, Status> {
        todo!()
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

    let addr = "[::1]:50051".parse().unwrap();

    Server::builder()
        .add_service(CustomerServer::new(customer_service))
        .serve(addr)
        .await
        .expect("Error while staring server"); // Todo implement ? from<?>

    Ok(())
}
