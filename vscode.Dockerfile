FROM skeggib/rust_dev

RUN apt update
RUN apt install -y openssl
RUN apt install -y libssl-dev
RUN apt install -y pkg-config

RUN apt install locales
RUN locale-gen en_US.UTF-8
RUN update-locale LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8

RUN apt install -y make
