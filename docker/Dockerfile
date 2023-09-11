FROM ubuntu

RUN apt update
RUN apt upgrade -y
RUN apt update
RUN apt install -y nano vim nasm xorriso grub-pc-bin grub-common make binutils gcc build-essential curl
RUN curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

VOLUME /root/env
WORKDIR /root/env
