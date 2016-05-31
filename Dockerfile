FROM ubuntu:16.04
MAINTAINER Dan Elbert

RUN apt-get -y update && apt-get -y upgrade
RUN apt-get -y install curl file less vim crossbuild-essential-armhf
RUN rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf > /tmp/sh.rustup.sh
RUN chmod +x /tmp/sh.rustup.sh
RUN /tmp/sh.rustup.sh -y
ENV PATH /root/.cargo/bin:$PATH

RUN echo "[target.armv7-unknown-linux-gnueabihf]\nlinker = \"arm-linux-gnueabihf-gcc\"" > /root/.cargo/config

RUN rustup target add armv7-unknown-linux-gnueabihf

RUN mkdir -p /code
COPY . /code
WORKDIR /code

CMD ["cargo", "build", "--target=armv7-unknown-linux-gnueabihf"]
