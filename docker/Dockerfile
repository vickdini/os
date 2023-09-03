FROM ubuntu

RUN apt update
RUN apt upgrade -y
RUN apt install -y nano vim nasm xorriso grub-pc-bin grub-common make binutils gcc

VOLUME /root/env
WORKDIR /root/env
