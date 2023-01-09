#!/bin/bash

# echo in green
function echo_green {
    echo -e "\033[32m$1\033[0m"
}

# echo red
function echo_red {
    echo -e "\033[31m$1\033[0m"
}

# Install libraries
echo_green "Installing libraries..."
sudo apt-get update
sudo apt-get -y install git \
                        build-essential \
                        pkg-config \
                        autoconf \
                        libc++-dev \
                        libc++abi-dev \
                        libboost-all-dev \
                        libjson-c-dev \
                        libxml2-dev \
                        libxml2-utils

echo_green "Cloning freesasa..."
cd ~
git clone https://github.com/mittinatten/freesasa.git
cd freesasa
git submodule init
git submodule update

# Echo in green
echo_green "Installing FreeSASA..."
autoreconf -i
./configure
make && sudo make install

# Check if FreeSASA is installed
if [ -f /usr/local/lib/libfreesasa.a ]; then
    echo_green "FreeSASA installed successfully"
else
    echo_red "FreeSASA installation failed"
    exit 1
fi
