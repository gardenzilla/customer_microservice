FROM ubuntu:latest
WORKDIR /usr/local/bin
COPY ./target/release/customer_microservice /usr/local/bin/customer_microservice
RUN apt-get update && apt-get install -y
RUN apt-get install curl -y
CMD ["customer_microservice"]