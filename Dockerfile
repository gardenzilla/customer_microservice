FROM fedora:33
RUN dnf update -y && dnf clean all -y
WORKDIR /usr/local/bin
COPY ./target/release/customer_microservice /usr/local/bin/customer_microservice
STOPSIGNAL SIGINT
ENTRYPOINT ["customer_microservice"]
